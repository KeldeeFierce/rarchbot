use std::fmt::Display;

#[derive(Debug)]
pub struct Post {
    pub id: i64,
    pub guid: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug)]
pub struct Chat {
    pub id: i64,
    pub chat_id: Option<i64>,
}

impl Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}",
            self.title.clone().unwrap_or_default(),
            self.content.clone().unwrap_or_default()
        )
    }
}
