use rocket::State;

use sqlx::{PgPool};

use crate::error::VortoResult;

use crate::responses::{VocView};


use crate::services::*;

#[get("/vocs")]
pub async fn get_vocs(pool: &State<PgPool>) -> VortoResult<Vec<VocView>> {
    voc_service::get_vocs(pool.inner()).await
}