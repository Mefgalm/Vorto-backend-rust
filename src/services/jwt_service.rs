use chrono::{DateTime, Duration, Utc};
use jsonwebtokens::{encode, Algorithm, AlgorithmID, Verifier};
use serde_json::json;

use crate::error::VortoResult;

const TOKEN_VALID_DAYS: i64 = 30;

fn get_algorithm() -> VortoResult<Algorithm> {
    let alg = Algorithm::new_hmac(
        AlgorithmID::HS256,
        std::env::var("JWT_SECRET").expect("JWT_SECRET cannot be empty"),
    )?;
    VortoResult::Ok(alg)
}

pub fn generate_token(now: DateTime<Utc>) -> VortoResult<String> {
    let alg = get_algorithm()?;
    let header = json!({
        "alg": alg.name(),
        "iat": now.timestamp(),
        "exp": now + Duration::days(TOKEN_VALID_DAYS)
    });
    let claims = json!({});
    let token = encode(&header, &claims, &alg)?;

    VortoResult::Ok(token)
}

pub fn verify(token: &str) -> VortoResult<()> {
    let alg = get_algorithm()?;
    let verifier = Verifier::create().build()?;

    let _ = verifier.verify(token, &alg)?;
    VortoResult::Ok(())
}
