use std::env;
use std::error::Error;

use sqlx::PgPool;
use teloxide::adaptors::throttle::Limits;
use teloxide::adaptors::{CacheMe, Throttle};
use teloxide::dispatching::{Dispatcher, UpdateFilterExt, UpdateHandler};
use teloxide::prelude::Update;
use teloxide::requests::RequesterExt;
use teloxide::Bot;

use crate::command::command_handler;
use crate::dedup::dedup;

mod command;
mod dedup;
mod ignore;
mod msg_ext;
mod mute;
mod utils;

// mod bot_service;

type BoxedError = Box<dyn Error + Send + Sync>;
type HandlerResult = Result<(), BoxedError>;
type BotType = CacheMe<Throttle<Bot>>;
type HandlerType = UpdateHandler<BoxedError>;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let db = PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL is set"))
        .await
        .expect("db connect");

    sqlx::migrate!().run(&db).await.expect("db migrate");

    let bot = Bot::from_env().throttle(Limits::default()).cache_me();

    let mut dp = Dispatcher::builder(
        bot.clone(),
        Update::filter_message()
            .branch(command_handler())
            .branch(dptree::endpoint(dedup)),
    )
    .dependencies(dptree::deps![db])
    .enable_ctrlc_handler()
    .build();

    dp.dispatch().await;
}
