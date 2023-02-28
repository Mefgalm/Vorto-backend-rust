use sqlx::{PgPool, query, query_as};

use crate::{domain::team::Team, error::VortoResult};

pub async fn insert(name: &str, pool: &PgPool) -> VortoResult<()> {
    query!("INSERT INTO teams (name) VALUES ($1)", name).execute(pool).await?;
    VortoResult::Ok(())
}

pub async fn get_by_ids_ordered(ids: &Vec<i32>, pool: &PgPool) -> VortoResult<Vec<Team>> {
    let ordered_ids_str_qry = 
        ids
        .iter()
        .enumerate()
        .map(|(order, id)| format!("({}, {})", order, id))
        .collect::<Vec<_>>()
        .join(",");

    let teams = query_as::<_, Team>(
        &format!(
            r#"
            SELECT * 
            FROM teams t
            JOIN (VALUES {}) s("order", team_id) ON team_id = t.id
            ORDER BY "order" ASC
            "#,
            ordered_ids_str_qry
        )
    )
    .fetch_all(pool)
    .await?;

    VortoResult::Ok(teams)
}