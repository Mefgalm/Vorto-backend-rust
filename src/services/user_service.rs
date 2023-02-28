use super::password_hasher::PwdHasher;
use crate::domain::user::User;
use crate::error::{VortoError, VortoErrorCode};
use crate::services::jwt_service::generate_token;
use crate::{error::VortoResult};
use chrono::Utc;
use serde::Serialize;
use sqlx::{query_as, PgPool};

#[derive(Serialize)]
pub struct LoginResponse {
    email: String,
    token: String,
}

fn invalid_login_or_error() -> VortoError {
    VortoError::new(
        VortoErrorCode::InvalidLoginOrPassword,
        String::from("Invalid login or password"),
    )
}

async fn get_by_email(pool: &PgPool, email: &str) -> VortoResult<User> {
    let user = query_as!(
        User,
        "SELECT *
        FROM users
        WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await?;

    match user {
        Some(u) => VortoResult::Ok(u),
        None => VortoResult::Err(invalid_login_or_error()),
    }
}

fn verify_password(pwd_hasher: &PwdHasher, password: &str, password_hash: &str) -> VortoResult<()> {
    if pwd_hasher.verify(password, &password_hash)? {
        VortoResult::Ok(())
    } else {
        VortoResult::Err(invalid_login_or_error())
    }
}

pub async fn login(
    pwd_hasher: &PwdHasher,
    email: &str,
    password: &str,
    pool: &PgPool
) -> VortoResult<LoginResponse> {
    let user = get_by_email(&pool, email).await?;
    verify_password(pwd_hasher, password, &user.password_hash)?;
    let token = generate_token(Utc::now())?;

    VortoResult::Ok(LoginResponse {
        email: email.to_owned(),
        token
    })
}
