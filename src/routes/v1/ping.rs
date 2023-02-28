#[get("/ping")]
pub fn pong() -> String {
    "pong".to_owned()
}