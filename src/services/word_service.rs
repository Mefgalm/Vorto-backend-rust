use std::{fmt::Display, str::FromStr};

use crate::domain::enums::{WordDefinitionStatus, WordLoadStatus, WordStatus};
use crate::domain::voc::Voc;
use crate::requests::UpdateWordRequest;
use crate::{
    common::{group, vec_to_map},
    db,
    error::VortoResult,
    responses::*,
};
use crate::{domain, wiki_parser_service};
use chrono::Utc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, PgPool};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FieldMatch {
    Body,
    Status,
    LoadStatus,
    Difficulty,
}

#[derive(Deserialize, Debug)]
pub struct FieldOrder {
    field_match: FieldMatch,
    is_asc: bool,
}

#[derive(Serialize, FromRow, Debug)]
pub struct WordQry {
    pub id: i32,
    pub body: String,
    pub status: String,
    pub is_edited_after_load: bool,
    pub load_status: String,
    pub timestamp: i64,
    pub difficulty: i32,

    pub word_definition_id: Option<i32>,
    pub definition: Option<String>,
    pub word_definition_status: Option<String>,
    pub order: Option<i32>,

    pub voc_id: Option<i32>,
    pub full: Option<String>,
    pub short: Option<String>,
}

const TRUE: &str = "true";

fn in_qry(field: &str, values: &Vec<String>) -> String {
    format!("{} IN ({})", field, values.join(","))
}

fn db_str<T: Display>(val: T) -> String {
    format!("'{}'", val.to_string())
}

fn field_order_qry(field_order: &FieldOrder) -> String {
    let field = match field_order.field_match {
        FieldMatch::Body => "body",
        FieldMatch::Status => "status",
        FieldMatch::LoadStatus => "load_status",
        FieldMatch::Difficulty => "difficulty",
    };
    let order = if field_order.is_asc { "ASC" } else { "DESC" };
    format!("{} {}", field, order)
}

pub async fn search(
    text: &str,
    statuses: &Vec<WordStatus>,
    load_statuses: &Vec<WordLoadStatus>,
    difficulties: &Vec<i32>,
    field_order: &FieldOrder,
    skip: i64,
    take: i64,
    pool: &PgPool,
) -> VortoResult<Vec<WordView>> {
    let text_q = if text.is_empty() {
        TRUE.to_owned()
    } else {
        format!("body LIKE '{}%'", text)
    };

    let status_q = if statuses.is_empty() {
        TRUE.to_owned()
    } else {
        in_qry("status", &statuses.iter().map(db_str).collect())
    };

    let load_status_q = if load_statuses.is_empty() {
        TRUE.to_owned()
    } else {
        in_qry("load_status", &load_statuses.iter().map(db_str).collect())
    };

    let difficulty_q = if difficulties.is_empty() {
        TRUE.to_owned()
    } else {
        in_qry(
            "difficulty",
            &difficulties.iter().map(|x| x.to_string()).collect(),
        )
    };
    let field_order_q = field_order_qry(&field_order);
    let wq = query_as::<_, WordQry>(&format!(
        r#" SELECT 
                w.id,
                w.body,
                w.status,
                w.is_edited_after_load, 
                w.load_status,
                w.timestamp,
                w.difficulty,
                wd.id AS word_definition_id,
                wd.definition,                
                wd.status AS word_definition_status,
                wd.order,
                v.id AS voc_id,
                v.full,
                v.short
            FROM (SELECT * 
                FROM words
                WHERE {} AND {} AND {} AND {}
                ORDER BY {}
                OFFSET {}
                LIMIT {}) w
            LEFT JOIN word_definitions wd ON wd.word_id = w.id
            LEFT JOIN vocs v ON wd.voc_id = v.id
            ORDER BY w.{}, wd.order"#,
        text_q, status_q, load_status_q, difficulty_q, field_order_q, skip, take, field_order_q
    ))
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| {
        let word = WordView {
            id: r.id,
            body: r.body.clone(),
            status: WordStatus::from_str(&r.status).unwrap(),
            is_edited_after_load: r.is_edited_after_load,
            load_status: WordLoadStatus::from_str(&r.load_status).unwrap(),
            definitions: vec![],
            timestamp: r.timestamp,
            difficulty: r.difficulty,
        };
        let definition = r.word_definition_id.map(|wd_id| WordDefinitionView {
            id: wd_id,
            definition: r.definition.as_ref().unwrap().clone(),
            status: WordDefinitionStatus::from_str(&r.word_definition_status.as_ref().unwrap())
                .unwrap(),
            order: r.order.unwrap(),
            voc: r.voc_id.map(|voc_id| VocView {
                id: voc_id,
                full: r.full.as_ref().unwrap().clone(),
                short: r.short.as_ref().unwrap().clone(),
            }),
        });
        (word, definition)
    })
    .collect();

    let word_views = group(
        &wq,
        |(word, _)| &word.id,
        |(word, _)| word.clone(),
        |(_, wv)| wv.clone(),
    )
    .iter_mut()
    .map(|(w, wdv)| {
        w.definitions = wdv.to_vec();
        w.clone()
    })
    .collect();

    VortoResult::Ok(word_views)
}

