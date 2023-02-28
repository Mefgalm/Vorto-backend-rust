use std::collections::HashMap;

use crate::{
    domain::{self, game::Game},
    error::{VortoError, VortoErrorCode, VortoResult},
    responses::{GameTeamResultView, GameView, GameWordResultView, GameWordView, TeamView},
};
use sqlx::{query, query_as, PgPool, Postgres, Transaction};

pub async fn insert(
    game: &Game,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<i32> {
    let qry = query!(
        r#"
        INSERT INTO public.games
            (state, created_at, expired_at, word_count, penalty, round_time, winner_id, turn, "token")
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        game.state.to_string(),
        game.created_at,
        game.expired_at,
        game.word_count,
        game.penalty,
        game.round_time,
        game.winner_id,
        game.turn,
        game.token
    )
    .map(|r| r.id);

    VortoResult::Ok(run_qry!(qry, fetch_one, pool, tx))
}

pub async fn update(
    game: &Game,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<()> {
    let qry = query!(
        r#"
        UPDATE public.games
            SET state=$2, expired_at=$3, word_count=$4, penalty=$5, round_time=$6, winner_id=$7, turn=$8, "token"=$9
        WHERE id=$1
        "#,
        game.id,
        game.state,
        game.expired_at,
        game.word_count,
        game.penalty,
        game.round_time,
        game.winner_id,
        game.turn,
        game.token
    );

    run_qry!(qry, execute, pool, tx);

    VortoResult::Ok(())
}

fn game_not_found<T>() -> VortoResult<T> {
    VortoResult::Err(VortoError::new(
        VortoErrorCode::NotFound,
        "Game not found".to_owned(),
    ))
}

pub async fn get_by_id(id: i32, pool: &PgPool) -> VortoResult<Game> {
    let game_opt = query_as!(
        Game,
        r#"
        SELECT * FROM games WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(game) = game_opt {
        VortoResult::Ok(game)
    } else {
        game_not_found()
    }
}

pub async fn game_view(id: i32, pool: &PgPool) -> VortoResult<GameView> {
    let rows = query!(
        r#"
        SELECT 
            g.id,
            g.state,
            g.created_at,
            g.expired_at,
            g.word_count,
            g.penalty,
            g.round_time,
            g.winner_id,
            g.turn,
            g.token,
            tr.id                         AS "tr_id!",
            tr.team_id                    AS "tr_team_id!",
            tr.game_id                    AS "tr_game_id!",
            tr."order"                    AS "tr_order!",
            t.id                          AS "t_id!",
            t.name                        AS "t_name",
            wr.id                         AS "wr_id?",
            wr.result                     AS "wr_result?",
            wr.word_id                    AS "wr_word_id?",
            wr.team_result_id             AS "wr_team_result_id?",
            wr."order"                    AS "wr_order?",
            w.id                          AS "w_id?",
            w.body                        AS "w_body?",
            w.status                      AS "w_status?",
            w.is_edited_after_load        AS "w_is_edited_after_load?",
            w.load_status                 AS "w_load_status?",
            w.difficulty                  AS "w_difficulty?",
            w.timestamp                   AS "w_timestamp?",
            winner_tr.id                  AS "winner_tr_id?",
            winner_tr.team_id             AS "winner_tr_team_id?",
            winner_tr.game_id             AS "winner_tr_game_id?",
            winner_tr."order"             AS "winner_tr_order?",
            winner_t.id                   AS "winner_t_id?",
            winner_t.name                 AS "winner_t_name?",
            winner_wr.id                  AS "winner_wr_id?",
            winner_wr.result              AS "winner_wr_result?",
            winner_wr.word_id             AS "winner_wr_word_id?",
            winner_wr.team_result_id      AS "winner_wr_team_result_id?",
            winner_wr."order"             AS "winner_wr_order?",
            winner_w.id                   AS "winner_w_id?",
            winner_w.body                 AS "winner_w_body?",
            winner_w.status               AS "winner_w_status?",
            winner_w.is_edited_after_load AS "winner_w_is_edited_after_load?",
            winner_w.load_status          AS "winner_w_load_status?",
            winner_w.difficulty           AS "winner_w_difficulty?",
            winner_w.timestamp            AS "winner_w_timestamp?"
        FROM games g
        JOIN team_results tr             ON tr.game_id = g.id
        JOIn teams t                     ON tr.team_id = t.id
        LEFT JOIN team_results winner_tr ON g.winner_id = winner_tr.id
        LEFT JOIN teams winner_t         ON winner_tr.team_id = winner_t.id
        LEFT JOIN word_results winner_wr ON winner_wr.team_result_id = winner_tr.id
        LEFT JOIN words winner_w         ON winner_w.id = winner_wr.word_id
        LEFT JOIN word_results wr        ON wr.team_result_id = tr.id
        LEFT JOIN words w                ON w.id = wr.word_id
        WHERE g.id = $1
        "#,
        id
    )
    .fetch_all(pool)
    .await?;

    let mut winner_team_result_opt: Option<GameTeamResultView> = None;
    let mut winner_word_results: HashMap<i32, GameWordResultView> = HashMap::new();
    let mut team_results: HashMap<i32, (GameTeamResultView, HashMap<i32, GameWordResultView>)> =
        HashMap::new();

    let first_row_opt = rows.first();
    if first_row_opt.is_none() {
        return game_not_found();
    }
    let first_row = first_row_opt.unwrap();

    for row in &rows {
        if winner_team_result_opt.is_none() {
            if let Some(winner_tr_id) = row.winner_tr_id {
                let team_view = TeamView {
                    id: row.t_id,
                    name: row.t_name.clone(),
                };

                let winner_team_result = GameTeamResultView {
                    id: winner_tr_id,
                    score: 0, // Will be assigned later
                    team: team_view,
                    word_results: vec![], // Will be assigned later
                };

                winner_team_result_opt = Some(winner_team_result);
            };
        };
        if let Some(winner_wr_id) = row.winner_wr_id {
            let winner_word = GameWordView {
                id: row.winner_w_id.unwrap(),
                body: row.winner_w_body.clone().unwrap(),
            };
            let winner_word_result = GameWordResultView {
                result: row.winner_wr_result.unwrap(),
                order: row.winner_wr_order.unwrap(),
                word: winner_word,
            };

            winner_word_results
                .entry(winner_wr_id)
                .or_insert(winner_word_result);
        }

        let team = TeamView {
            id: row.t_id,
            name: row.t_name.clone(),
        };

        let team_result = GameTeamResultView {
            id: row.tr_id,
            score: 0, // Will be assigned later
            team,
            word_results: vec![], // Will be assigned later
        };

        team_results
            .entry(row.tr_id)
            .or_insert((team_result, HashMap::new()));

        if let Some(wr_id) = row.wr_id {
            let word = GameWordView {
                id: row.w_id.unwrap(),
                body: row.w_body.clone().unwrap(),
            };
            let word_result = GameWordResultView {
                result: row.wr_result.unwrap(),
                order: row.wr_order.unwrap(),
                word,
            };

            if let Some(team_result) = team_results.get_mut(&row.wr_team_result_id.unwrap()) {
                team_result.1.entry(wr_id).or_insert(word_result);
            }
        }
    }

    if let Some(winner_team_result) = winner_team_result_opt.as_mut() {
        let mut winner_word_results = winner_word_results.values().cloned().collect::<Vec<_>>();
        winner_team_result.score = domain::game::calc_score(
            first_row.penalty,
            winner_word_results.iter().map(|wr| wr.result),
        );
        winner_word_results.sort_by(|wr1, wr2| wr1.order.cmp(&wr2.order));
        winner_team_result.word_results = winner_word_results;
    };

    let mut team_result_views: Vec<GameTeamResultView> = team_results
        .values_mut()
        .map(|(team_result, team)| {
            let mut word_results = team.values().cloned().collect::<Vec<_>>();
            team_result.score = domain::game::calc_score(
                first_row.penalty,
                word_results.iter().map(|wr| wr.result),
            );
            word_results.sort_by(|wr1, wr2| wr1.order.cmp(&wr2.order));
            team_result.word_results = word_results;
            team_result.clone()
        })
        .collect();

    team_result_views.sort_by(|tr1, tr2| tr2.score.cmp(&tr1.score));

    let game_view = GameView {
        id: first_row.id,
        state: first_row.state.clone(),
        word_count: first_row.word_count,
        penalty: first_row.penalty,
        round_time: first_row.round_time,
        turn: first_row.turn,
        token: first_row.token.clone(),
        created_at: first_row.created_at,
        expired_at: first_row.expired_at,
        team_results: team_result_views,
        winner: winner_team_result_opt,
    };

    VortoResult::Ok(game_view)
}
