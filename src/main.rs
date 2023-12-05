use std::env;

use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::framework::standard::{
    macros::{command, group},
    StandardFramework,
    Configuration,
    CommandResult
};

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().prefix("~")); // set the bot's prefix to "~"

    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
