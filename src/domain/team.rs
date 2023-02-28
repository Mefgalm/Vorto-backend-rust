
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String
}