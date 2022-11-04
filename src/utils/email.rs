use crate::config::*;
use crate::database::redis::*;
use crate::structures::User;
use crate::utils::Snowflake;
use lazy_regex::regex;
use ormlite::model::*;
use serde_json::json;
use sqlx::types::Uuid;

const THREE_HOURS_IN_SECONDS: i64 = 10800;

#[derive(Model, FromRow)]
#[ormlite(table = "account_invites")]
pub struct AccountInvite {
    #[ormlite(primary_key)]
    pub code: Uuid,
    pub used: bool,
    pub taken_by: Option<Snowflake>,
}

pub fn normalize(email: String) -> Option<String> {
    let split = regex!("([^@]+)(@.+)").captures(&email)?;
    let mut clean = regex!("\\+.+|\\.")
        .replace_all(split.get(1)?.as_str(), "")
        .to_string();
    clean.push_str(split.get(2)?.as_str());

    Some(clean.to_lowercase())
}

pub async fn send(user: &User) -> bool {
    let mut content = include_str!("../../assets/templates/verify.html").to_string();
    let code = Uuid::new_v4();

    content = content
        .replace("%%EMAIL%%", &*user.email)
        .replace("%%CODE%%", &code.to_string())
        .replace("%%USER_ID%%", &user.id.to_string());

    let body = json!({
        "subject": "Verify your ItChat account",
        "sender": { "email": "noreply@itchat.world" },
        "to": [{ "email": user.email }],
        "type": "classic",
        "htmlContent": content,
    });

    let res = reqwest::Client::new()
        .post("https://api.sendinblue.com/v3/smtp/email")
        .header("api-key", (*SENDINBLUE_API_KEY).clone())
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body.to_string())
        .send()
        .await;

    if res.map(|r| r.status().is_success()).unwrap_or(false) {
        REDIS
            .set::<(), _, _>(
                *user.id,
                code.to_string(),
                Expiration::EX(THREE_HOURS_IN_SECONDS).into(),
                None,
                false,
            )
            .await
            .is_ok()
    } else {
        false
    }
}

pub async fn verify(user_id: Snowflake, code: Uuid) -> bool {
    match REDIS.get::<String, _>(user_id.to_string()).await {
        Ok(token) if code.to_string() == token => {
            REDIS.del::<u32, _>(user_id.to_string()).await.ok();
            true
        }
        _ => false,
    }
}
