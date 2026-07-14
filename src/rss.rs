use std::{collections::HashSet, sync::Arc, time::Duration};

use rss::Channel;

use crate::{db::DB, errors::RssError, models::Post};

pub struct Poller {
    client: reqwest::Client,
    url: String,
    db: Arc<DB>,
}

impl Poller {
    pub fn new(url: &str, db: Arc<DB>) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            url: url.to_owned(),
            db,
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

    pub async fn run(&self) {
        // Initialize lookup table
        let mut seen = self.initialize_seen().await;

        let mut interval = tokio::time::interval(Duration::from_secs(10));

        // Runtime with initialized lookup table
        loop {
            interval.tick().await;

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

                    println!("{}\n{}", post.title.unwrap(), post.content.unwrap())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // const TEST_URL: &str = "https://lorem-rss.herokuapp.com/feed?unit=second&interval=10";
    const ARCH_URL: &str = "https://archlinux.org/feeds/news/";

    #[tokio::test]
    async fn test_poll() {
        let db = Arc::new(DB::new("sqlite_db/test.db").await.unwrap());
        let poller = Poller::new(ARCH_URL, Arc::clone(&db));
        let res = poller.poll().await;
        assert!(res.is_ok());
        println!("{}", res.unwrap()[0].content.clone().unwrap());
    }
}
