pub struct Post {
    pub id: i64,
    pub guid: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
}

pub struct Chat {
    pub id: i64,
    pub chat_id: Option<i64>,
}
