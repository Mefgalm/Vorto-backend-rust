#![feature(proc_macro_hygiene, decl_macro, try_trait_v2, never_type)]

mod auth;
mod common;
mod db;
mod domain;
mod error;
mod requests;
mod responses;
mod routes;
mod seed;
mod services;
mod states;

use std::env;

use dotenv::dotenv;
use error::VortoResult;
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket::http::{Header};
use rocket::serde::json::Json;
use routes::*;
use services::wiki_parser_service;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate lazy_static;

extern crate dotenv;

#[macro_use]
extern crate log;

use crate::services::password_hasher::PwdHasher;
use crate::states::*;
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde::Serialize;
use v1;

impl<'r, T: Serialize> Responder<'r, 'r> for VortoResult<T> {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        let response_value = match self {
            VortoResult::Ok(data) => serde_json::json!({
                "code": 0,
                "message": null,
                "data": data
            }),
            VortoResult::Err(vorto_error) => serde_json::json!({
                "code": vorto_error.code,
                "message": vorto_error.message,
                "data": null
            }),
        };
        Json(response_value).respond_to(request)
    }
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PUT, GET, PATCH, OPTIONS",
        ));
        res.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));

        if req.method() == rocket::http::Method::Options {
            res.set_status(rocket::http::Status::NoContent);
        }
    }
}

#[tokio::main]
async fn main() {
    // Unsure settings loaded
    dotenv().ok();
    env_logger::init();

    rocket::build()
        .attach(CORS)
        .attach(AdHoc::on_liftoff("Seed DB", |rocket| {
            Box::pin(async move { seed::run(rocket).await })
        }))
        .mount(
            "/api/v1/admin",
            routes![
                v1::admin::users::sign_in,
                v1::admin::words::search,
                v1::admin::words::load_definitions,
                v1::admin::words::update,
                v1::admin::words::word_stats,
            ],
        )
        .mount(
            "/api/v1",
            routes![
                v1::ping::pong,
                v1::teams::get_teams,
                v1::vocs::get_vocs,
                v1::game::create,
                v1::game::complete_round,
                v1::game::game_view
            ],
        )
        .manage(pg_sqlx_conect().await)
        .manage(PwdHasher::new())
        .launch()
        .await
        .unwrap();
}
