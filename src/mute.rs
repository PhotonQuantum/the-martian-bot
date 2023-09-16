use std::time::Duration;

use sqlx::postgres::types::PgInterval;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::Message;

use crate::{BotType, BoxedError, HandlerResult};

pub async fn mute(bot: BotType, msg: Message, dur: Duration, db: PgPool) -> HandlerResult {
    let interval = match PgInterval::try_from(dur) {
        Ok(i) => i,
        Err(e) => {
            bot.send_message(msg.chat.id, e.to_string())
                .reply_to_message_id(msg.id)
                .await?;
            return Ok(());
        }
    };

    let until = set_mute(msg.chat.id.0, interval, db).await?;
    let respond = if dur.is_zero() {
        "unmuted.".to_string()
    } else {
        format!("muted until {}.", until)
    };
    bot.send_message(msg.chat.id, respond)
        .reply_to_message_id(msg.id)
        .await?;

    Ok(())
}

async fn set_mute(
    chat_id: i64,
    interval: PgInterval,
    db: PgPool,
) -> Result<OffsetDateTime, BoxedError> {
    let result = sqlx::query_file_scalar!("sql/mute_for_chat.sql", chat_id, interval)
        .fetch_one(&db)
        .await?;
    Ok(result)
}
