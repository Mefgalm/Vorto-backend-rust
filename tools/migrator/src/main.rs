use dotenv::dotenv;
use futures::StreamExt;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, query, query_as, Executor, Pool, Postgres};
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::{any::Any, collections::HashMap, env};
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Word {
    pub id: i64,
    pub body: String,
    pub status: String,
    pub is_edited_after_load: bool,
    pub load_status: String,
    pub difficulty: i32,
    pub timestamp: i32,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct WordDefinition {
    pub id: i64,
    pub definition: String,
    pub status: String,
    pub order: i32,
    pub word_id: i64,
    pub voc_id: Option<i64>,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Voc {
    pub id: i64,
    pub short: String,
    pub full: String,
}

pub fn vec_to_map<T, K, V>(
    vec: &Vec<T>,
    key_fn: impl Fn(&T) -> K,
    value_fn: impl Fn(&T) -> V,
) -> HashMap<K, V>
where
    K: Hash + Eq,
{
    let mut hash_map: HashMap<K, V> = HashMap::new();
    for v in vec {
        hash_map.insert(key_fn(&v), value_fn(&v));
    }
    hash_map
}

async fn create_connection_by_key(
    connection_string_key: &str,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let source_database =
        env::var(connection_string_key).expect(&format!("{} must be set", connection_string_key));
    let pool = PgPoolOptions::new().connect(&source_database).await?;
    Ok(pool)
}

async fn vocs_load_from_source(source: &Pool<Postgres>) -> Result<Vec<Voc>, sqlx::Error> {
    let voc_records = query_as::<_, Voc>("SELECT * FROM vocs")
        .fetch_all(source)
        .await?;

    Ok(voc_records)
}

#[derive(Debug, sqlx::FromRow)]
struct VocInsert {
    id: i32,
    short: String,
}

async fn insert_vocs_old_new_map(
    vocs: Vec<Voc>,
    dest: &Pool<Postgres>,
) -> Result<HashMap<i64, i64>, sqlx::Error> {
    let values = vocs
        .iter()
        .map(|voc| format!("('{}', '{}')", voc.short, voc.full))
        .collect::<Vec<_>>()
        .join(",");

    let qry = format!(
        "INSERT INTO vocs(short, \"full\") 
         VALUES {}
         RETURNING id, short",
        values
    );

    let vocs_returning = query_as::<_, VocInsert>(&qry).fetch_all(dest).await?;

    let inserted_map = vec_to_map(&vocs_returning, |v| v.short.clone(), |v| v.id);

    let old_new_voc_id_map = vocs
        .iter()
        .map(|voc| (voc.id, inserted_map.get(&voc.short).unwrap().clone()))
        .collect::<Vec<_>>();

    Ok(vec_to_map(&old_new_voc_id_map, |x| x.0, |x| x.1 as i64))
}

pub fn opt_to_sql_value<T: Any + Display>(opt_value: Option<&T>) -> String {
    match opt_value {
        Some(value) => {
            let value_any = value as &dyn Any;

            if value_any.is::<String>() || value_any.is::<&str>() {
                //SQL injection protect
                format!("'{}'", value.to_string().replace("'", "''"))
            } else {
                value.to_string()
            }
        }
        None => "null".to_owned(),
    }
}

pub fn to_sql_value<T: Any + Display>(value: &T) -> String {
    opt_to_sql_value(Some(value))
}

async fn insert_word(
    word: &Word,
    vocs_old_new_map: &Arc<HashMap<i64, i64>>,
    source: &Arc<Pool<Postgres>>,
    dest: &Arc<Pool<Postgres>>,
) -> Result<(), sqlx::Error> {
    let source_word_definitions =
        query_as::<_, WordDefinition>("SELECT * FROM word_definitions WHERE word_id = $1")
            .bind(word.id)
            .fetch_all(&**source)
            .await?;

    let insert_word_qry = format!(
        r#"INSERT INTO words
                (body, status, is_edited_after_load, load_status, difficulty, "timestamp")
                VALUES('{}', '{}', {}, '{}', {}, {})
                RETURNING id"#,
        word.body,
        word.status,
        word.is_edited_after_load,
        word.load_status,
        word.difficulty,
        word.timestamp
    );

    let word_insert_row = query(&insert_word_qry).fetch_one(&**dest).await?;
    let word_id: i32 = word_insert_row.try_get("id").unwrap();

    if !source_word_definitions.is_empty() {
        let insert_definitions_qry = format!(
            r#"
            INSERT INTO word_definitions
            (definition, status, "order", word_id, voc_id)
            VALUES {}
            "#,
            source_word_definitions
                .iter()
                .map(|word_definition| {
                    format!(
                        "('{}', '{}', {}, {}, {})",
                        word_definition.definition,
                        word_definition.status,
                        word_definition.order,
                        word_id,
                        opt_to_sql_value(
                            word_definition
                                .voc_id
                                .and_then(|old_voc_id| vocs_old_new_map.get(&old_voc_id))
                        )
                    )
                })
                .collect::<Vec<_>>()
                .join(",")
        );

        query(&insert_definitions_qry).execute(&**dest).await?;
    }

    Ok(())
}

async fn load_words(
    vocs_old_new_map: Arc<HashMap<i64, i64>>,
    source: Arc<Pool<Postgres>>,
    dest: Arc<Pool<Postgres>>,
    chunk_size: usize,
) -> Result<(), sqlx::Error> {
    let source_words = query_as::<_, Word>("SELECT * FROM words")
        .fetch_all(&*source)
        .await?;

    let word_processed = Arc::new(Mutex::new(0));

    let tokie_spawns = source_words
        .chunks(chunk_size)
        .map(|words| {
            let arc_vocs_old_new_map = Arc::clone(&vocs_old_new_map);
            let arc_source = Arc::clone(&source);
            let arc_dest = Arc::clone(&dest);
            let word_processed = Arc::clone(&word_processed);
            let c_words = words.to_vec();
            tokio::spawn(async move {
                let mut results = vec![];
                for w in c_words {
                    results
                        .push(insert_word(&w, &arc_vocs_old_new_map, &arc_source, &arc_dest).await);
                    let mut current_word_processed = word_processed.lock().unwrap();

                    *current_word_processed += 1;
                }
                results
            })
        })
        .collect::<Vec<_>>();

    let word_count_to_process = source_words.len();
    let progress_thread = tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(100)).await;
            let current_word_processed = word_processed.lock().unwrap();
            println!("{}/{} {:.2}%", current_word_processed, 
                                     word_count_to_process,
                                     ((*current_word_processed as f32) / (word_count_to_process as f32) * 100.))
        }
    });

    let error: Option<Result<(), sqlx::Error>> = futures::future::join_all(tokie_spawns)
        .await
        .into_iter()
        .map(|join_result| join_result.unwrap())
        .flatten()
        .filter(|result| result.is_err())
        .next();

    progress_thread.abort();

    if let Some(e) = error {
        e
    } else {
        Ok(())
    }
}

async fn migrate(source: Pool<Postgres>, dest: Pool<Postgres>) -> Result<(), sqlx::Error> {
    let vocs = vocs_load_from_source(&source).await?;
    let arc_vocs_old_new_map = Arc::new(insert_vocs_old_new_map(vocs, &dest).await?);
    let arc_source = Arc::new(source);
    let arc_dest = Arc::new(dest);

    load_words(arc_vocs_old_new_map, arc_source, arc_dest, 1000).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let source_connection = create_connection_by_key("DATABASE_SOURCE").await.unwrap();
    let dest_connection = create_connection_by_key("DATABASE_DEST").await.unwrap();

    match migrate(source_connection, dest_connection).await {
        Ok(_) => println!("Done!"),
        Err(e) => println!("{}", e.to_string()),
    }
}
