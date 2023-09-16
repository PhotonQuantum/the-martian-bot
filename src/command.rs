#![allow(clippy::module_name_repetitions)]

use dptree::case;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Me, Message};
use teloxide::utils::command::BotCommands;

use crate::ignore::{ignore, unignore};
use crate::{BotType, HandlerResult, HandlerType};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "ignore entities in this message.")]
    Ignore,
    #[command(description = "unignore entities in this message.")]
    Unignore,
}

pub fn command_handler() -> HandlerType {
    teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Ignore].endpoint(ignore))
        .branch(case![Command::Unignore].endpoint(unignore))
}

async fn help(bot: BotType, me: Me, msg: Message) -> HandlerResult {
    let desc = Command::descriptions();
    bot.send_message(msg.chat.id, desc.username_from_me(&me).to_string())
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}
