

use crate::{
    common::{reduce_results},
    error::{VortoError, VortoErrorCode, VortoResult},
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;

use super::{voc::Voc, word_definition};
use super::word_definition::WordDefinition;
use super::{
    common::validate_fn,
    enums::{WordDefinitionStatus, WordLoadStatus, WordStatus},
};

#[derive(Deserialize, Clone, Debug)]
pub struct Word {
    pub id: i32,
    pub body: String,
    pub status: String,
    pub is_edited_after_load: bool,
    pub load_status: String,
    pub difficulty: i32,
    pub timestamp: i64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WordDefinitionDTO {
    pub definition: String,
    pub status: WordDefinitionStatus,
    pub voc_id: Option<i32>,
}

lazy_static! {
    static ref BODY_REGEX: Regex = Regex::new(r".{1,255}").unwrap();
}

fn validate_body(body: &str) -> VortoResult<()> {
    validate_fn(
        || !BODY_REGEX.is_match(body),
        VortoError::new(VortoErrorCode::Validation, "Body size 1-255".to_owned()),
    )
}

fn check_timestamp(word_timestamp: i64, new_timestamp: i64) -> VortoResult<()> {
    if word_timestamp != new_timestamp {
        VortoResult::Err(VortoError::new(
            VortoErrorCode::Timestamp,
            "Timestamp is wrong".to_owned(),
        ))
    } else {
        VortoResult::Ok(())
    }
}

pub fn load_definitions(
    word: &Word,
    timestamp: i64,
    definitions_result: &VortoResult<Vec<(Option<Voc>, String)>>,
    time: &DateTime<Utc>,
) -> VortoResult<(Word, Vec<WordDefinition>)> {
    check_timestamp(word.timestamp, timestamp)?;

    match definitions_result {
        VortoResult::Err(_) => {
            let new_word = Word {
                load_status: WordLoadStatus::LoadedWithFail.to_string(),
                timestamp: time.timestamp(),
                ..word.clone()
            };

            VortoResult::Ok((new_word, vec![]))
        },
        VortoResult::Ok(definitions) if definitions.is_empty() => {
            let new_word = Word {
                load_status: WordLoadStatus::LoadedWithFail.to_string(),
                timestamp: time.timestamp(),
                ..word.clone()
            };

            VortoResult::Ok((new_word, vec![]))
        },
        VortoResult::Ok(definitions) => {
            let word_definitions = reduce_results(
                &definitions
                    .iter()
                    .enumerate()
                    .map(|(order, (voc, def))| {
                        word_definition::new(
                            -1,
                            &def,
                            &WordDefinitionStatus::NotActive,
                            order as i32,
                            word.id,
                            voc.as_ref().map(|v| v.id),
                        )
                    })
                    .collect::<Vec<_>>(),
            )?;
        
            let new_word = Word {
                load_status: WordLoadStatus::Loaded.to_string(),
                timestamp: time.timestamp(),
                ..word.clone()
            };
        
            VortoResult::Ok((new_word, word_definitions))
        }
    }
}

pub fn update(
    word: &Word,
    new_status: &WordStatus,
    difficulty: i32,
    timestamp: i64,
    word_definitions: &Vec<WordDefinitionDTO>,
    time: &DateTime<Utc>,
) -> VortoResult<(Word, Vec<WordDefinition>)> {
    check_timestamp(word.timestamp, timestamp)?;

    let new_word_definitions = reduce_results(
        &word_definitions
            .iter()
            .enumerate()
            .map(|(order, wd)| {
                word_definition::new(
                    -1,
                    &wd.definition,
                    &wd.status,
                    order as i32,
                    word.id,
                    wd.voc_id
                )
            })
            .collect::<Vec<_>>(),
    )?;

    let new_word = Word {
        status: new_status.to_string(),
        timestamp: time.timestamp(),
        difficulty,
        ..word.clone()
    };

    VortoResult::Ok((new_word, new_word_definitions))
}

#[allow(dead_code)]
pub fn new(
    id: i32,
    body: &str,
    status: WordStatus,
    is_edited_after_load: bool,
    load_status: WordLoadStatus,
    difficulty: i32,
    timestamp: i64,
) -> VortoResult<Word> {
    validate_body(body)?;

    VortoResult::Ok(Word {
        id,
        body: body.to_owned(),
        status: status.to_string(),
        is_edited_after_load,
        load_status: load_status.to_string(),
        difficulty,
        timestamp,
    })
}
