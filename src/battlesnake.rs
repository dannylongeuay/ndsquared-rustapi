use rand::seq::SliceRandom;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Customizations {
    /// Hex color code used to display this Battlesnake. Must start with "#" and be 7 characters long. Example: "#888888"
    color: String,
    /// Displayed head of this Battlesnake. Example: "default"
    head: String,
    /// Displayed tail of this Battlesnake. Example: "default"
    tail: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
enum Source {
    #[default]
    #[serde(rename = "")]
    Empty,
    Tournament,
    League,
    Arena,
    Challenge,
    Ladder,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
enum GameMode {
    Standard,
    Solo,
    Royale,
    Squad,
    Constrictor,
    Wrapped,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
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
    HzIslandsBridges,
    HzRiversBridges,
    HzSpiral,
    HzScatter,
    HzGrowBox,
    HzExpandBox,
    HzExpandScatter,
    HzCastleWall,
}

#[derive(Debug, EnumIter, Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoyaleSettings {
    /// The number of turns between generating new hazards (shrinking the safe board space).
    shrink_every_n_turns: u32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Ruleset {
    /// Name of the ruleset being used to run this game.
    name: GameMode,
    /// The release version of the Rules module used in this game. Example: "version": "v1.2.3"
    version: String,
    /// A collection of specific settings being used by the current game that control how the rules are applied.
    settings: RulesetSettings,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
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
pub struct PriorityCoord {
    coord: Coord,
    cost: u32,
}

impl Ord for PriorityCoord {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for PriorityCoord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
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
    #[serde(skip)]
    safe_adjacent_heads: HashSet<Coord>,
    #[serde(skip)]
    dangerous_adjacent_heads: HashSet<Coord>,
}

#[derive(Debug, PartialEq, Eq)]
enum StrategyName {
    Random,
    Starving,
    Survival,
    FollowMySelf,
    TerritoryControl,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Strategy {
    name: StrategyName,
    direction: Direction,
    cost: i32,
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

#[derive(Debug)]
pub struct TerritoryInfo {
    controlled_squares: HashMap<String, HashSet<Coord>>,
    food_count: HashMap<String, i32>,
    tail_count: HashMap<String, i32>,
    contains_our_tail: HashMap<String, bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
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

impl GameState {
    fn new_from_text(text: &str) -> Self {
        let mut height: i32 = 0;
        let mut width: i32 = 0;
        let mut y = 0;
        let mut snake_bodies: HashMap<char, Vec<(Coord, u32)>> = HashMap::new();
        let mut food: HashSet<Coord> = HashSet::new();
        let mut hazards: HashSet<Coord> = HashSet::new();
        for row in text.lines().map(str::trim).rev() {
            if !row.starts_with("|") {
                continue;
            }
            let mut x = 0;
            height += 1;
            let splits: Vec<&str> = row.trim_start_matches("|").split_terminator("|").collect();
            if width == 0 {
                width = splits.len() as i32;
            }
            for split in splits {
                let coord = Coord { x, y };
                let chars: Vec<char> = split.chars().collect();
                match chars[0] {
                    'H' => {
                        hazards.insert(coord);
                    }
                    'F' => {
                        food.insert(coord);
                    }
                    ' ' => {}
                    _ => {
                        let body_tuple = (coord, chars[1].to_string().parse().unwrap());
                        if let Some(bodies) = snake_bodies.get_mut(&chars[0]) {
                            bodies.push(body_tuple);
                        } else {
                            snake_bodies.insert(chars[0], vec![body_tuple]);
                        }
                    }
                }
                x += 1;
            }
            y += 1;
        }
        let customizations = Customizations {
            color: "color".to_owned(),
            head: "head".to_owned(),
            tail: "tail".to_owned(),
        };
        let mut snakes: Vec<Battlesnake> = Vec::new();
        let mut you: Option<Battlesnake> = None;
        for (owner, mut coords) in snake_bodies.clone() {
            coords.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let (body, _): (Vec<Coord>, Vec<u32>) = coords.iter().cloned().unzip();
            let length: u32 = body.len() as u32;
            let head = body[0];
            let snake = Battlesnake {
                id: owner.to_string(),
                name: "my_name".to_owned(),
                health: 100,
                body,
                latency: "100".to_owned(),
                head,
                length,
                shout: "shout!".to_owned(),
                squad: "squad".to_owned(),
                customizations: customizations.clone(),
            };
            if snake.id.clone() == "Y" {
                you = Some(snake.clone())
            }
            snakes.push(snake);
        }
        let squad = SquadSettings {
            allow_body_collisions: true,
            shared_elimination: true,
            shared_health: true,
            shared_length: true,
        };
        let royale = RoyaleSettings {
            shrink_every_n_turns: 0,
        };
        let settings = RulesetSettings {
            food_spawn_chance: 0,
            minimum_food: 0,
            hazard_damage_per_turn: 0,
            royale,
            squad,
        };
        let ruleset = Ruleset {
            name: GameMode::Standard,
            version: "0.1.0".to_owned(),
            settings,
        };
        let game = Game {
            id: "gameid".to_owned(),
            map: GameMap::Standard,
            ruleset,
            timeout: 0,
            source: Source::Custom,
        };
        let board = Board {
            height,
            width,
            food,
            hazards,
            snakes,
            obstacles: HashSet::new(),
            safe_tails: HashSet::new(),
            safe_adjacent_heads: HashSet::new(),
            dangerous_adjacent_heads: HashSet::new(),
        };
        let gs = GameState {
            game,
            turn: 0,
            board,
            you: you.unwrap(),
        };
        gs
    }
    fn new_adjacent_coord(&self, coord: &Coord, dir: &Direction) -> Coord {
        let mut x: i32 = coord.x;
        let mut y: i32 = coord.y;
        match self.game.ruleset.name {
            GameMode::Wrapped => {
                match dir {
                    Direction::Up => y += 1,
                    Direction::Down => y -= 1,
                    Direction::Left => x -= 1,
                    Direction::Right => x += 1,
                };
                x = i32::rem_euclid(x, self.board.width);
                y = i32::rem_euclid(y, self.board.height);
            }
            _ => {
                match dir {
                    Direction::Up => y += 1,
                    Direction::Down => y -= 1,
                    Direction::Left => x -= 1,
                    Direction::Right => x += 1,
                };
            }
        }
        Coord { x, y }
    }
    fn new_adjacent_coords(&self, coord: &Coord) -> Vec<Coord> {
        let mut adjacent_coords: Vec<Coord> = Vec::new();
        for direction in Direction::iter() {
            adjacent_coords.push(self.new_adjacent_coord(coord, &direction));
        }
        adjacent_coords
    }
    fn in_bounds(&self, coord: &Coord) -> bool {
        return coord.x >= 0
            && coord.y >= 0
            && coord.x < self.board.width
            && coord.y < self.board.height;
    }
    fn is_valid_at(&self, coord: &Coord) -> bool {
        self.in_bounds(coord)
    }
    fn is_safe_at(&self, coord: &Coord) -> bool {
        !self.board.obstacles.contains(coord)
    }
    fn is_valid_and_safe_at(&self, coord: &Coord) -> bool {
        self.is_valid_at(coord) && self.is_safe_at(coord)
    }
    fn init(&mut self) {
        self.compute_metadata();
    }
    fn compute_metadata(&mut self) {
        let mut obstacles: HashSet<Coord> = HashSet::new();
        for snake in &self.board.snakes {
            // Compute for all snakes
            for (i, body) in snake.body.iter().enumerate() {
                if i == snake.body.len() - 1 {
                    if !snake.just_ate() {
                        self.board.safe_tails.insert(body.clone());
                        continue;
                    } else {
                        debug!("{:?} just ate", snake);
                    }
                }
                obstacles.insert(body.clone());
            }
            // Compute for opponents only
            if snake.id == self.you.id {
                continue;
            }
            let adjacent_head_coords = self.new_adjacent_coords(&snake.head);
            if self.you.length <= snake.length {
                self.board
                    .dangerous_adjacent_heads
                    .extend(adjacent_head_coords);
            } else {
                self.board.safe_adjacent_heads.extend(adjacent_head_coords);
            }
        }
        obstacles.extend(self.board.hazards.clone());
        self.board.obstacles = obstacles;
    }
    fn get_random_valid_direction(&self, coord: &Coord) -> Direction {
        let mut valid_directions: Vec<Direction> = Vec::new();

        for direction in Direction::iter() {
            let adjacent_coord = self.new_adjacent_coord(coord, &direction);
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
    fn shortest_distance(&self, start: &Coord, end: &Coord) -> Option<u32> {
        let mut nodes: BinaryHeap<PriorityCoord> = BinaryHeap::new();
        let mut visited: HashSet<Coord> = HashSet::new();
        let mut costs: HashMap<Coord, u32> = HashMap::new();
        nodes.push(PriorityCoord {
            coord: start.clone(),
            cost: 0,
        });
        visited.insert(start.clone());
        costs.insert(start.clone(), 0);
        while let Some(PriorityCoord { coord, cost }) = nodes.pop() {
            if coord == *end {
                return Some(cost);
            }
            for adjacent_coord in self.new_adjacent_coords(&coord) {
                if !self.is_valid_and_safe_at(&adjacent_coord) {
                    continue;
                }
                if visited.contains(&adjacent_coord) {
                    continue;
                }
                let mut cost_mod = 5;
                if self
                    .board
                    .dangerous_adjacent_heads
                    .contains(&adjacent_coord)
                {
                    cost_mod += 5;
                } else if self.board.safe_adjacent_heads.contains(&adjacent_coord) {
                    cost_mod -= 4;
                }
                let new_cost = costs[&coord] + cost_mod;
                let adjacent_cost = costs.get(&adjacent_coord);
                if adjacent_cost == None || new_cost < *adjacent_cost.unwrap() {
                    costs.insert(adjacent_coord.clone(), new_cost);
                    visited.insert(adjacent_coord.clone());
                    nodes.push(PriorityCoord {
                        coord: adjacent_coord.clone(),
                        cost: new_cost,
                    })
                }
            }
        }
        None
    }
    fn closest_food_distance(&self, coord: &Coord) -> Option<u32> {
        let mut closest_distance: Option<u32> = None;
        for food in &self.board.food {
            if let Some(food_distance) = self.shortest_distance(coord, &food) {
                if closest_distance == None || food_distance < closest_distance.unwrap() {
                    closest_distance = Some(food_distance);
                }
            }
        }
        closest_distance
    }
    fn compute_territory_info(&self, exclusions: &HashSet<Coord>) -> TerritoryInfo {
        let mut controlled_squares: HashMap<String, HashSet<Coord>> = HashMap::new();
        let mut nodes: VecDeque<(String, u32, Coord)> = VecDeque::new();
        let mut visited: HashSet<Coord> = HashSet::new();
        visited.extend(exclusions);
        let mut food_count: HashMap<String, i32> = HashMap::new();
        let mut tail_count: HashMap<String, i32> = HashMap::new();
        let mut contains_our_tail: HashMap<String, bool> = HashMap::new();
        for snake in &self.board.snakes {
            controlled_squares.insert(snake.id.clone(), HashSet::new());
            food_count.insert(snake.id.clone(), 0);
            tail_count.insert(snake.id.clone(), 0);
            contains_our_tail.insert(snake.id.clone(), false);
            nodes.push_back((snake.id.clone(), 0, snake.head));
            visited.insert(snake.head);
            controlled_squares
                .get_mut(&snake.id)
                .unwrap()
                .insert(snake.head);
        }
        while !nodes.is_empty() {
            let (owner, distance, current_coord) = nodes.pop_front().unwrap();
            for adjacent_coord in self.new_adjacent_coords(&current_coord) {
                if !visited.contains(&adjacent_coord) && self.is_valid_and_safe_at(&adjacent_coord)
                {
                    let distance = distance + 1;
                    if self.board.food.contains(&adjacent_coord) {
                        food_count.insert(owner.clone(), food_count[&owner] + 1);
                    }
                    for snake in &self.board.snakes {
                        if *snake.body.last().unwrap() != adjacent_coord {
                            continue;
                        }
                        tail_count.insert(owner.clone(), tail_count[&owner] + 1);
                        if snake.id == owner {
                            contains_our_tail.insert(owner.clone(), true);
                        }
                    }
                    nodes.push_back((owner.clone(), distance, adjacent_coord));
                    visited.insert(adjacent_coord);
                    controlled_squares
                        .get_mut(&owner)
                        .unwrap()
                        .insert(adjacent_coord);
                }
            }
        }
        TerritoryInfo {
            controlled_squares,
            food_count,
            tail_count,
            contains_our_tail,
        }
    }
    fn find_strategies(&self) -> BinaryHeap<Strategy> {
        let mut strategies: BinaryHeap<Strategy> = BinaryHeap::new();

        // If all else fails do something reasonable and random
        strategies.push(Strategy {
            name: StrategyName::Random,
            direction: self.get_random_valid_direction(&self.you.head),
            cost: 10000,
        });

        for direction in Direction::iter() {
            let adjacent_coord = self.new_adjacent_coord(&self.you.head, &direction);
            let mut cost_mod: i32 = 0;
            if !self.is_valid_and_safe_at(&adjacent_coord) {
                info!("Direction {:?} is not safe", direction);
                continue;
            }
            // Handle head collisions
            if self
                .board
                .dangerous_adjacent_heads
                .contains(&adjacent_coord)
            {
                info!("Direction {:?} is dangerous", direction);
                cost_mod += 25;
            } else if self.board.safe_adjacent_heads.contains(&adjacent_coord) {
                info!("Direction {:?} is appetizing", direction);
                cost_mod -= 25;
            }
            let exclusions = self
                .new_adjacent_coords(&self.you.head)
                .iter()
                .cloned()
                .filter(|&coord| coord != adjacent_coord)
                .collect();
            let territory_info = self.compute_territory_info(&exclusions);
            let controlled_squares = territory_info.controlled_squares[&self.you.id].len() as i32;
            let food_count = territory_info.food_count[&self.you.id];
            let tail_count = territory_info.tail_count[&self.you.id];
            if let Some(food_distance) = self.closest_food_distance(&adjacent_coord) {
                // Eat food in our territory
                if territory_info.food_count[&self.you.id] > 0
                    && controlled_squares as u32 > self.you.length + 1
                {
                    strategies.push(Strategy {
                        name: StrategyName::TerritoryControl,
                        direction,
                        cost: 1 - controlled_squares - food_count * 3
                            + food_distance as i32
                            + cost_mod,
                    });
                }
                // Eat any food when we get hungry enough, even if the food is not in our controlled area
                if self.you.health <= 33 {
                    strategies.push(Strategy {
                        name: StrategyName::Starving,
                        direction,
                        cost: 250 - controlled_squares + food_distance as i32 + cost_mod,
                    });
                }
            }
            // Follow our own tail when no food is in our controlled area and we aren't hungry yet
            if let Some(my_tail_distance) =
                self.shortest_distance(&adjacent_coord, &self.you.body.last().unwrap())
            {
                if my_tail_distance > 1 || !self.board.food.contains(&adjacent_coord) {
                    strategies.push(Strategy {
                        name: StrategyName::FollowMySelf,
                        direction,
                        cost: 500 + my_tail_distance as i32 + cost_mod,
                    });
                }
            }
            info!(
                "Direction {:?} controls {:?} square(s)",
                direction, controlled_squares
            );
            // Move towards an area with the most squares available if we can't eat or follow our tail
            if territory_info.contains_our_tail[&self.you.id] {
                cost_mod -= 50;
            }
            strategies.push(Strategy {
                name: StrategyName::Survival,
                direction,
                cost: 1000 - controlled_squares - food_count * 5 - tail_count * 3 + cost_mod,
            });
        }
        strategies
    }
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
    let customizations = Customizations {
        color: "#6434eb".to_owned(),
        head: "pixel".to_owned(),
        tail: "pixel".to_owned(),
    };

    let result = Info {
        apiversion: "1".to_owned(),
        author: "DeanRefined".to_owned(),
        customizations,
        version: "1.9.0".to_owned(),
    };

    info!("{:?}", result);

    result
}

pub fn make_move(mut gs: GameState) -> MoveResponse {
    info!(
        "########## TURN {:?} | {:?} ##########",
        gs.turn, gs.you.name
    );
    gs.init();

    let mut strategies = gs.find_strategies();

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

pub fn start(gs: GameState) {
    info!("START: {:?}", gs);
}

pub fn end(gs: GameState) {
    info!("END: {:?}", gs);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_new_from_text() {
        let gs = GameState::new_from_text(
            "
        |  |  |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |  |  |        
        ",
        );
        assert_eq!(gs.you.length, 3);
        assert_eq!(gs.board.width, 5);
        assert_eq!(gs.board.height, 5);
        assert_eq!(gs.you.body.contains(&Coord { x: 1, y: 2 }), true);
        assert_eq!(gs.you.head, Coord { x: 1, y: 3 });
        assert_eq!(*gs.you.body.last().unwrap(), Coord { x: 1, y: 1 });
        for snake in &gs.board.snakes {
            if snake.id != "A" {
                continue;
            }
            assert_eq!(snake.body.contains(&Coord { x: 3, y: 2 }), true);
            assert_eq!(snake.head, Coord { x: 3, y: 1 });
            assert_eq!(*snake.body.last().unwrap(), Coord { x: 3, y: 3 });
        }
        assert_eq!(gs.board.food.contains(&Coord { x: 2, y: 0 }), true);
        assert_eq!(gs.board.hazards.contains(&Coord { x: 4, y: 4 }), true);
    }
}
