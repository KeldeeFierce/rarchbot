use std::str::FromStr;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::models::{Chat, Post};

pub struct DB {
    pool: SqlitePool,
}

impl DB {
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(db_path)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("sqlite_db/migrations")
            .run(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn insert_post(&self, post: &Post) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO posts(guid, title, content) VALUES (?, ?, ?);",
            post.guid,
            post.title,
            post.content
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_posts(&self) -> Result<Vec<Post>, sqlx::Error> {
        let posts: Vec<Post> = sqlx::query_as!(Post, "SELECT id, guid, title, content FROM posts;")
            .fetch_all(&self.pool)
            .await?;
        Ok(posts)
    }

    pub async fn get_last_post(&self) -> Result<Post, sqlx::Error> {
        let post = sqlx::query_as!(
            Post,
            "Select id, guid, title, content FROM posts ORDER BY id DESC LIMIT 1;",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(post)
    }

    pub async fn insert_chat(&self, chat: &Chat) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO chats(id, chat_id) VALUES (?, ?);",
            chat.id,
            chat.chat_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_chats(&self) -> Result<Vec<Chat>, sqlx::Error> {
        let chats: Vec<Chat> = sqlx::query_as!(Chat, "SELECT id, chat_id FROM chats;")
            .fetch_all(&self.pool)
            .await?;
        Ok(chats)
    }

    pub async fn delete_chat(&self, chat: &Chat) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM chats WHERE chat_id=?;", chat.chat_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_post() -> Post {
        Post {
            id: 12,
            guid: Some("test_id_12)".to_owned()),
            title: Some("Title".to_owned()),
            content: Some("Content".to_owned()),
        }
    }

    #[tokio::test]
    async fn test_connect() {
        let db = DB::new("sqlite_db/test.db").await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_migrate() {
        let db = DB::new("sqlite_db/test.db").await.unwrap();
        let res = db.migrate().await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_crud() {
        let db = DB::new("sqlite_db/test.db").await.unwrap();
        let post = mock_post();
        let res = db.insert_post(&post).await;
        assert!(res.is_ok());
        let res = db.get_posts().await;
        assert!(res.is_ok());
    }
}
