use rocket::serde::json::Json;
use rocket::State;
use sqlx::PgPool;

use crate::auth::Admin;
use crate::error::VortoResult;
use crate::requests::{LoadDefinitionRequest, SearchRequest, UpdateWordRequest};
use crate::responses::{WordStats, WordView};
use crate::services::*;

#[post("/words/search", data = "<req>")]
pub async fn search(
    req: Json<SearchRequest>,
    _admin: Admin,
    pool: &State<PgPool>,
) -> VortoResult<Vec<WordView>> {
    word_service::search(
        &req.text,
        &req.statuses,
        &req.load_statuses,
        &req.difficulties,
        &req.field_order,
        req.skip,
        req.take,
        pool.inner(),
    )
    .await
}

#[put("/words/load_definitions", data = "<req>")]
pub async fn load_definitions(
    req: Json<LoadDefinitionRequest>,
    _admin: Admin,
    pool: &State<PgPool>,
) -> VortoResult<()> {
    word_service::load_definitions(req.id, req.timestamp, pool).await
}

#[put("/words", data = "<req>")]
pub async fn update(
    req: Json<UpdateWordRequest>,
    _admin: Admin,
    pool: &State<PgPool>,
) -> VortoResult<()> {
    word_service::update(req.into_inner(), pool).await
}

#[get("/words/stats")]
pub async fn word_stats(_admin: Admin, pool: &State<PgPool>) -> VortoResult<WordStats> {
    word_service::word_stats(pool).await
}
