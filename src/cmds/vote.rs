use std::collections::HashMap;

use chrono::{DateTime, Utc};
use getopts::{Matches, Options};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    futures::future::{abortable, AbortHandle},
    model::{
        channel::Message,
        interactions::{
            message_component::{ActionRowComponent, ButtonStyle},
            InteractionResponseType,
        },
        prelude::*,
    },
    prelude::*,
    utils::Colour,
};

use crate::cmds::utils;

struct Args {
    title: String,
    description: String,
    choices: Vec<String>,
    due: DateTime<Utc>,
    anonymous: bool,
    mask: bool,
    max: u8,
    editable: bool,
    duplicate: bool,
}

#[derive(Serialize, Deserialize)]
struct Votes {
    votes: Vec<Vec<VoteDetail>>,
    lastupdate: DateTime<Utc>,
    isended: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VoteDetail {
    id: u64,
    time: DateTime<Utc>,
}

pub static OPTIONS: Lazy<Options> = Lazy::new(|| init());
pub static EDITING: Lazy<RwLock<HashMap<u64, AbortHandle>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[command]
async fn vote(ctx: &Context, msg: &Message) -> CommandResult {
    let parsed = parser(&msg.content);
    if let Err(why) = parsed {
        msg.channel_id
            .send_message(&ctx.http, |msg_res| {
                msg_res.embed(|embed| {
                    embed.title("Error");
                    embed.description(why);
                    embed.footer(|f| {
                        f.text(msg.timestamp.to_rfc2822());
                        f
                    });
                    embed.colour(Colour::RED);
                    embed
                });
                msg_res.reference_message(msg);
                msg_res
            })
            .await?;
    } else {
        let args = parsed.unwrap();
        msg.channel_id
            .send_message(&ctx.http, |msg_res| {
                msg_res.embed(|embed| {
                    embed.author(|author| {
                        author.icon_url(msg.author.face());
                        author.name(&msg.author.name);
                        author
                    });
                    embed.title(&args.title);
                    embed.description(&args.description);
                    for choice in &args.choices {
                        embed.field(choice, "-", true);
                    }
                    embed.footer(|f| {
                        f.text(&msg.content);
                        f
                    });
                    embed.colour(Colour::ORANGE);
                    embed
                });
                msg_res.reference_message(msg);
                msg_res.components(|f| {
                    let mut slice = args.choices.clone();
                    let mut choices = Vec::new();
                    while slice.len() > 5 {
                        let (l, r) = slice.split_at(5);
                        choices.push(l.to_vec());
                        slice = r.to_vec();
                    }
                    choices.push(slice);
                    let mut count = 0;
                    for i in choices {
                        f.create_action_row(|row| {
                            for j in 0..i.len() {
                                row.create_button(|button| {
                                    button.label(&i[j]);
                                    button.style(ButtonStyle::Primary);
                                    button.custom_id(format!("choice_{}", count));
                                    button
                                });
                                count = count + 1;
                            }
                            row
                        });
                    }
                    f.create_action_row(|row| {
                        row.create_button(|button| {
                            button.label("End/Restart");
                            button.style(ButtonStyle::Danger);
                            button.custom_id("toggle");
                            button
                        });
                        row
                    });
                    f
                });
                msg_res
            })
            .await?;
    }

    Ok(())
}

pub async fn interaction_create(ctx: &Context, i: &Interaction) {
    if let Interaction::MessageComponent(interaction) = i {
        interaction
            .create_interaction_response(&ctx.http, |res| {
                res.kind(InteractionResponseType::DeferredUpdateMessage);
                res
            })
            .await
            .unwrap();
        let org_msg = &interaction.message;
        let mut args = parser(&org_msg.embeds[0].footer.as_ref().unwrap().text).unwrap();
        let mut votes: Votes = serde_json::from_str(&utils::decrypt_base64_to_string(
            &utils::db_get(&org_msg.id.as_u64().to_string()),
        ))
        .unwrap_or_else(|_| Votes {
            votes: vec![Vec::new(); args.choices.len()],
            lastupdate: Utc::now(),
            isended: false,
        });

        if interaction.data.custom_id.starts_with("choice_") {
            let result = validator(
                &args,
                votes,
                &interaction
                    .data
                    .custom_id
                    .to_string()
                    .replace("choice_", "")
                    .parse()
                    .unwrap(),
                interaction.member.as_ref().unwrap().user.id.as_u64(),
            );
            if let Err(why) = result {
                interaction
                    .member
                    .as_ref()
                    .unwrap()
                    .user
                    .dm(&ctx.http, |msg| {
                        msg.embed(|embed| {
                            embed.title("error");
                            embed.description(format!("Vote wasn't counted: {}", why.to_string()));
                            embed.footer(|footer| {
                                footer.text(Utc::now().to_rfc2822());
                                footer
                            });
                            embed.colour(Colour::RED);
                            embed
                        });
                        msg
                    })
                    .await
                    .unwrap();
                return;
            }
            votes = result.unwrap();
        } else if interaction.data.custom_id == "toggle" {
            if *interaction.member.as_ref().unwrap().user.id.as_u64()
                == utils::icon_url_to_uid(
                    &org_msg.embeds[0]
                        .author
                        .as_ref()
                        .unwrap()
                        .icon_url
                        .as_ref()
                        .unwrap(),
                )
            {
                votes.isended = !votes.isended;
                if votes.isended {
                    args.mask = false;
                }
            }
        }

        let mut value_vec = Vec::new();
        if !args.anonymous {
            for i in 0..votes.votes.len() {
                let mut value = String::new();
                for j in 0..votes.votes[i].len() {
                    value = format!("{}<@{}>\n", value, &votes.votes[i][j].id);
                }
                value_vec.push(value);
            }
        }

        utils::db_insert(
            &org_msg.id.as_u64().to_string(),
            &utils::encrypt_str_to_base64(&serde_json::to_string(&votes).unwrap()),
        );
        let mut m = org_msg.clone();
        let edit = m.edit(&ctx.http, |edit_msg| {
            edit_msg.content(format!(
                "Total vote(s): {}",
                votes.votes.iter().map(Vec::len).sum::<usize>()
            ));
            edit_msg.embed(|embed| {
                embed.author(|author| {
                    author.name(org_msg.embeds[0].author.clone().unwrap().name);
                    author.icon_url(org_msg.embeds[0].author.clone().unwrap().icon_url.unwrap());
                    author
                });
                embed.title(org_msg.embeds[0].title.clone().unwrap());
                embed.description(org_msg.embeds[0].description.clone().unwrap());
                embed.footer(|footer| {
                    footer.text(org_msg.embeds[0].footer.clone().unwrap().text);
                    footer
                });
                for i in 0..org_msg.embeds[0].fields.len() {
                    let mut value;
                    if args.mask {
                        value = "-".to_string();
                    } else {
                        let mut ratio = 0;
                        let total_votes = votes.votes.iter().map(Vec::len).sum::<usize>();
                        if total_votes != 0 {
                            ratio = votes.votes[i].len() * 100 / total_votes;
                        }
                        value = format!("**{} people(s), {}%**\n", votes.votes[i].len(), ratio);
                    }
                    if !args.anonymous && !args.mask {
                        value = value + &value_vec[i];
                    }

                    embed.field(&org_msg.embeds[0].fields[i].name, value, true);
                }
                embed.colour(Colour::ORANGE);
                embed
            });
            edit_msg.components(|f| {
                let mut c = org_msg.components.clone();
                c.remove(c.len() - 1);
                for org_row in c {
                    f.create_action_row(|row| {
                        for j in &org_row.components {
                            if let ActionRowComponent::Button(org_button) = j {
                                row.create_button(|button| {
                                    button.label(org_button.label.as_ref().unwrap());
                                    button.style(org_button.style);
                                    button.custom_id(org_button.custom_id.as_ref().unwrap());
                                    button.disabled(votes.isended);
                                    button
                                });
                            }
                        }
                        row
                    });
                }

                f.create_action_row(|row| {
                    row.create_button(|button| {
                        button.label("End/Restart");
                        button.style(ButtonStyle::Danger);
                        button.custom_id("toggle");
                        button
                    });
                    row
                });
                f
            });
            edit_msg
        });
        let aborted;
        if let Some(handle) = EDITING.read().await.get(&*org_msg.id.as_u64()) {
            handle.abort();
            aborted = true;
        } else {
            aborted = false;
        }
        if aborted {
            loop {
                if EDITING.read().await.contains_key(&*org_msg.id.as_u64()) {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                } else {
                    break;
                }
            }
        }
        let (fut, handle) = abortable(edit);
        EDITING.write().await.insert(*org_msg.id.as_u64(), handle);
        if let Ok(res) = fut.await {
            res.unwrap();
        }
        EDITING.write().await.remove(&*org_msg.id.as_u64());
    }
}

pub fn help() -> String {
    OPTIONS.usage(&(format!("{}{}", crate::config::CONFIG.infos.prefix, "vote")))
}

pub fn init() -> Options {
    for i in utils::db_iter() {
        let votes: Votes = serde_json::from_str(&utils::decrypt_base64_to_string(
            &String::from_utf8(i.as_ref().unwrap().1.to_vec()).unwrap(),
        ))
        .unwrap();
        if votes
            .lastupdate
            .checked_add_signed(chrono::Duration::days(30))
            .unwrap()
            < Utc::now()
        {
            utils::db_remove(&String::from_utf8(i.as_ref().unwrap().0.to_vec()).unwrap());
        }
    }
    let mut options = Options::new();

    options.optopt("d", "description", "set description", "DESCRIPTION");
    options.optopt("t", "due", "set due time/date", "RFC3339");
    options.optopt("x", "max", "Max vote", "NUM");
    options.optflag("a", "anonymous", "anonymous vote");
    options.optflag("m", "mask", "Mask vote status");
    options.optflag("n", "noedit", "Disable editing vote");
    options.optflag("p", "duplicate", "Allow duplicate vote");

    options
}

fn parser(msg: &String) -> std::result::Result<Args, String> {
    let msg_vec: Vec<&str> = msg.split_whitespace().collect();

    let matches: Matches;
    let m = OPTIONS.parse(&msg_vec[1..]);
    if m.is_err() {
        return Err(format!(
            "Request parse error: {}",
            m.unwrap_err().to_string()
        ));
    } else {
        matches = m.unwrap();
    }

    if matches.free.len() < 3 {
        return Err("Not enough params".to_string());
    }

    let title = (&matches.free[0]).to_string();
    let description = matches.opt_str("d").unwrap_or("No description".to_string());
    let d = matches.opt_str("t").unwrap_or("".to_string());
    let mut due: DateTime<Utc> = Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .unwrap();
    if &d != "" {
        due = DateTime::parse_from_rfc3339(&d)
            .expect("Date parse error")
            .with_timezone(&Utc);
    }
    let choices = (&matches.free[1..]).to_vec();
    if choices.len() > 20 {
        return Err("Too many choices".to_string());
    }
    let anonymous = matches.opt_present("a");
    let mask = matches.opt_present("m");
    let max: u8 = matches
        .opt_str("x")
        .unwrap_or(1.to_string())
        .parse()
        .expect("Max vote must be in u8");
    let mut editable = !matches.opt_present("n");
    let duplicate = matches.opt_present("p");
    if duplicate {
        editable = false;
    }
    Ok(Args {
        title,
        description,
        due,
        choices,
        anonymous,
        mask,
        max,
        editable,
        duplicate,
    })
}

fn validator(
    args: &Args,
    mut votes: Votes,
    num: &u8,
    id: &u64,
) -> std::result::Result<Votes, String> {
    //Due
    if args.due < Utc::now() {
        return Err("Vote already ended".to_string());
    }

    //Editable / Cancel
    if votes.votes[*num as usize]
        .iter()
        .any(|votedetail| votedetail.id == *id)
    {
        if args.editable {
            votes.votes[*num as usize].retain(|votedetail| votedetail.id != *id);
            return Ok(votes);
        } else if !args.duplicate {
            return Err("This vote is not editable".to_string());
        }
    }

    // Count
    let mut count = 0;
    for i in 0..votes.votes.len() {
        for j in 0..votes.votes[i].len() {
            if votes.votes[i][j].id == *id {
                count += 1;
            }
        }
    }
    if count + 1 > args.max {
        return Err("You already voted".to_string());
    }

    let id = *id;
    let time = Utc::now();
    votes.votes[*num as usize].push(VoteDetail { id, time });
    votes.lastupdate = Utc::now();
    Ok(votes)
}
