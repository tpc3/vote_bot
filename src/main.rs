mod cmds;
mod config;
pub mod built_info {
   // The file has been placed there by the build script.
   include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use cmds::{ping::*, vote::*, help::*};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::{
        gateway::Ready,
        interactions::{Interaction, InteractionData::MessageComponent, InteractionType},
    },
};
use tracing::info;

#[group]
#[commands(ping, vote, help)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if interaction.kind == InteractionType::MessageComponent {
            if let MessageComponent(msg) = interaction.data.as_ref().unwrap() {
                match &*msg.custom_id.to_string() {
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
    let mut client = Client::builder(&config::CONFIG.token)
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
