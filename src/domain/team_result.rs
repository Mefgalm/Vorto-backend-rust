use crate::error::VortoResult;

#[derive(Debug, Clone)]
pub struct TeamResult {
    pub id: i32,
    pub team_id: i32,
    pub game_id: i32,
    pub order: i32
}

pub fn new(id: i32, team_id: i32, game_id: i32, order: i32) -> VortoResult<TeamResult> {

    VortoResult::Ok(TeamResult {
        id,
        team_id,
        game_id,
        order,
    })
}