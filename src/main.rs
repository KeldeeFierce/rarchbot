use std::{env, io::Write, process::exit, sync::Arc};

use teloxide::prelude::*;
use tokio::sync::mpsc::{self};
use tokio_util::sync::CancellationToken;

use crate::{
    db::DB,
    models::Post,
    rss::Poller,
    telegram::{dispatcher, notifier::Notifier},
};
pub mod bot;
pub mod db;
pub mod errors;
pub mod models;
pub mod rss;
pub mod telegram;

// const TEST_URL: &str = "https://lorem-rss.herokuapp.com/feed?unit=second&interval=10";
// const ARCH_URL: &str = "https://archlinux.org/feeds/news/";

#[tokio::main]
async fn main() {
    init_logger();

    dotenvy::dotenv().ok();
    if env::var("TELOXIDE_TOKEN").is_err() {
        log::error!("missing TELOXIDE_TOKEN env var");
        exit(1);
    }

    let url = env::var("URL").unwrap_or_else(|_| {
        log::error!("missing URL env var");
        exit(1);
    });

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| {
        log::error!("missing DB_PATH env var");
        exit(1);
    });

    log::info!("bot starting");
    let bot = Bot::from_env();
    let shutdown = CancellationToken::new();
    let poller_shutdown = shutdown.clone();
    let notifier_shutdown = shutdown.clone();

    let (tx, rx) = mpsc::channel::<Post>(32);

    let db = Arc::new(DB::new(&db_path).await.unwrap_or_else(|e| {
        log::error!("Error opening db: {}", e);
        exit(1);
    }));

    let poller = Poller::new(url.to_string(), Arc::clone(&db), tx);
    let notifier = Notifier::new(bot.clone(), Arc::clone(&db), rx);

    if let Err(e) = db.migrate().await {
        log::error!("Unable to migrate: {}", e);
        exit(1);
    }

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");

        log::info!("Ctrl+C received, shutting down...");
        shutdown.cancel();
    });

    let notifier_handle = tokio::spawn(notifier.run(notifier_shutdown));
    let poller_handle = tokio::spawn(poller.run(poller_shutdown));

    dispatcher::run(bot.clone(), Arc::clone(&db)).await;

    // let _ = tokio::try_join!(notifier_handle, poller_handle,);
    let _ = notifier_handle.await;
    let _ = poller_handle.await;
}

fn init_logger() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                buf.timestamp_seconds(),
                record.level(),
                record.args()
            )
        })
        .init();
}
