use rand::seq::SliceRandom;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::time::Instant;
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
    hazard_damage_per_turn: i32,
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
    health: i32,
    /// Array of coordinates representing this Battlesnake's location on the game board. This array is ordered from head to tail. Example: [{"x": 0, "y": 0}, ..., {"x": 2, "y": 0}]
    body: VecDeque<Coord>,
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

#[derive(Debug)]
pub struct MinimaxResult {
    coord: Option<Coord>,
    direction: Option<Direction>,
    score: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct CoordDirection {
    coord: Coord,
    direction: Direction,
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
                    'Z' => {
                        hazards.insert(coord);
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
            let (body, _): (VecDeque<Coord>, Vec<u32>) = coords.iter().cloned().unzip();
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
            shrink_every_n_turns: 5,
        };
        let settings = RulesetSettings {
            food_spawn_chance: 25,
            minimum_food: 1,
            hazard_damage_per_turn: 15,
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
        };
        let gs = GameState {
            game,
            turn: 0,
            board,
            you: you.unwrap(),
        };
        gs
    }
    fn advance(&mut self, moves: &HashMap<String, Coord>) {
        let mut eaten_food: HashSet<Coord> = HashSet::new();
        let mut snake_heads: HashMap<String, (Coord, u32)> = HashMap::new();
        let mut snake_bodies: HashMap<String, HashSet<Coord>> = HashMap::new();
        // Apply snake moves
        for snake in self.board.snakes.iter_mut() {
            let new_head = moves.get(&snake.id).unwrap().clone();
            snake.head = new_head;
            snake.body.push_front(new_head);
            snake.body.pop_back();
            // Only decrease health in non-constrictor modes
            if self.game.ruleset.name == GameMode::Constrictor {
                snake.body.push_back(snake.body.back().unwrap().clone());
            } else {
                snake.health -= 1;
            }
            // Consume food
            if self.board.food.contains(&snake.head) {
                snake.health = 100;
                snake.body.push_back(snake.body.back().unwrap().clone());
                eaten_food.insert(snake.head);
            } else if self.board.hazards.contains(&snake.head) {
                snake.health -= self.game.ruleset.settings.hazard_damage_per_turn;
            }
            snake.length = snake.body.len() as u32;
            snake_heads.insert(snake.id.clone(), (snake.head, snake.length));
            snake_bodies.insert(snake.id.clone(), HashSet::new());
            for body in snake.body.range(1..) {
                snake_bodies
                    .get_mut(&snake.id)
                    .unwrap()
                    .insert(body.clone());
            }
        }
        // Remove Eaten Food
        for food in &eaten_food {
            self.board.food.remove(food);
        }
        // TODO: Add new food?
        // TODO: Add royale hazards?

        // Eliminate snakes
        let mut eliminated_snakes: HashSet<String> = HashSet::new();
        for snake in &self.board.snakes {
            if snake.health <= 0 {
                eliminated_snakes.insert(snake.id.clone());
                continue;
            }
            if !self.in_bounds(&snake.head) {
                eliminated_snakes.insert(snake.id.clone());
                continue;
            }
            for (id, (head, length)) in &snake_heads {
                // Snakes can't head-to-head with themselves
                if &snake.id == id {
                    continue;
                }
                if &snake.head == head && &snake.length <= length {
                    eliminated_snakes.insert(snake.id.clone());
                    continue;
                }
            }
            for (_, body) in &snake_bodies {
                if body.contains(&snake.head) {
                    eliminated_snakes.insert(snake.id.clone());
                    continue;
                }
            }
        }

        let mut snakes: Vec<Battlesnake> = Vec::new();
        for snake in &self.board.snakes {
            if eliminated_snakes.contains(&snake.id) {
                continue;
            }
            if snake.id == self.you.id {
                self.you = snake.clone();
            }
            snakes.push(snake.clone());
        }
        self.board.snakes = snakes;
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
    fn new_adjacent_coords(&self, coord: &Coord) -> Vec<CoordDirection> {
        let mut cds: Vec<CoordDirection> = Vec::new();
        for direction in Direction::iter() {
            cds.push(CoordDirection {
                coord: self.new_adjacent_coord(coord, &direction),
                direction,
            });
        }
        cds
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
            obstacles.extend(snake.body.range(..snake.body.len() - 1));
        }
        self.board.obstacles = obstacles;
    }
    fn get_random_valid_direction(&self, coord: &Coord) -> CoordDirection {
        let mut valid_directions: Vec<CoordDirection> = Vec::new();

        for direction in Direction::iter() {
            let adjacent_coord = self.new_adjacent_coord(coord, &direction);
            if self.is_valid_and_safe_at(&adjacent_coord) {
                valid_directions.push(CoordDirection {
                    coord: adjacent_coord,
                    direction,
                });
            }
        }

        // Default direction if no valid direction is found
        let mut random_direction: CoordDirection = CoordDirection {
            coord: Coord { x: -1, y: -1 },
            direction: Direction::Down,
        };

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
            for cd in self.new_adjacent_coords(&coord) {
                if !self.is_valid_and_safe_at(&cd.coord) {
                    continue;
                }
                if visited.contains(&cd.coord) {
                    continue;
                }
                let cost_mod = 5;
                let new_cost = costs[&coord] + cost_mod;
                let adjacent_cost = costs.get(&cd.coord);
                if adjacent_cost == None || new_cost < *adjacent_cost.unwrap() {
                    costs.insert(cd.coord.clone(), new_cost);
                    visited.insert(cd.coord.clone());
                    nodes.push(PriorityCoord {
                        coord: cd.coord.clone(),
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
            for cd in self.new_adjacent_coords(&current_coord) {
                if !visited.contains(&cd.coord) && self.is_valid_and_safe_at(&cd.coord) {
                    let distance = distance + 1;
                    if self.board.food.contains(&cd.coord) {
                        food_count.insert(owner.clone(), food_count[&owner] + 1);
                    }
                    for snake in &self.board.snakes {
                        if *snake.body.back().unwrap() != cd.coord {
                            continue;
                        }
                        tail_count.insert(owner.clone(), tail_count[&owner] + 1);
                        if snake.id == owner {
                            contains_our_tail.insert(owner.clone(), true);
                        }
                    }
                    nodes.push_back((owner.clone(), distance, cd.coord));
                    visited.insert(cd.coord);
                    controlled_squares.get_mut(&owner).unwrap().insert(cd.coord);
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
    fn find_best_direction(&self) -> MinimaxResult {
        iterative_deepening(self, 2)
    }
}

fn iterative_deepening(gs: &GameState, max_depth: u32) -> MinimaxResult {
    let start = Instant::now();
    let random_cd = &gs.get_random_valid_direction(&gs.you.head);
    let mut sc = MinimaxResult {
        coord: Some(random_cd.coord),
        direction: Some(random_cd.direction),
        score: i32::MIN,
    };
    for i in 1..=max_depth {
        let mut moves: HashMap<String, Coord> = HashMap::new();
        println!("staring iteration to max depth of {:?}", i);
        let local_sc = minimax(
            gs.clone(),
            &gs.you.id,
            gs.you.id.clone(),
            start,
            i,
            &mut moves,
        );
        if local_sc.score > sc.score {
            sc = local_sc;
        }
        if start.elapsed().as_millis() > 200 {
            break;
        }
    }
    sc
}

fn minimax(
    gs: GameState,
    maximizer: &String,
    current_id: String,
    start: Instant,
    depth: u32,
    moves: &mut HashMap<String, Coord>,
) -> MinimaxResult {
    if depth == 0 {
        return evaluate(&gs, maximizer);
    }
    let mut viable_cds: Vec<CoordDirection> = Vec::new();
    let mut next_id = String::new();
    for (i, snake) in gs.board.snakes.iter().enumerate() {
        if snake.id == current_id {
            viable_cds = gs
                .new_adjacent_coords(&snake.head)
                .iter()
                .cloned()
                .filter(|cd| gs.is_valid_and_safe_at(&cd.coord))
                .collect();
            let next_index = (i + 1) % gs.board.snakes.len();
            next_id = gs.board.snakes[next_index].id.clone();
            break;
        }
    }
    let mut sc = MinimaxResult {
        coord: None,
        direction: None,
        score: i32::MAX,
    };
    if maximizer == &current_id {
        sc.score = i32::MIN;
    }
    for cd in viable_cds {
        println!(
            "depth {:?} current_id: {:?} checking cd: {:?}",
            depth, current_id, cd
        );
        let mut cloned_gs = gs.clone();
        moves.insert(current_id.clone(), cd.coord);
        // All snakes have made moves, so we advance the gamestate
        if moves.len() == cloned_gs.board.snakes.len() {
            println!(
                "advanced gamestate with {:?} snakes alive",
                cloned_gs.board.snakes.len()
            );
            cloned_gs.advance(&moves);
            moves.clear();
        }
        let local_sc = minimax(
            cloned_gs,
            maximizer,
            next_id.clone(),
            start,
            depth - 1,
            moves,
        );
        if maximizer == &current_id && local_sc.score > sc.score {
            sc.coord = Some(cd.coord);
            sc.direction = Some(cd.direction);
            sc.score = local_sc.score;
        } else if local_sc.score < sc.score {
            sc.coord = Some(cd.coord);
            sc.direction = Some(cd.direction);
            sc.score = local_sc.score;
        }
        if start.elapsed().as_millis() > 200 {
            return MinimaxResult {
                coord: None,
                direction: None,
                score: i32::MIN,
            };
        }
    }
    sc
}

fn evaluate(gs: &GameState, id: &String) -> MinimaxResult {
    for snake in &gs.board.snakes {
        if snake.id == *id {
            return MinimaxResult {
                coord: None,
                direction: None,
                score: 10000,
            };
        }
    }
    MinimaxResult {
        coord: None,
        direction: None,
        score: i32::MIN,
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

    let sc = gs.find_best_direction();

    let mr = MoveResponse {
        direction: sc.direction.unwrap(),
        shout: format!("MOVE: {:?} | SCORE: {:?}", sc.direction, sc.score,),
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
        |Z |  |  |  |H |        
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
        assert_eq!(*gs.you.body.back().unwrap(), Coord { x: 1, y: 1 });
        for snake in &gs.board.snakes {
            if snake.id != "A" {
                continue;
            }
            assert_eq!(snake.body.contains(&Coord { x: 3, y: 2 }), true);
            assert_eq!(snake.head, Coord { x: 3, y: 1 });
            assert_eq!(*snake.body.back().unwrap(), Coord { x: 3, y: 3 });
        }
        assert_eq!(gs.board.food.contains(&Coord { x: 2, y: 0 }), true);
        assert_eq!(gs.board.food.contains(&Coord { x: 0, y: 4 }), true);
        assert_eq!(gs.board.hazards.contains(&Coord { x: 4, y: 4 }), true);
        assert_eq!(gs.board.hazards.contains(&Coord { x: 0, y: 4 }), true);
    }
    #[test]
    fn test_advance_basic() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 4 });
        moves.insert("A".to_owned(), Coord { x: 3, y: 0 });
        gs.advance(&moves);
        assert_eq!(gs.you.body.contains(&Coord { x: 1, y: 3 }), true);
        assert_eq!(gs.you.head, Coord { x: 1, y: 4 });
        assert_eq!(*gs.you.body.back().unwrap(), Coord { x: 1, y: 2 });
        for snake in &gs.board.snakes {
            if snake.id != "A" {
                continue;
            }
            assert_eq!(snake.body.contains(&Coord { x: 3, y: 1 }), true);
            assert_eq!(snake.head, Coord { x: 3, y: 0 });
            assert_eq!(*snake.body.back().unwrap(), Coord { x: 3, y: 2 });
        }
        assert_eq!(gs.board.food.contains(&Coord { x: 2, y: 0 }), true);
        assert_eq!(gs.board.hazards.contains(&Coord { x: 4, y: 4 }), true);
    }
    #[test]
    fn test_advance_food() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |F |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 4 });
        moves.insert("A".to_owned(), Coord { x: 3, y: 0 });
        gs.advance(&moves);
        for snake in &gs.board.snakes {
            match snake.id.as_str() {
                "A" => {
                    assert_eq!(snake.health, 100);
                    assert_eq!(snake.length, 4);
                    assert_eq!(snake.body[2], Coord { x: 3, y: 2 });
                    assert_eq!(snake.body[3], Coord { x: 3, y: 2 });
                }
                _ => {}
            }
        }
        assert_eq!(gs.board.food.contains(&Coord { x: 2, y: 0 }), true);
        assert_eq!(gs.board.food.contains(&Coord { x: 3, y: 0 }), false);
    }
    #[test]
    fn test_advance_multiple() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |F |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 4 });
        moves.insert("A".to_owned(), Coord { x: 3, y: 0 });
        gs.advance(&moves);
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 5 });
        moves.insert("A".to_owned(), Coord { x: 2, y: 0 });
        gs.advance(&moves);
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("A".to_owned(), Coord { x: 1, y: 0 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 1);
        for snake in &gs.board.snakes {
            match snake.id.as_str() {
                "A" => {
                    assert_eq!(snake.health, 99);
                    assert_eq!(snake.length, 5);
                    assert_eq!(snake.body[0], Coord { x: 1, y: 0 });
                    assert_eq!(snake.body[1], Coord { x: 2, y: 0 });
                    assert_eq!(snake.body[2], Coord { x: 3, y: 0 });
                    assert_eq!(snake.body[3], Coord { x: 3, y: 1 });
                    assert_eq!(snake.body[4], Coord { x: 3, y: 2 });
                }
                _ => {}
            }
        }
    }
    #[test]
    fn test_advance_chase_tail() {
        let mut gs = GameState::new_from_text(
            "
        |  |Y7|Y6|  |  |        
        |  |Y0|Y5|  |  |        
        |  |Y1|Y4|  |  |        
        |  |Y2|Y3|  |  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 4 });
        gs.advance(&moves);
        assert_eq!(gs.you.body[0], Coord { x: 1, y: 4 });
        assert_eq!(gs.you.body[7], Coord { x: 2, y: 4 });
        assert_eq!(gs.board.snakes.len(), 1);
    }
    #[test]
    fn test_advance_self_collision() {
        let mut gs = GameState::new_from_text(
            "
        |Y8|Y7|Y6|  |  |        
        |  |Y0|Y5|  |  |        
        |  |Y1|Y4|  |  |        
        |  |Y2|Y3|  |  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 4 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 0);
    }
    #[test]
    fn test_advance_other_collision() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |  |Y0|Y1|Y2|  |        
        |A2|A1|A0|  |  |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 2 });
        moves.insert("A".to_owned(), Coord { x: 3, y: 2 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 1);
    }
    #[test]
    fn test_advance_head_loss() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |  |Y0|Y1|Y2|  |        
        |  |  |  |  |  |        
        |  |A0|A1|A2|  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 2 });
        moves.insert("A".to_owned(), Coord { x: 1, y: 2 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 0);
    }
    #[test]
    fn test_advance_head_win() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |  |Y0|Y1|Y2|Y3|        
        |  |  |  |  |  |        
        |  |A0|A1|A2|  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 2 });
        moves.insert("A".to_owned(), Coord { x: 1, y: 2 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 1);
    }
    #[test]
    fn test_advance_hazard_basic() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |H |Y0|Y1|Y2|  |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 0, y: 3 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 1);
        assert_eq!(gs.you.health, 84);
    }
    #[test]
    fn test_advance_hazard_death() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |H |Y0|Y1|Y2|  |        
        |H |H |H |H |H |        
        |  |  |  |  |H |        
        |  |  |  |  |  |        
        ",
        );
        let coords = vec![
            Coord { x: 0, y: 3 },
            Coord { x: 0, y: 2 },
            Coord { x: 1, y: 2 },
            Coord { x: 2, y: 2 },
            Coord { x: 3, y: 2 },
            Coord { x: 4, y: 2 },
            Coord { x: 4, y: 1 },
        ];
        for coord in coords {
            let mut moves: HashMap<String, Coord> = HashMap::new();
            moves.insert("Y".to_owned(), coord);
            gs.advance(&moves);
        }
        let expected_health = 100 - 16 * 6;
        assert_eq!(gs.you.head, Coord { x: 4, y: 2 });
        assert_eq!(gs.board.snakes.len(), 0);
        assert_eq!(gs.you.health, expected_health);
    }
    #[test]
    fn test_advance_hazard_with_food() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |Z |Y0|Y1|Y2|  |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 0, y: 3 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 1);
        assert_eq!(gs.you.health, 100);
    }
    #[test]
    fn test_advance_starving() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |H |Y0|Y1|Y2|  |        
        |H |H |H |H |H |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        let coords = vec![
            Coord { x: 0, y: 3 },
            Coord { x: 0, y: 2 },
            Coord { x: 1, y: 2 },
            Coord { x: 2, y: 2 },
            Coord { x: 3, y: 2 },
            Coord { x: 4, y: 2 },
            Coord { x: 3, y: 1 },
            Coord { x: 2, y: 1 },
            Coord { x: 1, y: 1 },
            Coord { x: 0, y: 1 },
        ];
        for coord in coords {
            let mut moves: HashMap<String, Coord> = HashMap::new();
            moves.insert("Y".to_owned(), coord);
            gs.advance(&moves);
        }
        assert_eq!(gs.you.head, Coord { x: 1, y: 1 });
        assert_eq!(gs.board.snakes.len(), 0);
        assert_eq!(gs.you.health, 1);
    }
    #[test]
    fn test_advance_eat_food_on_starve_turn() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |H |Y0|Y1|Y2|  |        
        |H |H |H |H |H |        
        |F |  |  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        let coords = vec![
            Coord { x: 0, y: 3 },
            Coord { x: 0, y: 2 },
            Coord { x: 1, y: 2 },
            Coord { x: 2, y: 2 },
            Coord { x: 3, y: 2 },
            Coord { x: 4, y: 2 },
            Coord { x: 3, y: 1 },
            Coord { x: 2, y: 1 },
            Coord { x: 1, y: 1 },
            Coord { x: 0, y: 1 },
        ];
        for coord in coords {
            let mut moves: HashMap<String, Coord> = HashMap::new();
            moves.insert("Y".to_owned(), coord);
            gs.advance(&moves);
        }
        assert_eq!(gs.you.head, Coord { x: 0, y: 1 });
        assert_eq!(gs.board.snakes.len(), 1);
        assert_eq!(gs.you.health, 100);
    }
    #[test]
    fn test_advance_wrapped() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |  |Y0|  |  |  |        
        |  |Y1|  |  |  |        
        |  |Y2|  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        gs.game.ruleset.name = GameMode::Wrapped;
        let coords = vec![
            Coord { x: 1, y: 4 },
            Coord { x: 1, y: 0 },
            Coord { x: 1, y: 1 },
            Coord { x: 1, y: 2 },
        ];
        for coord in coords {
            let mut moves: HashMap<String, Coord> = HashMap::new();
            moves.insert("Y".to_owned(), coord);
            gs.advance(&moves);
        }
        assert_eq!(gs.you.body.contains(&Coord { x: 1, y: 1 }), true);
        assert_eq!(gs.you.head, Coord { x: 1, y: 2 });
        assert_eq!(*gs.you.body.back().unwrap(), Coord { x: 1, y: 0 });
        assert_eq!(gs.board.snakes.len(), 1);
    }
    #[test]
    fn test_advance_constrictor() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |  |        
        |  |Y0|  |  |  |        
        |  |Y1|  |  |  |        
        |  |Y2|  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        gs.game.ruleset.name = GameMode::Constrictor;
        let coords = vec![
            Coord { x: 1, y: 4 },
            Coord { x: 2, y: 4 },
            Coord { x: 3, y: 4 },
            Coord { x: 4, y: 4 },
        ];
        for coord in coords {
            let mut moves: HashMap<String, Coord> = HashMap::new();
            moves.insert("Y".to_owned(), coord);
            gs.advance(&moves);
        }
        assert_eq!(gs.you.body.contains(&Coord { x: 1, y: 3 }), true);
        assert_eq!(gs.you.body.contains(&Coord { x: 1, y: 4 }), true);
        assert_eq!(gs.you.body.contains(&Coord { x: 2, y: 4 }), true);
        assert_eq!(gs.you.body.contains(&Coord { x: 3, y: 4 }), true);
        assert_eq!(gs.you.head, Coord { x: 4, y: 4 });
        assert_eq!(*gs.you.body.back().unwrap(), Coord { x: 1, y: 2 });
        assert_eq!(gs.board.snakes.len(), 1);
        assert_eq!(gs.you.health, 100);
    }
    // TODO: test royale
    #[test]
    fn test_minimax() {
        let mut gs = GameState::new_from_text(
            "
        |  |  |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |  |  |        
        ",
        );
        gs.init();
        let mm = gs.find_best_direction();
        assert_eq!(mm.score, 10000);
    }
}
