use chrono::Utc;
use itertools::Itertools;
use sqlx::PgPool;

use crate::{
    db, domain,
    error::VortoResult,
    requests::{CompleteRoundRequest, CreateGameRequest},
    responses::GameView,
};

pub async fn create(req: CreateGameRequest, pool: &PgPool) -> VortoResult<GameView> {
    let teams = db::team::get_by_ids_ordered(&req.team_ids, pool).await?;

    let (game, mut team_results) = domain::game::new(
        -1,
        req.word_count,
        req.penalty,
        req.round_time,
        &teams,
        &Utc::now(),
    )?;

    let mut tx = pool.begin().await?;

    let game_id = db::game::insert(&game, pool, Some(&mut tx)).await?;
    for team_result in team_results.iter_mut() {
        // Does this look like a hack?
        // Should I use UUID instead of DB generated ids?
        team_result.game_id = game_id;
        db::team_result::insert(&team_result, pool, Some(&mut tx)).await?;
    }

    tx.commit().await?;

    let game_view = db::game::game_view(game_id, pool).await?;

    VortoResult::Ok(game_view)
}

pub async fn complete_round(req: CompleteRoundRequest, pool: &PgPool) -> VortoResult<GameView> {
    let game = db::game::get_by_id(req.id, pool).await?;
    let team_results_words = db::team_result::team_results_words_by_game(game.id, pool).await?;

    let (game, word_results) = domain::game::complete_round(
        &game,
        &team_results_words,
        &req.word_results
            .iter()
            .unique_by(|wd| wd.word_id)
            .map(|wd| (wd.word_id, wd.result))
            .collect(),
        &req.token,
        Utc::now(),
    )?;

    let mut tx = pool.begin().await?;

    db::game::update(&game, pool, Some(&mut tx)).await?;
    for word_result in word_results {
        db::word_result::insert(&word_result, pool, Some(&mut tx)).await?;
    }

    tx.commit().await?;
    let game_view = db::game::game_view(game.id, pool).await?;

    VortoResult::Ok(game_view)
}

pub async fn game_view(id: i32, pool: &PgPool) -> VortoResult<GameView> {
    db::game::game_view(id, pool).await
}
