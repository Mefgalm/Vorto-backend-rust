use crate::domain::voc::Voc;
use crate::{db::common::*, error::VortoResult};
use sqlx::{PgPool, query_as};


pub async fn get_by_shorts(shorts: &Vec<String>, pool: &PgPool) -> VortoResult<Vec<Voc>> {
    VortoResult::Ok(
        query_as::<_, Voc>(&format!(
            r#"
            SELECT * 
            FROM vocs
            WHERE {}
            "#, 
            in_qry("short", &shorts))
        )  
        .fetch_all(pool)
        .await?,
    )
}

