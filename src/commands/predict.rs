use serenity::all::{
    CommandDataOptionValue, CommandOptionType
};
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateEmbed,
    CreateInteractionResponseMessage, CreateEmbedAuthor,
    CreateEmbedFooter,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::model::Color;

use crate::api::vrc_data_analysis::VRCDataAnalysis;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct PredictCommand;

impl PredictCommand {
    pub fn command() -> CreateCommand {
        CreateCommand::new("predict")
			.description("Predict the outcome of a VRC match. Use \"AVG\" to represent an average team.")
			.add_option(
				CreateCommandOption::new(CommandOptionType::String, "r1", "Red Alliance Partner 1")
                    .required(true)
			)
            .add_option(
				CreateCommandOption::new(CommandOptionType::String, "r2", "Red Alliance Partner 2")
                    .required(true)
			)
            .add_option(
				CreateCommandOption::new(CommandOptionType::String, "b1", "Blue Alliance Partner 1")
                    .required(true)
			)
            .add_option(
				CreateCommandOption::new(CommandOptionType::String, "b2", "Blue Alliance Partner 2")
                    .required(true)
			)
    }

    fn progress_bar(length: usize, progress: f64) -> String {
        let red_dot_count = (length as f64 * (progress / 100.0)).round() as usize;

        "🟥".repeat(red_dot_count) + "🟦".repeat(length - red_dot_count).as_ref()
    }

    pub async fn response(
        &self,
        _ctx: &Context,
        interaction: &CommandInteraction,
        vrc_data_analysis: &VRCDataAnalysis
    ) -> CreateInteractionResponseMessage {
        let mut teams = Vec::new();
        for (idx, opt) in interaction.data.options.iter().enumerate() {
            if let CommandDataOptionValue::String(number) = &opt.value {
                teams.push(number);
            } else {
                return CreateInteractionResponseMessage::new()
                    .add_embed(CreateEmbed::new().title(format!("Invalid team number at argument {}", idx + 1)));
            }
        }

        let [r1, r2, b1, b2] = teams.as_slice() else {
            return CreateInteractionResponseMessage::new().add_embed(CreateEmbed::new().title("Missing team argument."));
        };

        let embed = match vrc_data_analysis.predict_match((r1, r2), (b1, b2)).await {
            Ok(results) => CreateEmbed::new()
                .author(CreateEmbedAuthor::new("Match Prediction Results"))
                .title(format!("{} {} (🔴) vs {} {} (🔵)", results.red1, results.red2, results.blue1, results.blue2))
                    .url("https://www.vrc-data-analysis.com/")
                .description(format!("{}\n\n{}", results.prediction_msg, Self::progress_bar(17, results.red_win_probability)))
                .footer(
                    CreateEmbedFooter::new("Match predictions provided by vrc-data-analysis.com")
                        .icon_url("https://cdn.discordapp.com/attachments/1181718273017004043/1185320302272585758/favicon-3.png")
                )
                .color(if results.red_win_probability > 50.0 {
                    Color::from_rgb(210, 38, 48)
                } else {
                    Color::from_rgb(0, 119, 200)
                }),
            Err(err) => CreateEmbed::new()
                    .title("Failed to fetch match prediction data from vrc-data-analysis.")
                    .description(format!("```rs\n{err:?}```")),
        };

        CreateInteractionResponseMessage::new().add_embed(embed)
    }
}