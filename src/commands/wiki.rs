use std::collections::HashMap;
use std::sync::LazyLock;

use serenity::all::{
    CommandDataOptionValue, CommandOptionType
};
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateEmbed,
    CreateInteractionResponseMessage
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct WikiCommand;

static PRIVILEGES: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert("main", ("https://wiki.purduesigbots.com/", "Sigbots: Main Page"));
    map.insert("building", ("https://wiki.purduesigbots.com/hardware/misc.-vex-parts", "Sigbots: Useful Building Techniques"));
    map.insert("structure", ("https://wiki.purduesigbots.com/hardware/misc.-vex-parts-1/structure", "Sigbots: Structural Parts"));
    map.insert("motion", ("https://wiki.purduesigbots.com/hardware/misc.-vex-parts-1/motion", "Sigbots: Motion parts"));
    map.insert("joints", ("https://wiki.purduesigbots.com/hardware/vex-joints", "Sigbots: Joints"));
    map.insert("drives", ("https://wiki.purduesigbots.com/hardware/vex-drivetrains", "Sigbots: Drivetrains"));
    map.insert("lifts", ("https://wiki.purduesigbots.com/hardware/lifts", "Sigbots: Lift Mechanisms"));
    map.insert("intakes", ("https://wiki.purduesigbots.com/hardware/intakes", "Sigbots: Intake Mechanisms"));
    map.insert("launchers", ("https://wiki.purduesigbots.com/hardware/shooting-mechanisms", "Sigbots: Launching Mechanisms"));
    map.insert("pneumatics", ("https://wiki.purduesigbots.com/hardware/pneumatics", "Sigbots: Pneumatics"));

    map
});

impl WikiCommand {
    pub fn command() -> CreateCommand {
        let mut option = CreateCommandOption::new(CommandOptionType::String, "name", "The article name").required(true);
        for (key, (url, title)) in PRIVILEGES.iter() {
            option = option.add_string_choice(*title, *key).description(*url);
        }
        CreateCommand::new("wiki").description("Link an article from the Purdue Sigbots Wiki or VEX Knowledge Base").add_option(option)
    }

    pub fn response(
        &self,
        _ctx: &Context,
        interaction: &CommandInteraction
    ) -> CreateInteractionResponseMessage {
        let name = if let CommandDataOptionValue::String(arg) = &interaction.data.options[0].value {
            arg.trim()
        } else {
            return CreateInteractionResponseMessage::new().content("No argument provided");
        };
    
        if PRIVILEGES.contains_key(name) {
            CreateInteractionResponseMessage::new().add_embed(
                CreateEmbed::new().title("Here you go").url(PRIVILEGES[name].0)
            )
        } else {
            CreateInteractionResponseMessage::new().content("Couldn't find the article you were looking for")
        }
    }
}