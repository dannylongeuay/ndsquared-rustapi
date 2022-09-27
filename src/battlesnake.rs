use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Info {
    /// Version of the Battlesnake API implemented by this Battlesnake. Currently only API version 1 is valid. Example: "1"
    apiversion: String,
    /// Username of the author of this Battlesnake. If provided, this will be used to verify ownership. Example: "BattlesnakeOfficial"
    author: String,
    /// Hex color code used to display this Battlesnake. Must start with "#" and be 7 characters long. Example: "#888888"
    color: String,
    /// Displayed head of this Battlesnake. Example: "default"
    head: String,
    /// Displayed tail of this Battlesnake. Example: "default"
    tail: String,
    /// A version number or tag for your snake.
    version: String,
}

pub fn info() -> Info {
    info!("INFO");

    Info {
        apiversion: "1".to_owned(),
        author: "DeanRefined".to_owned(),
        color: "#00ccff".to_owned(),
        head: "default".to_owned(),
        tail: "default".to_owned(),
        version: "0.0.1".to_owned(),
    }
}
