use crate::{error::VortoResult, responses::TeamView};

use sqlx::{query_as, PgPool};

pub async fn get_teams(pool: &PgPool) -> VortoResult<Vec<TeamView>> {
    let teams = query_as!(
        TeamView,
        "SELECT id, name 
        FROM teams"
    )
    .fetch_all(pool)
    .await?;
    VortoResult::Ok(teams)
}
