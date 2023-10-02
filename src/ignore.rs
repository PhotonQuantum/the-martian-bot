use sqlx::{Acquire, PgPool, Postgres};
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::Message;

use crate::msg_ext::MessageExt;
use crate::utils::{clean_url, hash_img};
use crate::{BotType, BoxedError, HandlerResult};

pub async fn ignore(bot: BotType, msg: Message, db: PgPool) -> HandlerResult {
    ignore_(bot, msg, true, db).await
}

pub async fn unignore(bot: BotType, msg: Message, db: PgPool) -> HandlerResult {
    ignore_(bot, msg, false, db).await
}

#[allow(clippy::unused_async)]
pub async fn nop() -> HandlerResult {
    Ok(())
}

async fn ignore_(bot: BotType, msg: Message, ignore: bool, db: PgPool) -> HandlerResult {
    let respond = if let Some(target) = msg.reply_to_message() {
        let (link, forward, img) = tokio::try_join!(
            async {
                let db = db.clone();
                set_ignore_links(target, ignore, &db).await
            },
            async {
                let db = db.clone();
                set_ignore_forward(target, ignore, &db).await
            },
            { set_ignore_img(&bot, target, ignore, &db) },
        )?;
        if ignore {
            format!("{} entities ignored.", link + forward + img)
        } else {
            format!("{} entities unignored.", link + forward + img)
        }
    } else if ignore {
        "You must reply to a message to ignore it.".to_string()
    } else {
        "You must reply to a message to unignore it.".to_string()
    };
    bot.send_message(msg.chat.id, respond)
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}

async fn set_ignore_links(
    msg: &Message,
    ignore: bool,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<u64, BoxedError> {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    let mut links: Vec<_> = msg.links().into_iter().map(clean_url).collect();
    links.sort_unstable();
    links.dedup();

    if links.is_empty() {
        return Ok(0);
    }

    let mut affected = 0;
    let mut txn = db.begin().await?;
    for link in links {
        let result = sqlx::query_file!(
            "sql/set_ignore_url.sql",
            link.to_string(),
            chat_id,
            message_id,
            ignore
        )
        .execute(&mut *txn)
        .await?;
        affected += result.rows_affected();
    }
    txn.commit().await?;
    Ok(affected)
}

async fn set_ignore_forward(
    msg: &Message,
    ignore: bool,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<u64, BoxedError> {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    if let Some((forward_channel_id, forward_message_id)) = msg
        .forward_from_message_id()
        .and_then(|msg_id| msg.forward_from_chat().map(|chat| (chat.id.0, msg_id)))
    {
        let mut conn = db.acquire().await?;
        let result = sqlx::query_file!(
            "sql/set_ignore_forward.sql",
            forward_channel_id,
            forward_message_id,
            chat_id,
            message_id,
            ignore
        )
        .execute(&mut *conn)
        .await?;
        return Ok(result.rows_affected());
    }

    Ok(0)
}

async fn set_ignore_img(
    bot: &BotType,
    msg: &Message,
    ignore: bool,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<u64, BoxedError> {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    if let Some(photo) = msg.image(bot).await? {
        let hash = tokio::task::spawn_blocking(move || hash_img(&photo)).await?;

        let mut conn = db.acquire().await?;
        let result = sqlx::query_file!("sql/set_ignore_img.sql", hash, chat_id, message_id, ignore)
            .execute(&mut *conn)
            .await?;

        return Ok(result.rows_affected());
    }

    Ok(0)
}
