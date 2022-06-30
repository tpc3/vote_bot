mod cmds;
mod config;
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use cmds::{help::*, ping::*, vote::*};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::{
        gateway::Ready,
        interactions::{Interaction, InteractionType},
        prelude::Activity,
    }, prelude::GatewayIntents,
};
use tracing::info;

#[group]
#[commands(ping, vote, help)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        ctx.set_activity(Activity::playing(&config::CONFIG.infos.activity))
            .await;
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if interaction.kind() == InteractionType::MessageComponent {
            if let Interaction::MessageComponent(msg) = interaction.clone() {
                match &*msg.data.custom_id.to_string() {
                    "ping" => cmds::ping::interaction_create(&ctx, &interaction).await,
                    _ => cmds::vote::interaction_create(&ctx, &interaction).await,
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix(&config::CONFIG.infos.prefix);
            c.allow_dm(false);
            c
        })
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let mut client = Client::builder(&config::CONFIG.token, GatewayIntents::default())
        .event_handler(Handler)
        .application_id(config::CONFIG.id)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start_shards(config::CONFIG.shards).await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
