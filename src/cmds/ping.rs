use crate::cmds::utils;
use ferris_says::say;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{
        application::{component::ButtonStyle, interaction::Interaction, interaction::InteractionResponseType},
        prelude::*,
    },
    prelude::*,
    utils::Colour,
};
use std::io::BufWriter;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let mut buf = vec![];
    {
        let mut f = BufWriter::new(&mut buf);
        say(b"Pong!", 12, &mut f).unwrap();
    }
    let say_str = std::str::from_utf8(&buf).unwrap().to_string();
    let user = &ctx.http.get_current_user().await.unwrap();
    msg.channel_id
        .send_message(&ctx.http, |msg_res| {
            msg_res.embed(|embed| {
                embed.author(|author| {
                    author.icon_url(user.face());
                    author.name(&user.name);
                    author
                });
                embed.title("Ping");
                embed.description(utils::codeblock(&say_str));
                embed.footer(|f| {
                    f.text(msg.timestamp.to_rfc2822());
                    f
                });
                embed.field(
                    crate::cmds::utils::link(
                        &crate::built_info::PKG_NAME.to_string(),
                        &crate::built_info::PKG_HOMEPAGE.to_string(),
                    ),
                    format!(
                        "{} {} {}",
                        crate::built_info::PKG_VERSION,
                        crate::built_info::RUSTC,
                        crate::built_info::TARGET
                    ),
                    false,
                );
                embed.colour(Colour::ORANGE);
                embed
            });
            msg_res.reference_message(msg);
            msg_res.components(|f| {
                f.create_action_row(|row| {
                    row.create_button(|button| {
                        button.label("ping");
                        button.style(ButtonStyle::Primary);
                        button.custom_id("ping");
                        button
                    });
                    row
                });
                f
            });
            msg_res
        })
        .await
        .unwrap();
    Ok(())
}

pub async fn interaction_create(ctx: &Context, interaction: &Interaction) {
    if let Interaction::MessageComponent(i) = interaction {
        if let Err(why) = i
            .create_interaction_response(&ctx, |res| {
                res.kind(InteractionResponseType::ChannelMessageWithSource);
                res.interaction_response_data(|msg| {
                    msg.embed(|embed| {
                        embed.title("Pong!");
                        embed.description("Did you pressed the button...?");
                        embed.colour(Colour::ORANGE);
                        embed
                    });
                    msg
                });
                res
            })
            .await
        {
            println!("{}", why);
        }
    }
}
