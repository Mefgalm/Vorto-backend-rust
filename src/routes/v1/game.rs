use crate::error::VortoResult;
use crate::requests::{CompleteRoundRequest, CreateGameRequest};
use crate::responses::GameView;
use crate::services::*;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::PgPool;

#[post("/games", data = "<req>")]
pub async fn create(req: Json<CreateGameRequest>, pool: &State<PgPool>) -> VortoResult<GameView> {
    game_service::create(req.into_inner(), pool).await
}

#[put("/games", data = "<req>")]
pub async fn complete_round(req: Json<CompleteRoundRequest>, pool: &State<PgPool>) -> VortoResult<GameView> {
    game_service::complete_round(req.into_inner(), pool).await
}

#[get("/games/<id>")]
pub async fn game_view(id: i32, pool: &State<PgPool>) -> VortoResult<GameView> {
    game_service::game_view(id, pool).await
}