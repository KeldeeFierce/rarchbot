use std::sync::Arc;

use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::{db::DB, models::Post};

pub struct Notifier {
    bot: Bot,
    db: Arc<DB>,
    rx: Receiver<Post>,
}

impl Notifier {
    pub fn new(bot: Bot, db: Arc<DB>, rx: Receiver<Post>) -> Self {
        Self { bot, db, rx }
    }

    pub async fn run(mut self, shutdown: CancellationToken) {
        log::info!("Starting notifier");
        loop {
            tokio::select! {
                Some(post) = self.rx.recv() => {
                    self.notify(post).await;
                }

                _ = shutdown.cancelled() => {
                    log::info!("Shutting down notifier");
                    break;
                }
            }
        }

        log::info!("Notifier stopped");
    }

    async fn notify(&self, post: Post) {
        let chats = self.db.get_chats().await;
        let chats = match chats {
            Ok(chats) => chats,
            Err(e) => {
                log::error!("Error getting chats from db: {}", e);
                vec![]
            }
        };
        let text = post.to_string();
        for chat in chats {
            if let Some(chat_id) = chat.chat_id {
                match self.bot.send_message(ChatId(chat_id), &text).await {
                    // Ok(m) => log::info!("Sending a notification {m:?}"),
                    Ok(m) => log::info!("Sending a notification {m:?}"),
                    Err(e) => log::error!("Error senfing a notification {e}"),
                };
            } else {
                log::warn!("Found chat without chat_id in DB, internal id: {}", chat.id)
            }
        }
    }
}
