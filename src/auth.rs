use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::error::VortoResult;
use crate::services::jwt_service;

pub struct Admin;

fn get_jwt_token<'r>(request: &'r Request<'_>) -> Option<String> {
    request
        .headers()
        .get_one("Authorization")
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        })
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(jwt_token) = get_jwt_token(request) {
            if let VortoResult::Ok(_) = jwt_service::verify(&jwt_token) {
                Outcome::Success(Admin {})
            } else {
                Outcome::Failure((Status::Unauthorized, ()))
            }
        } else {
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