fn get_all_vocs(def_with_vocs: &Vec<(Vec<String>, String)>) -> Vec<String> {
    def_with_vocs
        .iter()
        .flat_map(|x| x.0.clone())
        .unique()
        .collect()
}

async fn _load_vocs_to_definitions_new(
    definitions: &Vec<(Vec<String>, String)>,
    pool: &PgPool,
) -> VortoResult<Vec<(Vec<Voc>, String)>> {
    let shorts = get_all_vocs(definitions);
    let vocs = db::voc::get_by_shorts(&shorts, pool).await?;
    let vocs_short_map = vec_to_map(&vocs, |v| v.short.clone(), |v| v.clone());

    VortoResult::Ok(
        definitions
            .iter()
            .map(|(vocs, def)| {
                let db_vocs = vocs
                    .iter()
                    .filter_map(|v| vocs_short_map.get(v))
                    .map(Clone::clone)
                    .collect::<Vec<_>>();
                (db_vocs, def.clone())
            })
            .collect::<Vec<_>>(),
    )
}

async fn load_vocs_to_definitions(
    definitions_result: &VortoResult<Vec<(Vec<String>, String)>>,
    pool: &PgPool,
) -> VortoResult<Vec<(Option<Voc>, String)>> {
    match definitions_result {
        VortoResult::Ok(definitions) => {
            let shorts = get_all_vocs(definitions);
            let vocs = db::voc::get_by_shorts(&shorts, pool).await?;
            let vocs_short_map = vec_to_map(&vocs, |v| v.short.clone(), |v| v.clone());
            VortoResult::Ok(
                definitions
                    .iter()
                    .map(|(vocs, def)| {
                        let first_db_voc = vocs
                            .iter()
                            .next()
                            .and_then(|voc| vocs_short_map.get(voc))
                            .map(Clone::clone);
                        (first_db_voc, def.clone())
                    })
                    .collect::<Vec<_>>(),
            )
        }
        VortoResult::Err(_) => VortoResult::Ok(vec![]),
    }
}

pub async fn load_definitions(id: i32, timestamp: i64, pool: &PgPool) -> VortoResult<()> {
    let word = db::word::get_by_id(id, pool).await?;
    let definitions_result = wiki_parser_service::parse(&word.body).await;
    let db_voc_and_defs = load_vocs_to_definitions(&definitions_result, pool).await;

    let (new_word, new_word_definitions) =
        domain::word::load_definitions(&word, timestamp, &db_voc_and_defs, &Utc::now())?;

    let mut tx = pool.begin().await?;
    db::word::update(&new_word, pool, Some(&mut tx)).await?;

    db::word_definition::delete_by_word_id(word.id, pool, Some(&mut tx)).await?;
    for wd in new_word_definitions {
        db::word_definition::insert(&wd, pool, Some(&mut tx)).await?;
    }
    tx.commit().await?;

    VortoResult::Ok(())
}

pub async fn update(req: UpdateWordRequest, pool: &PgPool) -> VortoResult<()> {
    let word = db::word::get_by_id(req.id, pool).await?;

    let (new_word, new_word_definitions) = domain::word::update(
        &word,
        &req.status,
        req.difficulty,
        req.timestamp,
        &req.definitions,
        &Utc::now(),
    )?;

    let mut tx = pool.begin().await?;
    db::word::update(&new_word, pool, Some(&mut tx)).await?;

    db::word_definition::delete_by_word_id(word.id, pool, Some(&mut tx)).await?;
    for wd in new_word_definitions {
        db::word_definition::insert(&wd, pool, Some(&mut tx)).await?;
    }
    tx.commit().await?;

    VortoResult::Ok(())
}

pub async fn word_stats(pool: &PgPool) -> VortoResult<WordStats> {
    db::word::word_stats(pool).await
}
