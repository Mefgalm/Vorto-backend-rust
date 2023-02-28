#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Voc {
    pub id: i32,
    pub short: String,
    pub full: String
}
