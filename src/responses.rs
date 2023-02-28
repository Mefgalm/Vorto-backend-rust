use chrono::NaiveDateTime;
use serde::{Serialize};

use crate::{domain::enums::{WordDefinitionStatus, WordLoadStatus, WordStatus}};

#[derive(Serialize, Clone)]
pub struct TeamView {
    pub id: i32,
    pub name: String
}

#[derive(Serialize, Clone)]
pub struct VocView {
    pub id: i32,
    pub full: String,
    pub short: String,
}

#[derive(Serialize, Clone)]
pub struct WordDefinitionView {
    pub id: i32,
    pub definition: String,
    pub status: WordDefinitionStatus,
    pub order: i32,
    pub voc: Option<VocView>,
}

#[derive(Serialize, Clone)]
pub struct WordView {
    pub id: i32,
    pub body: String,
    pub status: WordStatus,
    pub is_edited_after_load: bool,
    pub load_status: WordLoadStatus,
    pub definitions: Vec<WordDefinitionView>,
    pub timestamp: i64,
    pub difficulty: i32,
}

#[derive(Serialize, Clone)]
pub struct UserView {
    pub id: i32,
    pub email: String,
    pub created_at: NaiveDateTime
}

#[derive(Serialize)]
pub struct StatusStats {
    pub active: i64,
    pub draft: i64,
    pub not_active: i64
}

#[derive(Serialize)]
pub struct LoadStatusStats {
    pub loaded: i64,
    pub loaded_with_fail: i64,
    pub not_loaded: i64
}

#[derive(Serialize)]
pub struct DifficultyStats {
    pub easy: i64,
    pub medium: i64,
    pub hard: i64
}

#[derive(Serialize)]
pub struct WordStats {
    pub status: StatusStats,
    pub load_status: LoadStatusStats,
    pub difficulties: DifficultyStats,
    pub total: i64
}

#[derive(Serialize, Clone)]
pub struct GameWordView {
    pub id: i32,
    pub body: String,
}

#[derive(Serialize, Clone)]
pub struct GameWordResultView {
    pub result: bool,
    pub order: i32,
    pub word: GameWordView
}

#[derive(Serialize, Clone)]
pub struct GameTeamResultView {
    pub id: i32,
    pub score: i32,
    pub team: TeamView,
    pub word_results: Vec<GameWordResultView>
}

#[derive(Serialize, Clone)]
pub struct GameView {
    pub id: i32,
    pub penalty: bool,
    pub state: String,
    pub token: String,
    pub turn: i32,
    pub word_count: i32,
    pub round_time: i32,
    pub team_results: Vec<GameTeamResultView>,
    pub winner: Option<GameTeamResultView>,
    pub created_at: NaiveDateTime,
    pub expired_at: NaiveDateTime
}