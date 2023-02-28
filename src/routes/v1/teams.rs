use crate::error::VortoResult;
use crate::responses::TeamView;
use crate::services::*;
use rocket::State;
use sqlx::PgPool;

#[get("/teams")]
pub async fn get_teams(pool: &State<PgPool>) -> VortoResult<Vec<TeamView>> {
    team_service::get_teams(pool.inner()).await
}
