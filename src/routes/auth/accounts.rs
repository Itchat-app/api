use crate::structures::{Base, User};
use crate::utils::error::{Error, Result};
use crate::SMTP_ENABLED;
use argon2::Config;
use nanoid::nanoid;
use rocket::serde::{json::Json, Deserialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone, Copy)]
pub struct RegisterSchema<'r> {
    #[validate(length(min = 3, max = 32))]
    pub username: &'r str,
    #[validate(length(min = 8, max = 32))]
    pub password: &'r str,
    #[validate(email)]
    pub email: &'r str,
}

#[post("/register", data = "<data>")]
pub async fn register(data: Json<RegisterSchema<'_>>) -> Result<Json<User>> {
    let data = data.into_inner();

    data.validate()
        .map_err(|error| Error::InvalidBody { error })?;

    let email_in_use = User::find_one(|q| q.eq("email", &data.email))
        .await
        .is_some();

    if email_in_use {
        return Err(Error::EmailAlreadyInUse);
    }

    let config = Config::default();
    let salt = nanoid!(24);
    let hashed_password = argon2::hash_encoded(
        data.password.to_string().as_bytes(),
        salt.as_bytes(),
        &config,
    )
    .unwrap();

    let mut user = User::new(data.username.into(), data.email.into(), hashed_password);

    if *SMTP_ENABLED {
        todo!("Send email verification")
    } else {
        user.verified = true;
    }

    user.save().await;

    Ok(Json(user))
}

#[get("/verify/<user_id>/<code>")]
pub async fn verify(user_id: i64, code: &str) -> Result<()> {
    let user = User::find_one(|q| q.eq("id", &user_id).eq("verified", false)).await;

    // TODO: Check verification code.

    if let Some(mut user) = user {
        user.verified = true;
        user.save().await;
        Ok(())
    } else {
        Err(Error::AccountNotFound)
    }
}
