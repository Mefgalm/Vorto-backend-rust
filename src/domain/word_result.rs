#[derive(Debug, Clone)]
pub struct WordResult {
    pub id: i32,
    pub result: bool,
    pub order: i32,
    pub word_id: i32,
    pub team_result_id: i32
}

pub fn new(id: i32, result: bool, order: i32, word_id: i32, team_result_id: i32) -> WordResult {
    WordResult {
        id,
        result,
        word_id,
        team_result_id,
        order
    }
}