use sqlx::{query_as, PgPool};

use crate::{error::VortoResult, responses::VocView};

pub async fn get_vocs(pool: &PgPool) -> VortoResult<Vec<VocView>> {
    let vocs = query_as!(
        VocView,
        "SELECT *
        FROM vocs"
    )
    .fetch_all(pool)
    .await?;
    VortoResult::Ok(vocs)
}
