use sqlx::{query, PgPool, Postgres, Transaction};

use crate::{domain::word_result::WordResult, error::VortoResult};

pub async fn insert(
    word_result: &WordResult,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<i32> {
    let qry = query!(
        r#"
        INSERT INTO word_results
            ("result", "order", word_id, team_result_id)
        VALUES($1, $2, $3, $4)
        RETURNING id
        "#,
        word_result.result,
        word_result.order,
        word_result.word_id,
        word_result.team_result_id
    )
    .map(|r| r.id);

    VortoResult::Ok(run_qry!(qry, fetch_one, pool, tx))
}
