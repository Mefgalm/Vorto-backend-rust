use rocket::State;
use rocket::serde::json::Json;
use sqlx::{PgPool};

use crate::error::VortoResult;
use crate::requests::{LoginRequest};

use crate::services::password_hasher::PwdHasher;
use crate::services::user_service::LoginResponse;
use crate::services::*;


#[post("/users/sign_in", data = "<req>")]
pub async fn sign_in(
    pool: &State<PgPool>,
    pwd_hasher: &State<PwdHasher>,
    req: Json<LoginRequest>,
) -> VortoResult<LoginResponse> {
    user_service::login(pwd_hasher, &req.email, &req.password, pool).await
}
