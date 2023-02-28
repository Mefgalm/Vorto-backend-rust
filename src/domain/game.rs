use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use uuid::Uuid;

use crate::{
    common::reduce_results,
    error::{VortoError, VortoErrorCode, VortoResult},
};

use super::{
    common::validate_fn,
    enums::GameState,
    team::Team,
    team_result::{self, TeamResult},
    word_result::{self, WordResult},
};

const EXPIRED_HOURS: i64 = 10;

#[derive(Debug, Clone)]
pub struct Game {
    pub id: i32,
    pub state: String,
    pub word_count: i32,
    pub penalty: bool,
    pub round_time: i32,
    pub winner_id: Option<i32>,
    pub turn: i32,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expired_at: NaiveDateTime,
}

pub fn validate_team_count(teams: &Vec<Team>) -> VortoResult<()> { 
    validate_fn(
        || teams.len() < 2,
        VortoError::new(
            VortoErrorCode::TeamSize,
            "Should be at least 2 teams".to_owned(),
        ),
    )
}

pub fn validate_round_time(round_time: i32) -> VortoResult<()> {
    validate_fn(
        || round_time < 1 || round_time > 1000,
        VortoError::new(
            VortoErrorCode::Validation,
            "Round time valid range 1-1000".to_owned(),
        ),
    )
}

fn validate_word_count(word_count: i32) -> VortoResult<()> {
    validate_fn(
        || word_count < 1 || word_count > 500,
        VortoError::new(
            VortoErrorCode::Validation,
            "Word count valid range 1-5000".to_owned(),
        ),
    )
}

pub fn new(
    id: i32,
    word_count: i32,
    penalty: bool,
    round_time: i32,
    teams: &Vec<Team>,
    now: &DateTime<Utc>,
) -> VortoResult<(Game, Vec<TeamResult>)> {
    validate_team_count(teams)?;
    validate_round_time(round_time)?;
    validate_word_count(word_count)?;

    let game = Game {
        id,
        state: GameState::Active.to_string(),
        word_count,
        penalty,
        round_time,
        winner_id: None,
        turn: 0,
        token: Uuid::new_v4().to_string(),
        created_at: now.naive_utc(),
        expired_at: (*now + Duration::hours(EXPIRED_HOURS)).naive_utc(),
    };

    let team_results = reduce_results(
        &teams
            .iter()
            .enumerate()
            .map(|(index, team)| team_result::new(-1, team.id, -1, index as i32))
            .collect::<Vec<_>>(),
    )?;

    VortoResult::Ok((game, team_results))
}

pub fn validate_token(game: &Game, token: &str) -> VortoResult<()> {
    validate_fn(
        || game.token != token,
        VortoError::new(
            VortoErrorCode::InvalidGameToken,
            "Invalid game token".to_owned(),
        ),
    )
}

pub fn validate_active(game: &Game) -> VortoResult<()> {
    validate_fn(
        || game.state != GameState::Active.to_string(),
        VortoError::new(VortoErrorCode::ActiveGame, "Game must be active".to_owned()),
    )
}

pub fn validate_expired(game: &Game, now: DateTime<Utc>) -> VortoResult<()> {
    validate_fn(
        || game.expired_at < now.naive_utc(),
        VortoError::new(
            VortoErrorCode::ExpiredGame,
            format!("Game expired at {}", game.expired_at),
        ),
    )
}

fn get_current_team_result(
    game: &Game,
    team_results_words: &Vec<(TeamResult, Vec<WordResult>)>,
) -> TeamResult {
    team_results_words
        .iter()
        .cycle()
        .skip(game.turn as usize)
        .next()
        .unwrap()
        .0
        .clone()
}

fn get_game_word_count(team_results_words: &Vec<(TeamResult, Vec<WordResult>)>) -> usize {
    team_results_words.iter().map(|(_, wrs)| wrs.len()).sum()
}

fn validate_words_count(
    game: &Game,
    team_results_words: &Vec<(TeamResult, Vec<WordResult>)>,
    word_results: &Vec<(i32, bool)>,
) -> VortoResult<()> {
    let current_game_word_count = get_game_word_count(team_results_words);

    validate_fn(
        || current_game_word_count + word_results.len() > game.word_count as usize,
        VortoError::new(VortoErrorCode::TooManyWords, "Too many words".to_owned()),
    )
}

fn get_score(penalty: bool, result: bool) -> i32 {
    match (result, penalty) {
        (true, _) => 1,
        (false, true) => -1,
        (false, false) => 0,
    }
}

pub fn calc_score<I: Iterator<Item = bool>>(penalty: bool, scores: I) -> i32 {
    scores
        .map(|result| get_score(penalty, result))
        .sum()
}

fn get_hightest_score_team_result(
    game: &Game,
    team_results_words: &Vec<(TeamResult, Vec<WordResult>)>,
    word_with_results: &Vec<(i32, bool)>,
    team_id: i32,
) -> TeamResult {
    team_results_words
        .iter()
        .max_by_key(|(tr, wr)| {
            let word_result_score = calc_score(game.penalty, wr.iter().map(|wr| wr.result));
            word_result_score
                + if tr.team_id == team_id {
                    calc_score(game.penalty, word_with_results.iter().map(|(_, r)| *r))
                } else {
                    0
                }
        })
        .unwrap()
        .0
        .clone()
}

pub fn complete_round(
    game: &Game,
    team_results_words: &Vec<(TeamResult, Vec<WordResult>)>,
    new_word_with_results: &Vec<(i32, bool)>,
    token: &str,
    now: DateTime<Utc>,
) -> VortoResult<(Game, Vec<WordResult>)> {
    validate_token(game, token)?;
    validate_active(game)?;
    validate_expired(game, now)?;
    let current_team_result = get_current_team_result(game, team_results_words);
    validate_words_count(game, team_results_words, new_word_with_results)?;

    let is_game_over = game.word_count as usize
        == get_game_word_count(team_results_words) + new_word_with_results.len();

    let game_clone = game.clone();
    let new_game = if is_game_over {
        let highest_score_team_result = get_hightest_score_team_result(
            game,
            team_results_words,
            new_word_with_results,
            current_team_result.team_id,
        );
        Game {
            winner_id: Some(highest_score_team_result.id),
            state: GameState::Ended.to_string(),
            ..game_clone
        }
    } else {
        Game {
            turn: game.turn + 1,
            ..game_clone
        }
    };

    let new_word_results = new_word_with_results
        .iter()
        .enumerate()
        .map(|(order, (word_id, result))| word_result::new(-1, *result, order as i32, *word_id, current_team_result.id))
        .collect::<Vec<_>>();

    VortoResult::Ok((new_game, new_word_results))
}
