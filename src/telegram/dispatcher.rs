use std::sync::Arc;

use teloxide::dispatching::dialogue::GetChatId;
use teloxide::dptree::deps;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::db::DB;
use crate::models::Chat;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "A bot that sends you updates from arlinux.org, so you won't miss the next attack on AUR"
)]
enum Command {
    #[command(description = "Display this text.")]
    Help,
    #[command(description = "Subscribe to updates")]
    Start,
    #[command(description = "Unsubscribe from updates")]
    Stop,
    #[command(description = "Get the latest post awailable")]
    Last,
    #[command(description = "Check subscribtion status")]
    IsSubscribed,
}

pub async fn run(bot: Bot, db: Arc<DB>) {
    let handler = Update::filter_message()
        .filter_command::<Command>()
        .branch(dptree::case![Command::Help].endpoint(help))
        .branch(dptree::case![Command::Start].endpoint(start))
        .branch(dptree::case![Command::Stop].endpoint(stop))
        .branch(dptree::case![Command::Last].endpoint(last))
        .branch(dptree::case![Command::IsSubscribed].endpoint(is_subscribed));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .dependencies(deps![db])
        .build()
        .dispatch()
        .await;
    log::info!("Dispatcher stopped")
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    let text = Command::descriptions().to_string();
    log::info!("sending help, chat: {}, message: {}", msg.chat.id, text);
    bot.send_message(msg.chat.id, &text).await?;
    Ok(())
}

async fn start(bot: Bot, msg: Message, db: Arc<DB>) -> HandlerResult {
    let chat_id = msg.chat.id.0;
    let chat = Chat {
        id: 0,
        chat_id: Some(chat_id),
    };

    match db.insert_chat(&chat).await {
        Ok(()) => {
            log::info!("Chat id {:?} just subscribed", chat_id);
            bot.send_message(msg.chat.id, "You are now subscribed!".to_owned())
                .await?;
        }
        Err(e) => {
            log::error!("Error subscribing in chat {}: {}", chat_id, e);
            bot.send_message(
                msg.chat.id,
                "Something went wrong, you are not subsribed".to_owned(),
            )
            .await?;
        }
    }

    Ok(())
}

async fn stop(bot: Bot, msg: Message, db: Arc<DB>) -> HandlerResult {
    let chat_id = msg.chat.id.0;
    let chat = Chat {
        id: 0,
        chat_id: Some(chat_id),
    };

    match db.delete_chat(&chat).await {
        Ok(()) => {
            log::info!("Chat id {} just unsubscribed", chat_id);
            bot.send_message(msg.chat.id, "You are now unsubscribed!".to_owned())
                .await?;
        }
        Err(e) => {
            log::error!("Error unsubscribing in chat {}: {}", chat_id, e);
            bot.send_message(
                msg.chat.id,
                "Something went wrong, you are not unsubsribed".to_owned(),
            )
            .await?;
        }
    }

    Ok(())
}

async fn last(bot: Bot, msg: Message, db: Arc<DB>) -> HandlerResult {
    let post = db.get_last_post().await;
    match post {
        Ok(post) => {
            bot.send_message(msg.chat.id, post.to_string()).await?;
            log::info!("Sending last post: {}", post);
        }
        Err(e) => {
            bot.send_message(msg.chat.id, "Unable to find any recent posts".to_owned())
                .await?;
            log::error!("Failed to get last post from DB: {e}");
        }
    }

    Ok(())
}

async fn is_subscribed(bot: Bot, msg: Message, db: Arc<DB>) -> HandlerResult {
    let chats = db.get_chats().await.unwrap_or_else(|e| {
        log::error!("Unable to fetch chats from DB: {e}");
        Vec::new()
    });

    for chat in chats {
        if let Some(chat_id) = chat.chat_id
            && chat_id == msg.chat.id.0
        {
            bot.send_message(msg.chat.id, "You are subsribed").await?;
            log::info!(
                "Chat id {} requsted subscribtion status, status is SUBSCRIBED",
                msg.chat.id
            );
            return Ok(());
        }
    }

    bot.send_message(msg.chat.id, "You are not subsribed")
        .await?;
    log::info!(
        "Chat id {} requsted subscribtion status, status is UNSUBSCRIBED",
        msg.chat.id
    );

    Ok(())
}
