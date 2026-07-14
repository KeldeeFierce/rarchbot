use std::{io::Write, sync::Arc};

use crate::{db::DB, rss::Poller};
pub mod db;
pub mod errors;
pub mod models;
pub mod rss;

const TEST_URL: &str = "https://lorem-rss.herokuapp.com/feed?unit=second&interval=10";
const ARCH_URL: &str = "https://archlinux.org/feeds/news/";

#[tokio::main]
async fn main() {
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

    log::info!("bot starting");

    let db = Arc::new(DB::new("sqlite_db/test.db").await.unwrap());
    let poller = Poller::new(TEST_URL, Arc::clone(&db));
    poller.run().await;
}
