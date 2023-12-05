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
use std::collections::HashMap;

lazy_static! {
static ref PRIVILEGES : HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("main", "https://wiki.purduesigbots.com/");
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
        return CreateInteractionResponse::Message(message)
    }
    let uname = name.unwrap().trim();

    println!("{}", uname);
    if PRIVILEGES.contains_key(uname) {
        let message = CreateInteractionResponseMessage::new().add_embed(
            CreateEmbed::new().title("Here you go").url(PRIVILEGES[uname].to_string())
        );

        CreateInteractionResponse::Message(message)
    } else {
        let message = CreateInteractionResponseMessage::new().content("Couldn't find the article you were looking for");
        CreateInteractionResponse::Message(message)
    }
}

pub fn register() -> CreateCommand {
    let mut option = CreateCommandOption::new(CommandOptionType::String, "name", "The article name").required(true).set_autocomplete(true);
    for (a, _) in PRIVILEGES.iter() {
        option = option.add_string_choice(a.to_string(), a.to_string())
    }
    CreateCommand::new("wiki").description("Send a wiki article").add_option(option)
}