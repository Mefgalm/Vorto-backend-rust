use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Debug, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WordStatus {
    Active,
    NotActive,
    Draft,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Debug, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WordLoadStatus {
    NotLoaded,
    Loaded,
    LoadedWithFail,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Debug, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WordDefinitionStatus {
    Active,
    NotActive,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Debug, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum GameState {
    Active,
    Ended
}