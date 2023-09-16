#![allow(clippy::module_name_repetitions)]

use image_hasher::{HashAlg, Hasher, HasherConfig};
use once_cell::sync::Lazy;
use sqlx::{Acquire, Postgres};
use teloxide::adaptors::Throttle;
use teloxide::prelude::Message;
use teloxide::Bot;
use tracing::debug;

use crate::msg_ext::MessageExt;
use crate::BoxedError;

pub async fn dedup_links(
    msg: &Message,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<Option<i32>, BoxedError> {
    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    let mut links = msg.links();
    for link in &mut links {
        // Try to remove tracking parameters
        link.query_pairs_mut().clear();
    }
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
        if record.message_id != message_id {
            seen_before.get_or_insert(record.message_id);
        }
    }
    txn.commit().await?;
    Ok(seen_before)
}

pub async fn dedup_forward(
    msg: &Message,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<Option<i32>, BoxedError> {
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
        if record.message_id != message_id {
            return Ok(Some(record.message_id));
        }
    }

    Ok(None)
}

pub async fn dedup_img(
    bot: &Throttle<Bot>,
    msg: &Message,
    db: impl Acquire<'_, Database = Postgres> + Send,
) -> Result<Option<i32>, BoxedError> {
    static IMG_HASHER: Lazy<Hasher> = Lazy::new(|| {
        HasherConfig::new()
            .hash_alg(HashAlg::DoubleGradient)
            .preproc_dct()
            .to_hasher()
    });

    let chat_id = msg.chat.id.0;
    let message_id = msg.id.0;

    if let Some(photo) = msg.image(bot).await? {
        let hash = {
            let img_hash =
                tokio::task::spawn_blocking(move || IMG_HASHER.hash_image(&photo)).await?;
            let mut buf = [0u8; 8];
            buf[..5].copy_from_slice(img_hash.as_bytes());
            i64::from_be_bytes(buf)
        };
        debug!(chat_id, message_id, hash, "image found");

        let mut conn = db.acquire().await?;
        let record = sqlx::query_file!("sql/sim_img.sql", hash, chat_id)
            .fetch_optional(&mut *conn)
            .await?;

        if let Some(record) = &record {
            return Ok(Some(record.message_id));
        }

        if record.map_or(true, |record| record.dist != 0) {
            // A different image, need to insert the entity
            sqlx::query_file!("sql/insert_img.sql", hash, chat_id, message_id)
                .execute(&mut *conn)
                .await?;
        }
    }

    Ok(None)
}
