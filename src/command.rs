#![allow(clippy::module_name_repetitions)]

use std::time::Duration;

use dptree::case;
use fundu::parse_duration;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Me, Message};
use teloxide::utils::command::{BotCommands, ParseError};

use crate::ignore::{ignore, unignore};
use crate::mute::mute;
use crate::{BotType, HandlerResult, HandlerType};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "ignore entities in this message.")]
    Ignore,
    #[command(description = "unignore entities in this message.")]
    Unignore,
    #[command(description = "mute for a period of time. set to 0 to unmute.", parse_with = human_duration_parser)]
    Mute(Duration),
}

fn human_duration_parser(s: String) -> Result<(Duration,), ParseError> {
    parse_duration(&s)
        .map(|d| (d,))
        .map_err(|e| ParseError::IncorrectFormat(Box::new(e)))
}

pub fn command_handler() -> HandlerType {
    teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Ignore].endpoint(ignore))
        .branch(case![Command::Unignore].endpoint(unignore))
        .branch(case![Command::Mute(dur)].endpoint(mute))
}

async fn help(bot: BotType, me: Me, msg: Message) -> HandlerResult {
    let desc = Command::descriptions();
    bot.send_message(msg.chat.id, desc.username_from_me(&me).to_string())
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}
