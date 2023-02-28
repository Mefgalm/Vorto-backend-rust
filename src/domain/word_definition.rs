use crate::error::{VortoError, VortoErrorCode, VortoResult};

use super::{common::validate_fn, enums::WordDefinitionStatus};

#[derive(Clone, Debug)]
pub struct WordDefinition {
    pub id: i32,
    pub definition: String,
    pub status: String,
    pub order: i32,
    pub word_id: i32,
    pub voc_id: Option<i32>,
}

fn validate_definition(definition: &str) -> VortoResult<()> {
    validate_fn(
        || {
            let definition_len = definition.len();
            definition_len < 10 || definition_len > 1000
        },
        VortoError::new(
            VortoErrorCode::Validation,
            "Definition's lenght should be in range 10-1000".to_owned(),
        ),
    )
}

pub fn new(
    id: i32,
    definition: &str,
    status: &WordDefinitionStatus,
    order: i32,
    word_id: i32,
    voc_id: Option<i32>,
) -> VortoResult<WordDefinition> {
    validate_definition(definition)?;

    VortoResult::Ok(WordDefinition {
        id,
        definition: definition.to_owned(),
        status: status.to_string(),
        order,
        word_id,
        voc_id,
    })
}
