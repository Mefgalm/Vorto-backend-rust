use sqlx::{query, query_as, PgPool, Postgres, Transaction};

use crate::{
    domain::word::Word,
    error::VortoResult,
    responses::{DifficultyStats, LoadStatusStats, StatusStats, WordStats},
};

pub async fn get_by_id(id: i32, pool: &PgPool) -> VortoResult<Word> {
    let word = query_as!(Word, 
        r#"
        SELECT * FROM words WHERE id = $1
        "#, id)
        .fetch_one(pool)
        .await?;

    VortoResult::Ok(word)
}

pub async fn update(
    word: &Word,
    pool: &PgPool,
    tx: Option<&mut Transaction<'_, Postgres>>,
) -> VortoResult<()> {
    let qry = query!(
        r#"
        UPDATE words
        SET body = $1,            
            status = $2, 
            is_edited_after_load = $3,
            load_status = $4,
            difficulty = $5,
            timestamp = $6
        WHERE id = $7
        "#,
        word.body,
        word.status,
        word.is_edited_after_load,
        word.load_status,
        word.difficulty,
        word.timestamp,
        word.id
    );

    run_qry!(qry, execute, pool, tx);

    VortoResult::Ok(())
}

#[derive(sqlx::FromRow, Debug)]
struct WordStatsQry {
    // status
    pub active: i64,
    pub not_active: i64,
    pub draft: i64,

    // load status
    pub loaded: i64,
    pub loaded_with_fail: i64,
    pub not_loaded: i64,

    // diffucilty
    pub easy: i64,
    pub medium: i64,
    pub hard: i64,

    pub total: i64,
}

pub async fn word_stats(pool: &PgPool) -> VortoResult<WordStats> {
    let word_stats_qry = query_as!(
        WordStatsQry,
        r#"
        select 
        count(w.id) as "total!",
        SUM(case w.status when 'active' then 1 else 0 end) as "active!",	
        SUM(case w.status when 'not_active' then 1 else 0 end) as "not_active!",
        SUM(case w.status when 'draft' then 1 else 0 end) as "draft!",	
        SUM(case w.load_status when 'loaded' then 1 else 0 end) as "loaded!",	
        SUM(case w.load_status when 'loaded_with_fail' then 1 else 0 end) as "loaded_with_fail!",
        SUM(case w.load_status when 'not_loaded' then 1 else 0 end) as "not_loaded!",
        SUM(case w.difficulty when 0 then 1 else 0 end) as "easy!",	
        SUM(case w.difficulty when 1 then 1 else 0 end) as "medium!",
        SUM(case w.difficulty when 2 then 1 else 0 end) as "hard!"
        from words w;
        "#,
    )
    .fetch_one(pool)
    .await?;

    VortoResult::Ok(WordStats {
        status: StatusStats {
            active: word_stats_qry.active,
            draft: word_stats_qry.draft,
            not_active: word_stats_qry.not_active,
        },
        load_status: LoadStatusStats {
            loaded: word_stats_qry.loaded,
            loaded_with_fail: word_stats_qry.loaded_with_fail,
            not_loaded: word_stats_qry.not_loaded,
        },
        difficulties: DifficultyStats {
            easy: word_stats_qry.easy,
            medium: word_stats_qry.medium,
            hard: word_stats_qry.hard,
        },
        total: word_stats_qry.total,
    })
}
