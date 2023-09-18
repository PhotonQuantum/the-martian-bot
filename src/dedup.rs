#![allow(clippy::module_name_repetitions)]

use sqlx::{Acquire, PgPool, Postgres};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Message;
use teloxide::requests::Requester;
use teloxide::types::MessageId;
use tracing::debug;

use crate::msg_ext::MessageExt;
use crate::utils::{clean_url, hash_img};
use crate::{BotType, BoxedError, HandlerResult};

struct Duplicate {
    pub msg_id: i32,
    pub duplicate_cnt: i32,
}

pub async fn dedup(bot: BotType, msg: Message, db: PgPool) -> HandlerResult {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    let (link, forward, img) = tokio::try_join!(
        async {
            let db = db.clone();
            dedup_links(&msg, &db).await
        },
        async {
            let db = db.clone();
            dedup_forward(&msg, &db).await
        },
        { dedup_img(&bot, &msg, &db) },
    )?;

    let muted = sqlx::query_file!("sql/query_muted.sql", chat_id)
        .fetch_optional(&db)
        .await?;
    if muted.is_some() {
        debug!(chat_id, message_id, "muted");
        return Ok(());
    }

    let seen_before = link.or(forward).or(img);

    let Some(Duplicate {
        msg_id: seen_msg_id,
        duplicate_cnt: dup_cnt,
    }) = seen_before
    else {
        return Ok(());
    };

    debug!(chat_id, message_id, seen_msg_id, "seen before");
    let msg_link = Message::url_of(msg.chat.id, msg.chat.username(), MessageId(seen_msg_id));
    let body = msg_link.map_or_else(
        || make_reply_msg(dup_cnt),
        |link| make_reply_msg_with_link(dup_cnt, link),
    );
    bot.send_message(msg.chat.id, body)
        .reply_to_message_id(msg.id)
        .await?;

    Ok(())
}

async fn dedup_links(
    msg: &Message,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<Option<Duplicate>, BoxedError> {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    let mut links: Vec<_> = msg.links().into_iter().map(clean_url).collect();
    links.sort_unstable();
    links.dedup();

    if links.is_empty() {
        return Ok(None);
    }

    let mut txn = db.begin().await?;
    let mut seen_before = None;
    for link in links {
        debug!(chat_id, message_id, url=%link, "link found");
        let record = sqlx::query_file!("sql/insert_url.sql", link.to_string(), chat_id, message_id)
            .fetch_one(&mut *txn)
            .await?;
        if !record.ignore && record.message_id != message_id {
            let dup = Duplicate {
                msg_id: record.message_id,
                duplicate_cnt: record.duplicate_cnt,
            };
            seen_before.get_or_insert(dup);
        }
    }
    txn.commit().await?;
    Ok(seen_before)
}

async fn dedup_forward(
    msg: &Message,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<Option<Duplicate>, BoxedError> {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    if let Some((forward_channel_id, forward_message_id)) = msg
        .forward_from_message_id()
        .and_then(|msg_id| msg.forward_from_chat().map(|chat| (chat.id.0, msg_id)))
    {
        debug!(
            chat_id,
            message_id, forward_channel_id, forward_message_id, "forward found"
        );
        let mut conn = db.acquire().await?;
        let record = sqlx::query_file!(
            "sql/insert_forward.sql",
            forward_channel_id,
            forward_message_id,
            chat_id,
            message_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !record.ignore && record.message_id != message_id {
            let dup = Duplicate {
                msg_id: record.message_id,
                duplicate_cnt: record.duplicate_cnt,
            };
            return Ok(Some(dup));
        }
    }

    Ok(None)
}

async fn dedup_img(
    bot: &BotType,
    msg: &Message,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<Option<Duplicate>, BoxedError> {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    let Some(photo) = msg.image(bot).await? else {
        return Ok(None);
    };

    let hash = tokio::task::spawn_blocking(move || hash_img(&photo)).await?;
    debug!(chat_id, message_id, hash, "image found");

    let mut conn = db.acquire().await?;
    let record = sqlx::query_file!("sql/sim_img.sql", hash, chat_id)
        .fetch_optional(&mut *conn)
        .await?;

    if let Some(record) = &record {
        if !record.ignore {
            let dup = Duplicate {
                msg_id: record.message_id,
                duplicate_cnt: record.duplicate_cnt,
            };
            return Ok(Some(dup));
        }
    }

    if record.map_or(true, |record| record.dist != 0) {
        // A different image, need to insert the entity
        sqlx::query_file!("sql/insert_img.sql", hash, chat_id, message_id)
            .execute(&mut *conn)
            .await?;
    }

    Ok(None)
}

fn make_reply(seen_times: i32) -> String {
    match seen_times {
        0 => unreachable!("seen_times should not be 0"),
        1..=2 => "看过了".to_string(),
        3..=5 => format!("看过 {} 次啦", seen_times),
        _ => format!("看过 {} 次啦！不要再发啦", seen_times),
    }
}

fn make_reply_msg(times: i32) -> String {
    format!("{}!", make_reply(times))
}
fn make_reply_msg_with_link(seen_times: i32, link: impl ToString) -> String {
    format!("{}: {}", make_reply(seen_times), link.to_string())
}
