#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::response::Debug;
use rocket::serde::json::Json;
use rocket::tokio::task::{spawn_blocking, JoinError};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::env;

mod battlesnake;

/// # Get Info
///
/// Returns Battlesnake info for health validation, customization, and latency.
#[openapi(tag = "Battlesnake")]
#[get("/")]
fn handle_index() -> Json<battlesnake::Info> {
    Json(battlesnake::info())
}

/// # Game Start
///
/// This request is received when the Battlesnake has been entered into a new game.
#[openapi(tag = "Battlesnake")]
#[post("/start", format = "json", data = "<gs>")]
fn handle_start(gs: Json<battlesnake::GameState>) -> Status {
    battlesnake::start(gs.into_inner());
    Status::Ok
}

/// # Move
///
/// This request will be sent for every turn of the game. Use the information provided to determine how your Battlesnake will move on that turn, either up, down, left, or right.
#[openapi(tag = "Battlesnake")]
#[post("/move", format = "json", data = "<gs>")]
async fn handle_move(
    gs: Json<battlesnake::GameState>,
) -> Result<Json<battlesnake::MoveResponse>, Debug<JoinError>> {
    let result = spawn_blocking(move || Json(battlesnake::make_move(gs.into_inner()))).await?;
    Ok(result)
}

/// # Game End
///
/// Your Battlesnake will receive this request whenever a game it was playing has ended.
#[openapi(tag = "Battlesnake")]
#[post("/end", format = "json", data = "<gs>")]
fn handle_end(gs: Json<battlesnake::GameState>) -> Status {
    battlesnake::end(gs.into_inner());
    Status::Ok
}

/// # Ping
///
/// Returns a pong.
#[openapi(tag = "Health")]
#[get("/ping")]
fn handle_ping() -> &'static str {
    "pong"
}

#[launch]
fn launch() -> _ {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn,ndsquared_rustapi::battlesnake=info/.*");
    }
    env_logger::init();
    info!("LAUNCH");
    rocket::build()
        .mount(
            "/",
            openapi_get_routes![
                handle_index,
                handle_start,
                handle_move,
                handle_end,
                handle_ping
            ],
        )
        .mount(
            "/docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
