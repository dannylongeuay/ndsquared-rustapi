#[macro_use]
extern crate rocket;

use log::info;
use rocket::serde::json::Json;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use std::env;

mod battlesnake;

/// # Get info
///
/// Returns Battlesnake info
#[openapi(tag = "Battlesnake")]
#[get("/")]
fn index() -> Json<battlesnake::Info> {
    info!("INDEX");
    Json(battlesnake::info())
}

#[launch]
fn launch() -> _ {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    info!("LAUNCH");
    rocket::build()
        .mount("/", openapi_get_routes![index])
        .mount(
            "/docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
