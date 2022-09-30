use rand::seq::SliceRandom;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, EnumIter, Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Coord {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CoordItem {
    coord: Coord,
    cost: u32,
}

impl Ord for CoordItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for CoordItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Coord {
    fn new_adjacent_coord(&self, dir: &Direction) -> Coord {
        let mut x: i32 = self.x;
        let mut y: i32 = self.y;
        match dir {
            Direction::Up => y += 1,
            Direction::Down => y -= 1,
            Direction::Left => x -= 1,
            Direction::Right => x += 1,
        };
        Coord { x, y }
    }
    fn new_adjacent_coords(&self) -> Vec<Coord> {
        let mut adjacent_coords: Vec<Coord> = Vec::new();
        for direction in Direction::iter() {
            adjacent_coords.push(self.new_adjacent_coord(&direction));
        }
        adjacent_coords
    }
    // fn direction_to(&self, other: &Coord) -> Direction {
    //     match (self.x - other.x, self.y - other.y) {
    //         (0, -1) => Direction::Up,
    //         (0, 1) => Direction::Down,
    //         (-1, 0) => Direction::Right,
    //         (1, 0) => Direction::Left,
    //         _ => {
    //             error!("could not get direction from {:?} to {:?}", self, other);
    //             Direction::Down
    //         }
    //     }
    // }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Board {
    /// The number of rows in the y-axis of the game board. Example: 11
    height: i32,
    /// The number of columns in the x-axis of the game board. Example: 11
    width: i32,
    /// Array of coordinates representing food locations on the game board. Example: [{"x": 5, "y": 5}, ..., {"x": 2, "y": 6}]
    food: HashSet<Coord>,
    /// Array of coordinates representing hazardous locations on the game board. These will only appear in some game modes. Example: [{"x": 0, "y": 0}, ..., {"x": 0, "y": 1}]
    hazards: HashSet<Coord>,
    /// Array of Battlesnake Objects representing all Battlesnakes remaining on the game board (including yourself if you haven't been eliminated). Example: [{"id": "snake-one", ...}, ...]
    snakes: Vec<Battlesnake>,
    /// User Defined
    #[serde(skip)]
    obstacles: HashSet<Coord>,
    #[serde(skip)]
    safe_tails: HashSet<Coord>,
}

#[derive(Debug, PartialEq, Eq)]
enum StrategyName {
    Random,
    Survival,
    FollowMySelf,
    FollowAFriend,
    NomNom,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Strategy {
    name: StrategyName,
    direction: Direction,
    cost: u32,
}

impl Ord for Strategy {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.cost < self.cost {
            return Ordering::Less;
        } else if other.cost > self.cost {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

impl PartialOrd for Strategy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct Square {
    owner: String,
    coord: Coord,
    distance: u32,
}

#[derive(Debug)]
pub struct ControlledSquares {
    squares: HashMap<String, HashMap<Coord, Square>>,
    closest_food_distance: HashMap<String, Option<u32>>,
    closest_tail_distance: HashMap<String, Option<u32>>,
}

impl Board {
    fn in_bounds(&self, coord: &Coord) -> bool {
        return coord.x >= 0 && coord.y >= 0 && coord.x < self.width && coord.y < self.height;
    }
    fn is_valid_at(&self, coord: &Coord) -> bool {
        self.in_bounds(coord)
    }
    fn is_safe_at(&self, coord: &Coord) -> bool {
        !self.obstacles.contains(coord)
    }
    fn is_valid_and_safe_at(&self, coord: &Coord) -> bool {
        self.is_valid_at(coord) && self.is_safe_at(coord)
    }
    fn compute_obstacles(&mut self) {
        let mut obstacles: HashSet<Coord> = HashSet::new();
        for snake in &self.snakes {
            for (i, body) in snake.body.iter().enumerate() {
                if i == snake.body.len() - 1 {
                    if !snake.just_ate() {
                        self.safe_tails.insert(body.clone());
                        continue;
                    } else {
                        debug!("{:?} just ate", snake);
                    }
                }
                obstacles.insert(body.clone());
            }
        }
        obstacles.extend(self.hazards.clone());
        self.obstacles = obstacles;
    }
    fn get_random_valid_direction(&self, coord: &Coord) -> Direction {
        let mut valid_directions: Vec<Direction> = Vec::new();

        for direction in Direction::iter() {
            let adjacent_coord = coord.new_adjacent_coord(&direction);
            if self.is_valid_and_safe_at(&adjacent_coord) {
                valid_directions.push(direction);
            }
        }

        // Default direction if no valid direction is found
        let mut random_direction: Direction = Direction::Down;

        if valid_directions.len() > 0 {
            random_direction = *valid_directions.choose(&mut rand::thread_rng()).unwrap();
        }
        random_direction
    }
    fn astar(&self, start: &Coord, end: &Coord) -> Option<u32> {
        let mut nodes = BinaryHeap::new();
        let mut visited: HashSet<Coord> = HashSet::new();
        let mut costs: HashMap<Coord, u32> = HashMap::new();
        nodes.push(CoordItem {
            coord: start.clone(),
            cost: 0,
        });
        visited.insert(start.clone());
        costs.insert(start.clone(), 0);
        while let Some(CoordItem { coord, cost }) = nodes.pop() {
            if coord == *end {
                info!("{:?}", costs);
                return Some(cost);
            }
            for adjacent_coord in coord.new_adjacent_coords() {
                if !self.is_valid_and_safe_at(&adjacent_coord) {
                    continue;
                }
                if visited.contains(&adjacent_coord) {
                    continue;
                }
                let new_cost = costs[&coord] + 1;
                let adjacent_cost = costs.get(&adjacent_coord);
                if adjacent_cost == None || new_cost < *adjacent_cost.unwrap() {
                    costs.insert(adjacent_coord.clone(), new_cost);
                    visited.insert(adjacent_coord.clone());
                    nodes.push(CoordItem {
                        coord: adjacent_coord.clone(),
                        cost: new_cost,
                    })
                }
            }
        }
        info!("{:?}", costs);
        None
    }
    fn compute_controlled_squares(&self, exclusions: &HashSet<Coord>) -> ControlledSquares {
        let mut squares: HashMap<String, HashMap<Coord, Square>> = HashMap::new();
        let mut nodes: VecDeque<Square> = VecDeque::new();
        let mut visited: HashSet<Coord> = HashSet::new();
        visited.extend(exclusions);
        let mut paths: HashMap<String, HashMap<Coord, Coord>> = HashMap::new();
        let mut closest_food_distance: HashMap<String, Option<u32>> = HashMap::new();
        let mut closest_tail_distance: HashMap<String, Option<u32>> = HashMap::new();
        for snake in &self.snakes {
            squares.insert(snake.id.clone(), HashMap::new());
            paths.insert(snake.id.clone(), HashMap::new());
            closest_food_distance.insert(snake.id.clone(), None);
            closest_tail_distance.insert(snake.id.clone(), None);
            let square = Square {
                owner: snake.id.clone(),
                coord: snake.head,
                distance: 0,
            };
            nodes.push_back(square.clone());
            visited.insert(snake.head);
            squares
                .get_mut(&snake.id)
                .unwrap()
                .insert(snake.head, square.clone());
        }
        while !nodes.is_empty() {
            let current_square = nodes.pop_front().unwrap();
            for coord in current_square.coord.new_adjacent_coords() {
                if !visited.contains(&coord) && self.is_valid_and_safe_at(&coord) {
                    let current_square =
                        squares[&current_square.owner][&current_square.coord].clone();
                    let distance = current_square.distance + 1;
                    let food = self.food.contains(&coord);
                    let square = Square {
                        owner: current_square.owner.clone(),
                        coord,
                        distance,
                    };
                    if closest_food_distance[&current_square.owner] == None && food {
                        closest_food_distance.insert(current_square.owner.clone(), Some(distance));
                    }
                    if closest_tail_distance[&current_square.owner] == None
                        && self.safe_tails.contains(&coord)
                    {
                        closest_tail_distance.insert(current_square.owner.clone(), Some(distance));
                    }
                    nodes.push_back(square.clone());
                    visited.insert(coord);
                    paths
                        .get_mut(&current_square.owner)
                        .unwrap()
                        .insert(coord, current_square.coord);
                    squares
                        .get_mut(&current_square.owner)
                        .unwrap()
                        .insert(coord, square.clone());
                }
            }
        }
        ControlledSquares {
            squares,
            closest_food_distance,
            closest_tail_distance,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

impl Battlesnake {
    fn just_ate(&self) -> bool {
        if self.body.len() < 2 {
            return false;
        }
        self.body[self.body.len() - 1] == self.body[self.body.len() - 2]
    }
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
        version: "1.6.1".to_owned(),
    }
}

pub fn make_move(mut gs: GameState) -> MoveResponse {
    info!(
        "########## TURN {:?} | {:?} ##########",
        gs.turn, gs.you.name
    );

    gs.board.compute_obstacles();

    let mut strategies = BinaryHeap::new();

    strategies.push(Strategy {
        name: StrategyName::Random,
        direction: gs.board.get_random_valid_direction(&gs.you.head),
        cost: 10000,
    });

    let mut closest_food_distance = u32::MAX;
    let mut closest_tail_distance = u32::MAX;

    for direction in Direction::iter() {
        let mut exclusions: HashSet<Coord> = HashSet::new();
        let adjacent_coord = gs.you.head.new_adjacent_coord(&direction);
        // TODO: handle head on collisions
        if !gs.board.is_valid_and_safe_at(&adjacent_coord) {
            info!("Direction {:?} is not safe", direction);
            continue;
        }
        if let Some(my_tail_distance) = gs
            .board
            .astar(&adjacent_coord, &gs.you.body.last().unwrap())
        {
            if my_tail_distance > 1 || !gs.board.food.contains(&adjacent_coord) {
                strategies.push(Strategy {
                    name: StrategyName::FollowMySelf,
                    direction,
                    cost: 500 + my_tail_distance,
                });
            }
        }
        match direction {
            Direction::Up => {
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Down));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Left));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Right));
            }
            Direction::Down => {
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Up));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Left));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Right));
            }
            Direction::Left => {
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Up));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Down));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Right));
            }
            Direction::Right => {
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Up));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Down));
                exclusions.insert(gs.you.head.new_adjacent_coord(&Direction::Left));
            }
        };
        let squares = gs.board.compute_controlled_squares(&exclusions);
        let squares_count = squares.squares[&gs.you.id].len();
        info!(
            "Direction {:?} controls {:?} square(s)",
            direction, squares_count
        );
        strategies.push(Strategy {
            name: StrategyName::Survival,
            direction,
            cost: 1000 - squares_count as u32,
        });
        if let Some(direction_food_distance) = squares.closest_food_distance[&gs.you.id] {
            if squares_count as u32 > gs.you.length + 1
                && direction_food_distance < closest_food_distance
            {
                strategies.push(Strategy {
                    name: StrategyName::NomNom,
                    direction,
                    cost: 1 + direction_food_distance,
                });
                closest_food_distance = direction_food_distance;
            }
        }
        if let Some(direction_tail_distance) = squares.closest_tail_distance[&gs.you.id] {
            if direction_tail_distance < closest_tail_distance {
                strategies.push(Strategy {
                    name: StrategyName::FollowAFriend,
                    direction,
                    cost: 5000 + direction_tail_distance,
                });
                closest_tail_distance = direction_tail_distance;
            }
        }
    }

    let strategy = strategies.pop().unwrap();

    let mr = MoveResponse {
        direction: strategy.direction,
        shout: format!(
            "STRATEGY: {:?} | COST: {:?} | MOVE: {:?}",
            strategy.name, strategy.cost, strategy.direction
        ),
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
