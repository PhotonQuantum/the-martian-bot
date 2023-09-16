use std::env;
use std::error::Error;

use sqlx::PgPool;
use teloxide::adaptors::throttle::Limits;
use teloxide::adaptors::Throttle;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Requester, RequesterExt};
use teloxide::types::{Message, MessageId, Update};
use teloxide::Bot;
use tracing::debug;

use dedup::{dedup_forward, dedup_img, dedup_links};

mod dedup;
mod msg_ext;

// mod bot_service;

type BoxedError = Box<dyn Error + Send + Sync>;
type HandlerResult = Result<(), BoxedError>;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let db = PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL is set"))
        .await
        .expect("db connect");

    sqlx::migrate!().run(&db).await.expect("db migrate");

    let bot = Bot::from_env().throttle(Limits::default());

    let mut dp = Dispatcher::builder(bot.clone(), Update::filter_message().endpoint(dedup))
        .dependencies(dptree::deps![db])
        .enable_ctrlc_handler()
        .build();

    dp.dispatch().await;
}

async fn dedup(bot: Throttle<Bot>, msg: Message, db: PgPool) -> HandlerResult {
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

    let seen_before = link.or(forward).or(img);

    if let Some(seen_msg_id) = seen_before {
        debug!(chat_id, message_id, seen_msg_id, "seen before");
        let msg_link = Message::url_of(msg.chat.id, msg.chat.username(), MessageId(seen_msg_id));
        let body = msg_link.map_or_else(|| "看过了".to_string(), |link| format!("看过了: {link}"));
        bot.send_message(msg.chat.id, body)
            .reply_to_message_id(msg.id)
            .await?;
    }

    Ok(())
}
