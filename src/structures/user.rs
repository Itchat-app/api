use super::{Base, Bot, Server, Session};
use crate::database::DB as db;
use crate::utils::snowflake::generate_id;
use serde::{Deserialize, Serialize};

#[crud_table(table_name:users)]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub avatar: Option<String>,
    pub password: String,
    pub email: String,
    pub badges: i64,
    pub verified: bool,
}

impl Base for User {}

impl User {
    pub fn new(username: String, email: String, password: String) -> Self {
        Self {
            id: generate_id(),
            username,
            email,
            password,
            verified: false,
            ..Default::default()
        }
    }

    pub async fn fetch_sessions(&self) -> Vec<Session> {
        Session::find(|q| q.eq("user_id", &self.id)).await
    }

    pub async fn fetch_servers(&self) -> Vec<Server> {
        db.fetch("SELECT * FROM servers WHERE owner_id = $1 OR id IN ( SELECT server_id FROM members WHERE id = $1 )", vec![self.id.into()]).await.unwrap()
    }

    pub async fn fetch_bots(&self) -> Vec<Bot> {
        Bot::find(|q| q.eq("owner_id", &self.id)).await
    }

    // pub async fn fetch_relations(&self) {}

    #[sql(crate::database::DB, "SELECT * FROM users LEFT JOIN sessions ON sessions.user_id = users.id WHERE verified = TRUE AND sessions.token = $1")]
    pub async fn fetch_by_token(_token: &str) -> User {}

    pub fn to_public(&self) -> Self {
        Self {
            id: self.id,
            username: self.username.clone(),
            avatar: self.avatar.clone(),
            badges: self.badges,
            ..Default::default()
        }
    }
}
