use chrono::Utc;
use futures::future::join_all;
use rocket::Orbit;
use sqlx::{query, PgPool};

use crate::{db, services::password_hasher::PwdHasher};

async fn insert_admin(rocket: &rocket::Rocket<Orbit>, pool: &PgPool) {
    let pwd_hasher = rocket
        .state::<PwdHasher>()
        .expect("pwd hasher is not registered");

    let password_hash = pwd_hasher
        .hash_password("123456")
        .expect("Password hash failed")
        .to_owned();

    let admin_create_result = query!(
        "INSERT INTO users(email, password_hash, created_at)
        VALUES ($1, $2, $3)",
        "admin@mail.com",
        password_hash,
        Utc::now().naive_utc()
    )
    .execute(pool)
    .await;

    match admin_create_result {
        Ok(_) => info!("Admin created"),
        Err(_) => info!("Admin already created"),
    }
}

async fn insert_teams(pool: &PgPool) {
    let inserts = [
        "Вертолеты",
        "Крысы",
        "Боевые вертолёты",
        "Мозамбийские девочки",
    ]
    .iter()
    .map(|team_name| db::team::insert(team_name, pool));

    join_all(inserts).await;
}

pub async fn run(rocket: &rocket::Rocket<Orbit>) {
    let pool = rocket.state::<PgPool>().expect("Pg poll not found");
    insert_admin(rocket, pool).await;
    insert_teams(pool).await;
}
