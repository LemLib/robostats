use std::collections::HashMap;

use lazy_static::lazy_static;
use serenity::all::{CommandDataOptionValue, CommandOptionType};
use serenity::builder::{
    CreateCommand,
    CreateCommandOption,
    CreateEmbed,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

lazy_static! {
static ref PRIVILEGES : HashMap<&'static str, (&'static str, &'static str)> = {
        let mut map = HashMap::new();
        map.insert("main", ("https://wiki.purduesigbots.com/", "Sigbots Main Page"));
        map
        };
}

pub fn response(_ctx: &Context, interaction: &CommandInteraction) -> CreateInteractionResponse {
    let name = if let CommandDataOptionValue::String(arg) = &interaction.data.options[0].value {
        Some(arg)
    } else {
        None
    };
    if name.is_none() {
        let message = CreateInteractionResponseMessage::new().content("No argument provided");
        return CreateInteractionResponse::Message(message);
    }
    let uname = name.unwrap().trim();

    if PRIVILEGES.contains_key(uname) {
        let message = CreateInteractionResponseMessage::new().add_embed(
            CreateEmbed::new().title("Here you go").url(PRIVILEGES[uname].0)
        );

        CreateInteractionResponse::Message(message)
    } else {
        let message = CreateInteractionResponseMessage::new().content("Couldn't find the article you were looking for");
        CreateInteractionResponse::Message(message)
    }
}

pub fn register() -> CreateCommand {
    let mut option = CreateCommandOption::new(CommandOptionType::String, "name", "The article name").required(true);
    for (a, (_, c)) in PRIVILEGES.iter() {
        option = option.add_string_choice(*c, *a)
    }
    CreateCommand::new("wiki").description("Link an article from the Purdue Sigbots Wiki or VEX Knowledge Base").add_option(option)
}