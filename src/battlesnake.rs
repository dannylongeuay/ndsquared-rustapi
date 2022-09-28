use rand::seq::SliceRandom;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Customizations {
    /// Hex color code used to display this Battlesnake. Must start with "#" and be 7 characters long. Example: "#888888"
    color: String,
    /// Displayed head of this Battlesnake. Example: "default"
    head: String,
    /// Displayed tail of this Battlesnake. Example: "default"
    tail: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Info {
    /// Version of the Battlesnake API implemented by this Battlesnake. Currently only API version 1 is valid. Example: "1"
    apiversion: String,
    /// Username of the author of this Battlesnake. If provided, this will be used to verify ownership. Example: "BattlesnakeOfficial"
    author: String,
    /// The collection of customizations applied to this Battlesnake that represent how it is viewed.
    #[serde(flatten)]
    customizations: Customizations,
    /// A version number or tag for your snake.
    version: String,
}

#[derive(Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Source {
    #[default]
    #[serde(rename = "")]
    Empty,
    Tournament,
    League,
    Arena,
    Challenge,
    Custom,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum GameMode {
    Standard,
    Solo,
    Royale,
    Squad,
    Constrictor,
    Wrapped,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum GameMap {
    Standard,
    Empty,
    ArcadeMaze,
    Royale,
    SoloMaze,
    HzInnerWall,
    HzRings,
    HzColumns,
    HzRiversBridges,
    HzSpiral,
    HzScatter,
    HzGrowBox,
    HzExpandBox,
    HzExpandScatter,
}

#[derive(Debug, EnumIter, Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoyaleSettings {
    /// The number of turns between generating new hazards (shrinking the safe board space).
    shrink_every_n_turns: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SquadSettings {
    /// Allow members of the same squad to move over each other without dying.
    allow_body_collisions: bool,
    /// All squad members are eliminated when one is eliminated.
    shared_elimination: bool,
    /// All squad members share health.
    shared_health: bool,
    /// All squad members share length.
    shared_length: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RulesetSettings {
    /// Percentage chance of spawning a new food every round.
    food_spawn_chance: u32,
    /// Minimum food to keep on the board every turn.
    minimum_food: u32,
    /// Health damage a snake will take when ending its turn in a hazard. This stacks on top of the regular 1 damage a snake takes per turn.
    hazard_damage_per_turn: u32,
    /// Royale game mode specific settings.
    royale: RoyaleSettings,
    /// Squad game mode specific settings.
    squad: SquadSettings,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Ruleset {
    /// Name of the ruleset being used to run this game.
    name: GameMode,
    /// The release version of the Rules module used in this game. Example: "version": "v1.2.3"
    version: String,
    /// A collection of specific settings being used by the current game that control how the rules are applied.
    settings: RulesetSettings,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Game {
    /// A unique identifier for this Game. Example: "totally-unique-game-id"
    id: String,
    /// Information about the ruleset being used to run this game. Example: {"name": "standard", "version": "v1.2.3"}
    ruleset: Ruleset,
    /// The name of the map used to populate the game board with snakes, food, and hazards. Example: "standard"
    map: GameMap,
    /// How much time your snake has to respond to requests for this Game. Example: 500
    timeout: u32,
    /// The source of this game.
    #[serde(default)]
    source: Source,
}

#[derive(PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn adjacent(&self, dir: &Direction, board: &Board) -> Option<Coord> {
        let mut x: i32 = self.x;
        let mut y: i32 = self.y;
        match dir {
            Direction::Up => y += 1,
            Direction::Down => y -= 1,
            Direction::Left => x -= 1,
            Direction::Right => x += 1,
        };
        let adjacent_coord = Coord { x, y };
        if !board.in_bounds(&adjacent_coord) {
            None
        } else {
            Some(adjacent_coord)
        }
    }
    fn valid_adjacent(&self, dir: &Direction, board: &Board) -> Option<Coord> {
        if let Some(adjacent_coord) = self.adjacent(dir, board) {
            if board.obstacles().contains(&&adjacent_coord) {
                return None;
            }
            Some(adjacent_coord)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Board {
    /// The number of rows in the y-axis of the game board. Example: 11
    height: i32,
    /// The number of columns in the x-axis of the game board. Example: 11
    width: i32,
    /// Array of coordinates representing food locations on the game board. Example: [{"x": 5, "y": 5}, ..., {"x": 2, "y": 6}]
    food: Vec<Coord>,
    /// Array of coordinates representing hazardous locations on the game board. These will only appear in some game modes. Example: [{"x": 0, "y": 0}, ..., {"x": 0, "y": 1}]
    hazards: Vec<Coord>,
    /// Array of Battlesnake Objects representing all Battlesnakes remaining on the game board (including yourself if you haven't been eliminated). Example: [{"id": "snake-one", ...}, ...]
    snakes: Vec<Battlesnake>,
}

impl Board {
    fn in_bounds(&self, coord: &Coord) -> bool {
        return coord.x >= 0 && coord.y >= 0 && coord.x < self.width && coord.y < self.height;
    }
    fn obstacles(&self) -> Vec<&Coord> {
        let mut result: Vec<&Coord> = Vec::new();
        for snake in &self.snakes {
            for body in &snake.body {
                result.push(body);
            }
        }
        result.extend(&self.hazards);
        result
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Battlesnake {
    /// Unique identifier for this Battlesnake in the context of the current Game. Example: "totally-unique-snake-id"
    id: String,
    /// Name given to this Battlesnake by its author. Example: "Sneky McSnek Face"
    name: String,
    /// Health value of this Battlesnake, between 0 and 100 inclusively. Example: 54
    health: u32,
    /// Array of coordinates representing this Battlesnake's location on the game board. This array is ordered from head to tail. Example: [{"x": 0, "y": 0}, ..., {"x": 2, "y": 0}]
    body: Vec<Coord>,
    /// The previous response time of this Battlesnake, in milliseconds. If the Battlesnake timed out and failed to respond, the game timeout will be returned (game.timeout) Example: "500"
    latency: String,
    /// Coordinates for this Battlesnake's head. Equivalent to the first element of the body array. Example: {"x": 0, "y": 0}
    head: Coord,
    /// Length of this Battlesnake from head to tail. Equivalent to the length of the body array. Example: 3
    length: u32,
    /// Message shouted by this Battlesnake on the previous turn. Example: "why are we shouting??"
    shout: String,
    /// The squad that the Battlesnake belongs to. Used to identify squad members in Squad Mode games. Example: "1"
    squad: String,
    /// The collection of customizations applied to this Battlesnake that represent how it is viewed.
    customizations: Customizations,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameState {
    /// Game Object describing the game being played.
    game: Game,
    /// Turn number of the game being played (0 for new games).
    turn: u32,
    /// Board Object describing the initial state of the game board.
    board: Board,
    /// Battlesnake Object describing your Battlesnake.
    you: Battlesnake,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MoveResponse {
    /// Your Battlesnake's move for this turn. Valid moves are up, down, left, or right. Example: "up"
    #[serde(rename = "move")]
    direction: Direction,
    /// An optional message sent to all other Battlesnakes on the next turn. Must be 256 characters or less. Example: "I am moving up!"
    shout: String,
}

pub fn info() -> Info {
    info!("INFO");

    let customizations = Customizations {
        color: "#00ccff".to_owned(),
        head: "default".to_owned(),
        tail: "default".to_owned(),
    };

    Info {
        apiversion: "1".to_owned(),
        author: "DeanRefined".to_owned(),
        customizations,
        version: "1.5.1".to_owned(),
    }
}

pub fn make_move(gs: GameState) -> MoveResponse {
    info!("MOVE");

    let mut valid_directions: Vec<Direction> = Vec::new();
    for direction in Direction::iter() {
        if let Some(_) = gs.you.head.valid_adjacent(&direction, &gs.board) {
            valid_directions.push(direction);
        }
    }

    let mut chosen_direction: Direction = Direction::Up;

    if valid_directions.len() > 0 {
        chosen_direction = *valid_directions.choose(&mut rand::thread_rng()).unwrap();
    }

    let mr = MoveResponse {
        direction: chosen_direction,
        shout: format!("Moving {:?}!", chosen_direction),
    };

    info!("{:?}", mr);

    mr
}

pub fn start() {
    info!("START");
}

pub fn end() {
    info!("END");
}
