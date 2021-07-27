use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &ctx.http.get_current_user().await.unwrap();
    msg.channel_id
        .send_message(&ctx, |new_msg| {
            new_msg.embed(|embed| {
                embed.author(|author| {
                    author.icon_url(user.face());
                    author.name(&user.name);
                    author
                });
                embed.title("help");
                embed.description("Help (eng only)");
                embed.field("ping", "pong", true);
                embed.field("help", "this page", true);
                embed.field("vote", crate::cmds::vote::help(), false);
                embed.colour(Colour::ORANGE);
                embed
            });
            new_msg
        })
        .await
        .unwrap();
    Ok(())
}
