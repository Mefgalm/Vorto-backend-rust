use sqlx::{query, PgPool, Postgres, Transaction};

use crate::{
    common::group,
    domain::{team_result::TeamResult, word_result::WordResult},
    error::VortoResult,
};

pub async fn insert(
    team_result: &TeamResult,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<i32> {
    let qry = query!(
        r#"
        INSERT INTO team_results
            (team_id, game_id, "order")
        VALUES($1, $2, $3)
        RETURNING id
        "#,
        team_result.team_id,
        team_result.game_id,
        team_result.order
    )
    .map(|r| r.id);

    VortoResult::Ok(run_qry!(qry, fetch_one, pool, tx))
}

pub async fn team_results_words_by_game(
    game_id: i32,
    pool: &PgPool,
) -> VortoResult<Vec<(TeamResult, Vec<WordResult>)>> {
    let datas = query!(
        r#"
        SELECT tr.id,
               tr.team_id,
               tr.game_id,
               tr."order",
               wr.id AS "wr_id?",
               wr.word_id AS "wr_word_id?",
               wr.result AS "wr_result?",
               wr."order" AS "wr_order?",
               wr.team_result_id AS "wr_team_result_id?"
        FROM team_results tr
        LEFT JOIN word_results wr ON wr.team_result_id = tr.id
        WHERE tr.game_id = $1
        "#,
        game_id
    )
    .fetch_all(pool)
    .await?;

    let team_results_words = group(
        &datas,
        |r| &r.id,
        |r| TeamResult {
            id: r.id,
            team_id: r.team_id,
            game_id: r.game_id,
            order: r.order,
        },
        |r| {
            if let Some(wr_id) = r.wr_id {
                Some(WordResult {
                    id: wr_id,
                    result: r.wr_result.unwrap(),
                    word_id: r.wr_word_id.unwrap(),
                    team_result_id: r.wr_team_result_id.unwrap(),
                    order: r.wr_order.unwrap(),
                })
            } else {
                None
            }
        },
    );

    VortoResult::Ok(team_results_words)
}
