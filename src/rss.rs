use std::{collections::HashSet, sync::Arc, time::Duration};

use rss::Channel;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{db::DB, errors::RssError, models::Post};

pub struct Poller {
    client: reqwest::Client,
    url: String,
    db: Arc<DB>,
    tx: Sender<Post>,
}

impl Poller {
    pub fn new(url: String, db: Arc<DB>, tx: Sender<Post>) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            url,
            db,
            tx,
        }
    }

    pub async fn poll(&self) -> Result<Vec<Post>, RssError> {
        let contents = self.client.get(&self.url).send().await?.bytes().await?;
        let channel = Channel::read_from(&contents[..])?;

        let mut posts = vec![];
        for item in channel.into_items() {
            let content = item
                .description
                .map(|content| html2text::from_read(content.as_bytes(), 80))
                .transpose()?;

            posts.push(Post {
                id: 0,
                guid: item.guid.map(|guid| guid.value),
                title: item.title,
                // content: Some(content),
                content,
            });
        }

        Ok(posts)
    }

    async fn initialize_seen(&self) -> HashSet<String> {
        let repository_posts = match self.db.get_posts().await {
            Ok(posts) => posts,
            Err(e) => {
                log::error!("Error getting old posts from db: {e}");
                Vec::new()
            }
        };

        let fetched_posts = match self.poll().await {
            Ok(posts) => posts,
            Err(e) => {
                log::error!("Error fetching posts: {e}");
                Vec::new()
            }
        };

        let mut seen: HashSet<String> = HashSet::new();
        for post in repository_posts {
            if let Some(guid) = post.guid
                && !seen.contains(&guid)
            {
                seen.insert(guid);
            }
        }

        for post in fetched_posts {
            if let Some(guid) = &post.guid
                && !seen.contains(guid)
            {
                seen.insert(guid.to_string());
                if let Err(e) = self.db.insert_post(&post).await {
                    log::error!("Error inserting into database: {e}")
                };
            }
        }

        seen
    }

    pub async fn run(self, shutdown: CancellationToken) {
        log::info!("Starting poller");

        let mut seen = self.initialize_seen().await;

        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    log::info!("Shutting down poller");
                    break;
                }
                _ = interval.tick() => {

                    let fetched_posts = match self.poll().await {
                        Ok(posts) => posts,
                        Err(e) => {
                            log::error!("Error fetching posts: {e}");
                            Vec::new()
                        }
                    };

                    for post in fetched_posts {
                        if let Some(guid) = &post.guid
                            && !seen.contains(guid)
                        {
                            seen.insert(guid.to_string());
                            if let Err(e) = self.db.insert_post(&post).await {
                                log::error!("Error inserting into database: {e}")
                            };

                            if let Err(e) = self.tx.send(post).await {
                                log::error!("Error sending post to notifier: {e}")
                            }
                        }
                    }


                }
            }
        }

        log::info!("Poller stopped");
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    const ARCH_URL: &str = "https://archlinux.org/feeds/news/";

    #[tokio::test]
    async fn test_poll() {
        let db = Arc::new(DB::new("sqlite_db/test.db").await.unwrap());
        let (tx, rx) = mpsc::channel::<Post>(32);
        let poller = Poller::new(ARCH_URL.to_string(), Arc::clone(&db), tx);
        let res = poller.poll().await;
        assert!(res.is_ok());
        println!("{}", res.unwrap()[0].content.clone().unwrap());
    }
}
