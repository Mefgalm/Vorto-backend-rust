use sqlx::{query, PgPool, Postgres, Transaction};

use crate::{
    domain::{
        enums::WordDefinitionStatus,
        word_definition::{WordDefinition},
    },
    error::VortoResult,
};

use std::{collections::HashMap};

#[allow(dead_code)]
fn word_definition_insert(
    definitions: &Vec<(Vec<String>, String)>,
    short_id_map: &HashMap<String, i32>,
    word_id: i32,
) -> String {
    definitions
        .iter()
        .enumerate()
        .map(|(i, (vocs, def))| {
            let first_voc = vocs
                .iter()
                .next()
                .map(|v| v.clone())
                .unwrap_or(String::new());
            let voc_id = short_id_map
                .get(&first_voc)
                .map(|x| x.to_string())
                .unwrap_or(String::from("null"));
            format!(
                "('{}', '{}', {}, {}, {})",
                def,
                WordDefinitionStatus::Active.to_string(),
                i,
                word_id,
                voc_id
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub async fn delete_by_word_id(
    word_id: i32,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<()> {
    run_qry!(
        query!("DELETE FROM word_definitions WHERE word_id = $1", word_id),
        execute,
        pool,
        tx
    );
    VortoResult::Ok(())
}

pub async fn insert(
    word_definition: &WordDefinition,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<i32> {
    let qry = query!(
        r#"
        INSERT INTO word_definitions
            (definition, status, "order", word_id, voc_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id   
        "#,
        word_definition.definition,
        word_definition.status,
        word_definition.order,
        word_definition.word_id,
        word_definition.voc_id
    )
    .map(|r| r.id);

    VortoResult::Ok(run_qry!(qry, fetch_one, pool, tx))
}

#[allow(dead_code)]
pub async fn update(
    word_definition: &WordDefinition,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<()> {
    let qry = query!(
        r#"
        UPDATE word_definitions
        SET definition = $1, 
            status = $2, 
            "order" = $3,
            word_id = $4,
            voc_id = $5
        WHERE id = $6
        "#,
        word_definition.definition,
        word_definition.status,
        word_definition.order,
        word_definition.word_id,
        word_definition.voc_id,
        word_definition.id
    );

    run_qry!(qry, execute, pool, tx);

    VortoResult::Ok(())
}
