#[macro_use]
extern crate rocket;

use log::info;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::env;

mod battlesnake;

/// # Get Info
///
/// Returns Battlesnake info for health validation, customization, and latency.
#[openapi(tag = "Battlesnake")]
#[get("/")]
fn handle_index() -> Json<battlesnake::Info> {
    info!("HANDLE INDEX");
    Json(battlesnake::info())
}

/// # Game Start
///
/// This request is received when the Battlesnake has been entered into a new game.
#[openapi(tag = "Battlesnake")]
#[post("/start", format = "json", data = "<_gs>")]
fn handle_start(_gs: Json<battlesnake::GameState>) -> Status {
    info!("HANDLE START");
    battlesnake::start();
    Status::Ok
}

/// # Move
///
/// This request will be sent for every turn of the game.
#[openapi(tag = "Battlesnake")]
#[post("/move", format = "json", data = "<gs>")]
fn handle_move(gs: Json<battlesnake::GameState>) -> Json<battlesnake::MoveResponse> {
    info!("HANDLE MOVE");
    Json(battlesnake::make_move(gs.into_inner()))
}

/// # Game End
///
/// Your Battlesnake will receive this request whenever a game it was playing has ended.
#[openapi(tag = "Battlesnake")]
#[post("/end", format = "json", data = "<_gs>")]
fn handle_end(_gs: Json<battlesnake::GameState>) -> Status {
    info!("HANDLE END");
    battlesnake::end();
    Status::Ok
}

#[launch]
fn launch() -> _ {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    info!("LAUNCH");
    rocket::build()
        .mount(
            "/",
            openapi_get_routes![handle_index, handle_start, handle_move, handle_end],
        )
        .mount(
            "/docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
