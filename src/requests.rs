use serde::Deserialize;

use crate::{domain::{enums::{WordLoadStatus, WordStatus}, word::WordDefinitionDTO}, services::word_service::FieldOrder};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct SearchRequest {
    pub text: String,
    pub statuses: Vec<WordStatus>,
    pub load_statuses: Vec<WordLoadStatus>,
    pub difficulties: Vec<i32>,
    pub field_order: FieldOrder,
    pub skip: i64,
    pub take: i64,
}

#[derive(Deserialize, Debug)]
pub struct LoadDefinitionRequest {
    pub id: i32,
    pub timestamp: i64,
}

#[derive(Deserialize, Debug)]   
pub struct UpdateWordRequest {
    pub id: i32,
    pub status: WordStatus,
    pub difficulty: i32,
    pub timestamp: i64,
    pub definitions: Vec<WordDefinitionDTO>
}


#[derive(Deserialize, Debug)]
pub struct CreateGameRequest {
    pub penalty: bool,
    pub round_time: i32,
    pub team_ids: Vec<i32>,
    pub word_count: i32
}

#[derive(Deserialize, Debug)]
pub struct WordResultsDTO {
    pub result: bool,
    pub word_id: i32
}

#[derive(Deserialize, Debug)]
pub struct CompleteRoundRequest {
    pub id: i32,
    pub token: String,
    pub word_results: Vec<WordResultsDTO>
}