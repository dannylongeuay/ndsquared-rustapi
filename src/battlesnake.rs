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

impl Coord {
    fn manhattan_distance(&self, other: &Coord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PriorityCoord {
    coord: Coord,
    priority: u32,
}

impl Ord for PriorityCoord {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
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
    hazards: Vec<Coord>,
    /// Array of Battlesnake Objects representing all Battlesnakes remaining on the game board (including yourself if you haven't been eliminated). Example: [{"id": "snake-one", ...}, ...]
    snakes: Vec<Battlesnake>,
    /// Set of coords for all snake's bodies minus tails.
    #[serde(skip)]
    obstacles: HashSet<Coord>,
    /// Mapping of hazard coordinates and their corresponding damage.
    #[serde(skip)]
    hazard_damage: HashMap<Coord, i32>,
    /// Set of coords adjacent to enemy snake heads that are smaller in size.
    #[serde(skip)]
    stomps: HashSet<Coord>,
    /// Set of coords adjacent to enemy snake heads that are equal or bigger in size.
    #[serde(skip)]
    avoids: HashSet<Coord>,
    /// Mapping of snake ids to their index in the snakes array.
    #[serde(skip)]
    snake_indexes: HashMap<String, usize>,
}

impl Board {
    fn get_snake(&self, id: &String) -> Option<&Battlesnake> {
        let snake_index = self.snake_indexes.get(id);
        if snake_index.is_none() {
            return None;
        }
        self.snakes.get(*snake_index.unwrap())
    }
    fn center(&self) -> Coord {
        Coord {
            x: self.width / 2,
            y: self.width / 2,
        }
    }
}

#[derive(Debug)]
pub struct TerritoryInfo {
    controlled_squares: HashMap<String, HashSet<Coord>>,
    available_squares: HashMap<String, HashSet<Coord>>,
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
    #[serde(skip)]
    eliminated: bool,
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

fn in_bounds(coord: &Coord, width: i32, height: i32) -> bool {
    return coord.x >= 0 && coord.y >= 0 && coord.x < width && coord.y < height;
}

impl GameState {
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
            } else if let Some(damage) = self.board.hazard_damage.get(&snake.head) {
                snake.health -= damage;
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
        for snake in self.board.snakes.iter_mut() {
            if snake.health <= 0 {
                snake.eliminated = true;
                continue;
            }
            if !in_bounds(&snake.head, self.board.width, self.board.height) {
                snake.eliminated = true;
                continue;
            }
            for (id, (head, length)) in &snake_heads {
                // Snakes can't head-to-head with themselves
                if &snake.id == id {
                    continue;
                }
                if &snake.head == head && &snake.length <= length {
                    snake.eliminated = true;
                    continue;
                }
            }
            for (_, body) in &snake_bodies {
                if body.contains(&snake.head) {
                    snake.eliminated = true;
                    continue;
                }
            }
        }

        // TODO: combine this into the previous loop?
        let mut snakes: Vec<Battlesnake> = Vec::new();
        for snake in &self.board.snakes {
            if snake.id == self.you.id {
                self.you = snake.clone();
            }
            if snake.eliminated {
                continue;
            }
            snakes.push(snake.clone());
        }
        self.board.snakes = snakes;
        self.compute_metadata();
    }
    fn adjacent_coord(&self, coord: &Coord, dir: &Direction) -> Coord {
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
    fn adjacent_moves(&self, coord: &Coord) -> Vec<(Coord, Direction)> {
        let mut moves: Vec<(Coord, Direction)> = Vec::new();
        for direction in Direction::iter() {
            moves.push((self.adjacent_coord(coord, &direction), direction));
        }
        moves
    }
    fn valid_at(&self, coord: &Coord) -> bool {
        in_bounds(coord, self.board.width, self.board.height)
    }
    fn safe_at(&self, coord: &Coord) -> bool {
        !self.board.obstacles.contains(coord)
    }
    fn viable(&self, coord: &Coord) -> bool {
        self.valid_at(coord) && self.safe_at(coord)
    }
    fn init(&mut self) {
        self.compute_metadata();
    }
    fn compute_metadata(&mut self) {
        let mut obstacles: HashSet<Coord> = HashSet::new();
        let mut hazard_damage: HashMap<Coord, i32> = HashMap::new();
        let mut stomps: HashSet<Coord> = HashSet::new();
        let mut avoids: HashSet<Coord> = HashSet::new();
        let mut snake_indexes: HashMap<String, usize> = HashMap::new();
        for (i, snake) in self.board.snakes.iter().enumerate() {
            snake_indexes.insert(snake.id.clone(), i);
            for (i, coord) in snake.body.iter().enumerate() {
                if i != snake.body.len() - 1 {
                    obstacles.insert(coord.clone());
                }
                if self.you.id == snake.id {
                    continue;
                }
                if i != 0 {
                    continue;
                }
                if self.you.length <= snake.length {
                    avoids.extend(self.adjacent_moves(&snake.head).iter().map(|&t| t.0));
                } else {
                    stomps.extend(self.adjacent_moves(&snake.head).iter().map(|&t| t.0));
                }
            }
        }
        for hazard in &self.board.hazards {
            let mut total_damage: i32 = self.game.ruleset.settings.hazard_damage_per_turn;
            if let Some(damage) = hazard_damage.get_mut(&hazard) {
                *damage += total_damage;
                total_damage = damage.clone();
            } else {
                hazard_damage.insert(hazard.clone(), total_damage);
            }
            if total_damage >= self.you.health {
                obstacles.insert(hazard.clone());
            }
        }

        self.board.snake_indexes = snake_indexes;
        self.board.obstacles = obstacles;
        self.board.hazard_damage = hazard_damage;
        self.board.stomps = stomps;
        self.board.avoids = avoids;
    }
    fn random_valid_move(&self, coord: &Coord) -> (Coord, Direction) {
        let mut valid_moves: Vec<(Coord, Direction)> = Vec::new();
        let mut food_moves: Vec<(Coord, Direction)> = Vec::new();

        for direction in Direction::iter() {
            let adjacent_coord = self.adjacent_coord(coord, &direction);
            if !self.viable(&adjacent_coord) {
                continue;
            }
            valid_moves.push((adjacent_coord, direction));
            if self.board.food.contains(&adjacent_coord) {
                food_moves.push((adjacent_coord, direction));
            }
        }

        // Default direction if no valid direction is found
        let mut random_move: (Coord, Direction) = (Coord { x: -1, y: -1 }, Direction::Down);

        if food_moves.len() > 0 {
            random_move = *food_moves.choose(&mut rand::thread_rng()).unwrap();
        } else if valid_moves.len() > 0 {
            random_move = *valid_moves.choose(&mut rand::thread_rng()).unwrap();
        }
        random_move
    }
    fn shortest_distance(&self, start: &Coord, end: &Coord) -> Option<u32> {
        let mut nodes: BinaryHeap<PriorityCoord> = BinaryHeap::new();
        let mut visited: HashSet<Coord> = HashSet::new();
        let mut distances: HashMap<Coord, u32> = HashMap::new();
        nodes.push(PriorityCoord {
            coord: start.clone(),
            priority: 0,
        });
        visited.insert(start.clone());
        distances.insert(start.clone(), 0);
        while let Some(PriorityCoord { coord, priority: _ }) = nodes.pop() {
            if coord == *end {
                return Some(distances[&coord]);
            }
            for (adj_coord, _) in self.adjacent_moves(&coord) {
                if !self.viable(&adj_coord) {
                    continue;
                }
                if visited.contains(&adj_coord) {
                    continue;
                }
                let new_distance = distances[&coord] + 1;
                let adjacent_distance = distances.get(&adj_coord);
                if adjacent_distance == None || new_distance < *adjacent_distance.unwrap() {
                    distances.insert(adj_coord.clone(), new_distance);
                    visited.insert(adj_coord.clone());
                    let new_priority = distances[&coord] + adj_coord.manhattan_distance(end) as u32;
                    nodes.push(PriorityCoord {
                        coord: adj_coord.clone(),
                        priority: new_priority,
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
                if closest_distance.is_none() || food_distance < closest_distance.unwrap() {
                    closest_distance = Some(food_distance);
                }
            }
        }
        closest_distance
    }
    // TODO: this is horribly innefficient
    fn compute_territory_info(&self) -> TerritoryInfo {
        let mut controlled_squares: HashMap<String, HashSet<Coord>> = HashMap::new();
        let mut available_squares: HashMap<String, HashSet<Coord>> = HashMap::new();
        let mut nodes: VecDeque<(String, u32, Coord)> = VecDeque::new();
        let mut distances: HashMap<Coord, u32> = HashMap::new();
        let mut visited: HashSet<Coord> = HashSet::new();
        for snake in &self.board.snakes {
            controlled_squares.insert(snake.id.clone(), HashSet::new());
            nodes.push_back((snake.id.clone(), 0, snake.head));
            distances.insert(snake.head, 0);
            controlled_squares
                .get_mut(&snake.id)
                .unwrap()
                .insert(snake.head);
        }
        while let Some((owner, distance, current_coord)) = nodes.pop_front() {
            for (adj_coord, _dir) in self.adjacent_moves(&current_coord) {
                if !self.viable(&adj_coord) {
                    continue;
                }
                let dist_check = distances.get(&adj_coord);
                let new_distance = distance + 1;
                if dist_check.is_none() {
                    nodes.push_back((owner.clone(), new_distance, adj_coord));
                    distances.insert(adj_coord, new_distance);
                    controlled_squares
                        .get_mut(&owner)
                        .unwrap()
                        .insert(adj_coord);
                } else if dist_check.is_some() && *dist_check.unwrap() == new_distance {
                    // Squares are owned by all snakes that can reach them in the same distance
                    controlled_squares
                        .get_mut(&owner)
                        .unwrap()
                        .insert(adj_coord);
                }
            }
        }
        for snake in &self.board.snakes {
            nodes.clear();
            visited.clear();
            available_squares.insert(snake.id.clone(), HashSet::new());
            nodes.push_back((snake.id.clone(), 0, snake.head));
            visited.insert(snake.head);
            available_squares
                .get_mut(&snake.id)
                .unwrap()
                .insert(snake.head);
            while let Some((_owner, distance, current_coord)) = nodes.pop_front() {
                for (coord, _) in self.adjacent_moves(&current_coord) {
                    if !visited.contains(&coord) && self.viable(&coord) {
                        let distance = distance + 1;
                        nodes.push_back((snake.id.clone(), distance, coord));
                        visited.insert(coord);
                        available_squares.get_mut(&snake.id).unwrap().insert(coord);
                    }
                }
            }
        }
        TerritoryInfo {
            controlled_squares,
            available_squares,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Score {
    min: bool,
    max: bool,
    biggest: bool,
    center_dist: i32,
    tail_dist: i32,
    food_dist: i32,
    length: i32,
    snake_stomps: i32,
    snake_avoids: i32,
    board_control: i32,
    snakes_eliminated: i32,
}

impl Score {
    fn new() -> Self {
        Score {
            min: false,
            max: false,
            biggest: false,
            center_dist: 0,
            tail_dist: 0,
            food_dist: 0,
            length: 0,
            snake_stomps: 0,
            snake_avoids: 0,
            board_control: 0,
            snakes_eliminated: 0,
        }
    }
    fn sum(&self) -> i32 {
        if self.min {
            return i32::MIN;
        } else if self.max {
            return i32::MAX;
        }
        let mut result: i32 = 0;
        if self.biggest {
            result += 1000;
        }
        result += self.center_dist;
        result += self.tail_dist;
        result += self.food_dist;
        result += self.length;
        result += self.snake_stomps;
        result += self.snake_avoids;
        result += self.board_control;
        result += self.snakes_eliminated;
        result
    }
}

#[derive(Debug)]
pub struct Search {
    current_depth: u32,
    iteration_reached: u32,
    advances: u32,
    terminals: u32,
    best_direction: Direction,
    best_score: Score,
    best_pv: Vec<Coord>,
    search_time: u128,
}

impl Search {
    fn new(gs: &GameState) -> Self {
        let mut best_score = Score::new();
        best_score.min = true;
        Search {
            current_depth: 0,
            iteration_reached: 1,
            advances: 0,
            terminals: 0,
            best_direction: gs.random_valid_move(&gs.you.head).1,
            best_score,
            best_pv: Vec::new(),
            search_time: 0,
        }
    }
    fn iterative_deepening(&mut self, gs: &GameState, max_depth: u32) {
        let start = Instant::now();
        for i in 1..=max_depth {
            let moves: HashMap<String, Coord> = HashMap::new();
            let mut root_pv: Vec<Coord> = Vec::new();
            let _score = self.minimax_alphabeta(
                gs.clone(),
                &gs.you.id,
                gs.you.id.clone(),
                start,
                i,
                i32::MIN,
                i32::MAX,
                moves.clone(),
                &mut root_pv,
            );
            let debug_header = format!("{} Depth {:?} {}", "#".repeat(75), i, "#".repeat(25));
            if i <= 20 {
                debug!("\n{}", debug_header);
                debug!(
                "Advances: {:?} | Terminals: {:?} | Best Direction: {:?} | Best Score Sum: {:?}",
                self.advances,
                self.terminals,
                self.best_direction,
                self.best_score.sum()
            );
                debug!("Sum: {:?}\n{:?}", _score.sum(), _score);
                debug!(
                    "Best Sum: {:?}\n{:?}",
                    self.best_score.sum(),
                    self.best_score
                );
                debug!("PV: {:?}\n{}", root_pv, "#".repeat(debug_header.len()));
            }
            self.advances = 0;
            self.terminals = 0;
            self.current_depth = 0;
            if start.elapsed().as_millis() > gs.game.timeout as u128 - 100 {
                break;
            }
            self.iteration_reached = i;
        }
        self.search_time = start.elapsed().as_millis();
    }

    fn minimax_alphabeta(
        &mut self,
        gs: GameState,
        maximizer: &String,
        current_id: String,
        start: Instant,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        mut pending_moves: HashMap<String, Coord>,
        pv: &mut Vec<Coord>,
    ) -> Score {
        let mut score = Score::new();

        if maximizer == &current_id {
            score.min = true;
        } else {
            score.max = true;
        }

        if start.elapsed().as_millis() > gs.game.timeout as u128 - 100 {
            score.min = true;
            return score;
        }

        if depth == 0 {
            self.terminals += 1;
            return self.evaluate(&gs);
        }

        let snake = gs.board.get_snake(&current_id);
        if snake.is_none() {
            self.terminals += 1;
            return self.evaluate(&gs);
        }

        let snake = snake.unwrap();
        let mut viable_moves: Vec<(Coord, Direction)> = gs
            .adjacent_moves(&snake.head)
            .iter()
            .cloned()
            .filter(|(coord, _)| gs.viable(&coord))
            .collect();
        trace!(
            "Current Depth {:?} | Tree Depth {:?} | Current ID: {:?} | Viable Moves: {:?} | Pending Moves: {:?}",
            self.current_depth,
            depth,
            current_id,
            viable_moves,
            pending_moves,
        );

        let next_index = (gs.board.snake_indexes[&current_id] + 1) % gs.board.snakes.len();
        let next_id = gs.board.snakes[next_index].id.clone();

        // If a snake has no viable moves, we make a random move
        if viable_moves.len() == 0 {
            viable_moves.push(gs.random_valid_move(&snake.head));
        }

        for (coord, direction) in viable_moves {
            let mut node_pv: Vec<Coord> = Vec::new();
            let mut cloned_gs = gs.clone();
            pending_moves.insert(current_id.clone(), coord);
            // All snakes have made moves, so we advance the gamestate
            if pending_moves.len() == cloned_gs.board.snakes.len() {
                trace!(
                    "Advanced > Current Depth {:?} | Tree Depth {:?} | Moves: {:?}",
                    self.current_depth,
                    depth,
                    pending_moves
                );
                self.advances += 1;
                cloned_gs.advance(&pending_moves);
                // Remove the move that was just played and filter out any moves for eliminated snakes
                pending_moves.remove(&current_id);
                pending_moves.retain(|k, _| cloned_gs.board.snake_indexes.get(k).is_some());
            }
            trace!(
                    "DOWN > Current Depth {:?} | Tree Depth {:?} | Score: {:?} | A: {:?} | B: {:?} | Current ID: {:?} | Coord: {:?} | Move: {:?}",
                    self.current_depth, depth, score, alpha, beta, current_id, coord, direction
                );
            if maximizer == &current_id {
                self.current_depth += 1;
                let node_score = self.minimax_alphabeta(
                    cloned_gs,
                    maximizer,
                    next_id.clone(),
                    start,
                    depth - 1,
                    alpha,
                    beta,
                    pending_moves.clone(),
                    &mut node_pv,
                );
                if node_score.sum() > score.sum() {
                    score = node_score;
                }
                self.current_depth -= 1;
                if score.sum() > alpha {
                    pv.clear();
                    pv.push(coord);
                    pv.append(&mut node_pv);
                    alpha = score.sum();
                }
            } else {
                self.current_depth += 1;
                let node_score = self.minimax_alphabeta(
                    cloned_gs,
                    maximizer,
                    next_id.clone(),
                    start,
                    depth - 1,
                    alpha,
                    beta,
                    pending_moves.clone(),
                    pv,
                );
                if node_score.sum() < score.sum() {
                    score = node_score;
                }
                self.current_depth -= 1;
                if score.sum() < beta {
                    beta = score.sum();
                }
            }
            trace!(
                    "UP   > Current Depth {:?} | Tree Depth {:?} | Score: {:?} | A: {:?} | B: {:?} | Current ID: {:?} | Coord: {:?} | Move: {:?}",
                    self.current_depth, depth, score, alpha, beta, current_id, coord, direction
                );
            // If we run out of time, return before we attemp to set a new direction
            if start.elapsed().as_millis() > gs.game.timeout as u128 - 100 {
                score.min = true;
                return score;
            }
            if self.current_depth == 0 && self.advances > 0 && score.sum() > self.best_score.sum() {
                trace!(
                    "New Best Score: {:?} | A: {:?} | B: {:?} | Current ID: {:?} | Coord: {:?} | Move: {:?}",
                    score, alpha, beta, current_id, coord, direction
                );
                self.best_direction = direction;
                self.best_score = score.clone();
                self.best_pv = pv.clone();
            }
            if maximizer == &current_id && alpha >= beta {
                break;
            } else if beta <= alpha {
                break;
            }
        }
        score
    }

    fn evaluate(&self, gs: &GameState) -> Score {
        let mut score = Score::new();
        // Elimination is bad
        if gs.you.eliminated {
            score.min = true;
            return score;
        }

        // The closer we are to the center the better
        score.center_dist = -gs.you.head.manhattan_distance(&gs.board.center());

        // Penalize moving to where a bigger or equal snakes head might be
        // Incentivize moving to where a smaller snakes head might be
        if gs.board.avoids.contains(&gs.you.head) {
            score.snake_avoids = -5000;
        } else if gs.board.stomps.contains(&gs.you.head) {
            score.snake_stomps = 5000;
        }

        // Maximize our "controlled" squares
        let territory_info = gs.compute_territory_info();
        if let Some(controlled_squares) = territory_info.controlled_squares.get(&gs.you.id) {
            score.board_control = controlled_squares.len() as i32 * 10;
        }

        // Going into a dead end is bad
        if let Some(available_squares) = territory_info.available_squares.get(&gs.you.id) {
            if available_squares.len() < gs.you.length as usize + 1 {
                score.board_control = -10000;
            }
        }

        // Having a path to our own tail is good
        if let Some(tail_distance) =
            gs.shortest_distance(&gs.you.head, &gs.you.body.back().unwrap())
        {
            score.tail_dist = -(tail_distance as i32);
        }

        if let Some(food_distance) = gs.closest_food_distance(&gs.you.head) {
            let food_mod: i32 = 101 - gs.you.health;
            score.food_dist = -(food_distance as i32) * food_mod;
        } else if gs.you.health < 20 {
            score.food_dist = -5000;
        }

        // Growing bigger is good
        score.length = gs.you.length as i32 * 100;

        // Being the biggest snake is good
        score.biggest = gs.board.snakes.iter().all(|s| {
            if s.id == gs.you.id {
                return true;
            }
            s.length < gs.you.length
        });

        // Other snakes being eliminated is good
        if gs.game.ruleset.name != GameMode::Solo && gs.board.snakes.len() == 1 {
            score.max = true;
        }

        score
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
        version: "1.11.0".to_owned(),
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

    let mut search = Search::new(&gs);
    search.iterative_deepening(&gs, 50);

    let mr = MoveResponse {
        direction: search.best_direction,
        shout: format!(
            "MOVE: {:?} | SCORE: {:?} | TIME: {:?} | ITERATIONS: {:?} | PV LENGTH: {:?}",
            search.best_direction,
            search.best_score.sum(),
            search.search_time,
            search.iteration_reached,
            search.best_pv.len()
        ),
    };

    info!("{:?}", mr);
    info!("{:?}", search.best_score);
    info!("PV: {:?}", search.best_pv);

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
    use test_log::test;

    fn new_gamestate_from_text(text: &str) -> GameState {
        let mut height: i32 = 0;
        let mut width: i32 = 0;
        let mut y = 0;
        let mut snake_bodies: HashMap<char, Vec<(Coord, u32)>> = HashMap::new();
        let mut food: HashSet<Coord> = HashSet::new();
        let mut hazards: Vec<Coord> = Vec::new();
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
                        hazards.push(coord);
                    }
                    'F' => {
                        food.insert(coord);
                    }
                    'Z' => {
                        hazards.push(coord);
                        food.insert(coord);
                    }
                    'G' => {
                        hazards.push(coord);
                        hazards.push(coord);
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
                eliminated: false,
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
            version: "1.11.0".to_owned(),
            settings,
        };
        let game = Game {
            id: "gameid".to_owned(),
            map: GameMap::Standard,
            ruleset,
            timeout: 500,
            source: Source::Custom,
        };
        let board = Board {
            height,
            width,
            food,
            hazards,
            snakes,
            obstacles: HashSet::new(),
            hazard_damage: HashMap::new(),
            stomps: HashSet::new(),
            avoids: HashSet::new(),
            snake_indexes: HashMap::new(),
        };
        let mut gs = GameState {
            game,
            turn: 0,
            board,
            you: you.unwrap(),
        };
        gs.compute_metadata();
        gs
    }
    #[test]
    fn test_new_from_text() {
        let gs = new_gamestate_from_text(
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
        let snake = gs.board.get_snake(&"A".to_owned());
        assert_eq!(snake.is_none(), false);
        let snake = snake.unwrap();
        assert_eq!(snake.body.contains(&Coord { x: 3, y: 2 }), true);
        assert_eq!(snake.head, Coord { x: 3, y: 1 });
        assert_eq!(*snake.body.back().unwrap(), Coord { x: 3, y: 3 });
        assert_eq!(gs.board.food.contains(&Coord { x: 2, y: 0 }), true);
        assert_eq!(gs.board.food.contains(&Coord { x: 0, y: 4 }), true);
        assert_eq!(
            gs.board.hazard_damage.contains_key(&Coord { x: 4, y: 4 }),
            true
        );
        assert_eq!(
            gs.board.hazard_damage.contains_key(&Coord { x: 0, y: 4 }),
            true
        );
    }
    #[test]
    fn test_gamestate_cloning() {
        let gs = new_gamestate_from_text(
            "
        |  |F |  |  |H |
        |  |Y0|  |A2|  |
        |  |Y1|  |A1|  |
        |  |Y2|  |A0|  |
        |  |  |F |  |  |
        ",
        );
        let mut cloned_gs = gs.clone();
        let food = Coord { x: 1, y: 4 };
        cloned_gs.board.food.remove(&food);
        cloned_gs.board.snakes.pop();
        cloned_gs.you.health -= 10;
        assert_eq!(gs.board.food.contains(&food), true);
        assert_eq!(gs.board.snakes.len(), 2);
        assert_eq!(gs.you.health, 100);
        assert_eq!(cloned_gs.board.food.contains(&food), false);
        assert_eq!(cloned_gs.board.snakes.len(), 1);
        assert_eq!(cloned_gs.you.health, 90);
    }
    #[test]
    fn test_advance_basic() {
        let mut gs = new_gamestate_from_text(
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
        let snake = gs.board.get_snake(&"A".to_owned());
        assert_eq!(snake.is_none(), false);
        let snake = snake.unwrap();
        assert_eq!(snake.body.contains(&Coord { x: 3, y: 1 }), true);
        assert_eq!(snake.head, Coord { x: 3, y: 0 });
        assert_eq!(*snake.body.back().unwrap(), Coord { x: 3, y: 2 });
        assert_eq!(gs.board.food.contains(&Coord { x: 2, y: 0 }), true);
        assert_eq!(
            gs.board.hazard_damage.contains_key(&Coord { x: 4, y: 4 }),
            true
        );
    }
    #[test]
    fn test_advance_food() {
        let mut gs = new_gamestate_from_text(
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
        let snake = gs.board.get_snake(&"A".to_owned());
        assert_eq!(snake.is_none(), false);
        let snake = snake.unwrap();
        assert_eq!(snake.health, 100);
        assert_eq!(snake.length, 4);
        assert_eq!(snake.body[2], Coord { x: 3, y: 2 });
        assert_eq!(snake.body[3], Coord { x: 3, y: 2 });
        assert_eq!(gs.board.food.contains(&Coord { x: 2, y: 0 }), true);
        assert_eq!(gs.board.food.contains(&Coord { x: 3, y: 0 }), false);
    }
    #[test]
    fn test_advance_multiple() {
        let mut gs = new_gamestate_from_text(
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
        let snake = gs.board.get_snake(&"A".to_owned());
        assert_eq!(snake.is_none(), false);
        let snake = snake.unwrap();
        assert_eq!(snake.health, 99);
        assert_eq!(snake.length, 5);
        assert_eq!(snake.body[0], Coord { x: 1, y: 0 });
        assert_eq!(snake.body[1], Coord { x: 2, y: 0 });
        assert_eq!(snake.body[2], Coord { x: 3, y: 0 });
        assert_eq!(snake.body[3], Coord { x: 3, y: 1 });
        assert_eq!(snake.body[4], Coord { x: 3, y: 2 });
    }
    #[test]
    fn test_advance_chase_tail() {
        let mut gs = new_gamestate_from_text(
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
        let mut gs = new_gamestate_from_text(
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
        let mut gs = new_gamestate_from_text(
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
        let mut gs = new_gamestate_from_text(
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
    fn test_advance_head_loss_over_food() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |        
        |  |Y0|Y1|Y2|  |        
        |  |F |  |  |  |        
        |  |A0|A1|A2|  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 2 });
        moves.insert("A".to_owned(), Coord { x: 1, y: 2 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 0);
        assert_eq!(gs.you.eliminated, true);
    }
    #[test]
    fn test_advance_head_win() {
        let mut gs = new_gamestate_from_text(
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
    fn test_advance_head_win_over_food() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |        
        |  |Y0|Y1|Y2|Y3|        
        |  |F |  |  |  |        
        |  |A0|A1|A2|  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 1, y: 2 });
        moves.insert("A".to_owned(), Coord { x: 1, y: 2 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 1);
        assert_eq!(gs.you.health, 100);
    }
    #[test]
    fn test_advance_hazard_basic() {
        let mut gs = new_gamestate_from_text(
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
    fn test_advance_hazard_double() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |        
        |G |Y0|Y1|Y2|  |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        |  |  |  |  |  |        
        ",
        );
        let mut moves: HashMap<String, Coord> = HashMap::new();
        moves.insert("Y".to_owned(), Coord { x: 0, y: 3 });
        gs.advance(&moves);
        assert_eq!(gs.board.snakes.len(), 1);
        assert_eq!(gs.you.health, 69);
    }
    #[test]
    fn test_advance_hazard_death() {
        let mut gs = new_gamestate_from_text(
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
        let expected_health = 100 - 16 * 7;
        assert_eq!(gs.you.head, Coord { x: 4, y: 1 });
        assert_eq!(gs.board.snakes.len(), 0);
        assert_eq!(gs.you.eliminated, true);
        assert_eq!(gs.you.health, expected_health);
    }
    #[test]
    fn test_advance_hazard_with_food() {
        let mut gs = new_gamestate_from_text(
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
        let mut gs = new_gamestate_from_text(
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
        assert_eq!(gs.you.head, Coord { x: 0, y: 1 });
        assert_eq!(gs.board.snakes.len(), 0);
        assert_eq!(gs.you.eliminated, true);
        assert_eq!(gs.you.health, 0);
    }
    #[test]
    fn test_advance_eat_food_on_starve_turn() {
        let mut gs = new_gamestate_from_text(
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
        let mut gs = new_gamestate_from_text(
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
        let mut gs = new_gamestate_from_text(
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
    #[test]
    fn test_shortest_distance_basic_01() {
        let gs = new_gamestate_from_text(
            "
        |  |F |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let dist = gs.shortest_distance(&gs.you.head, &Coord { x: 1, y: 4 });
        assert_eq!(dist.unwrap(), 1);
    }
    #[test]
    fn test_shortest_distance_basic_02() {
        let gs = new_gamestate_from_text(
            "
        |  |F |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let dist = gs.shortest_distance(&gs.you.head, &Coord { x: 2, y: 0 });
        assert_eq!(dist.unwrap(), 4);
    }
    #[test]
    fn test_shortest_distance_basic_03() {
        let gs = new_gamestate_from_text(
            "
        |  |F |  |A3|H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|Y3|A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let dist = gs.shortest_distance(&gs.you.head, &Coord { x: 4, y: 4 });
        assert_eq!(dist.unwrap(), 4);
    }
    #[test]
    fn test_shortest_distance_basic_04() {
        let gs = new_gamestate_from_text(
            "
        |  |F |A4|A3|H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|Y3|A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let dist = gs.shortest_distance(&gs.you.head, &Coord { x: 4, y: 4 });
        assert_eq!(dist.unwrap(), 10);
    }
    #[test]
    fn test_shortest_distance_basic_05() {
        let gs = new_gamestate_from_text(
            "
        |  |F |A4|A3|H |        
        |  |Y0|  |A2|  |        
        |  |Y1|Y4|A1|  |        
        |  |Y2|Y3|A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let dist = gs.shortest_distance(&gs.you.head, &Coord { x: 4, y: 4 });
        assert_eq!(dist.unwrap(), 12);
    }
    #[test]
    fn test_shortest_distance_basic_06() {
        let gs = new_gamestate_from_text(
            "
        |  |F |A5|A4|H |        
        |  |Y0|  |A3|  |        
        |  |Y1|Y4|A2|  |        
        |  |Y2|Y3|A1|  |        
        |  |  |F |A0|  |        
        ",
        );
        let dist = gs.shortest_distance(&gs.you.head, &Coord { x: 4, y: 4 });
        assert_eq!(dist.is_none(), true);
    }
    #[test]
    fn test_territory_info_01() {
        let gs = new_gamestate_from_text(
            "
        |  |F |A5|A4|H |        
        |  |Y0|  |A3|  |        
        |  |Y1|Y4|A2|  |        
        |  |Y2|Y3|A1|  |        
        |  |  |F |A0|  |        
        ",
        );
        let t_info = gs.compute_territory_info();
        let controlled_squares = t_info.controlled_squares.get(&gs.you.id).unwrap();
        assert_eq!(controlled_squares.len(), 9);
        let available_squares = t_info.available_squares.get(&gs.you.id).unwrap();
        assert_eq!(available_squares.len(), 12);
    }
    #[test]
    fn test_territory_info_02() {
        let gs = new_gamestate_from_text(
            "
        |  |F |A4|A3|H |        
        |  |Y0|  |A2|  |        
        |  |Y1|Y4|A1|  |        
        |  |Y2|Y3|A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let t_info = gs.compute_territory_info();
        let controlled_squares = t_info.controlled_squares.get(&gs.you.id).unwrap();
        assert_eq!(controlled_squares.len(), 10);
        let available_squares = t_info.available_squares.get(&gs.you.id).unwrap();
        assert_eq!(available_squares.len(), 18);
    }
    #[test]
    fn test_closest_food_distance() {
        let gs = new_gamestate_from_text(
            "
        |  |F |  |  |H |        
        |  |Y0|  |A2|  |        
        |  |Y1|  |A1|  |        
        |  |Y2|  |A0|  |        
        |  |  |F |  |  |        
        ",
        );
        let dist = gs.closest_food_distance(&gs.you.head);
        assert_eq!(dist.unwrap(), 1);
    }
    #[test]
    fn test_search_basic() {
        let mut gs = new_gamestate_from_text(
            "
        |  |F |  |  |H |
        |  |Y0|  |A2|  |
        |  |Y1|  |A1|  |
        |  |Y2|  |A0|  |
        |  |  |F |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Up);
        // assert_eq!(search.best_score, 100);
    }
    #[test]
    fn test_search_solo() {
        let mut gs = new_gamestate_from_text(
            "
        |  |F |  |  |H |
        |  |Y0|  |  |  |
        |  |Y1|  |  |  |
        |  |Y2|  |  |  |
        |  |  |F |  |  |
        ",
        );
        gs.init();
        gs.game.ruleset.name = GameMode::Solo;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Up);
        // assert_eq!(search.best_score, 100);
    }
    #[test]
    fn test_search_choose_open_space_01() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |
        |  |  |  |  |  |
        |  |  |  |  |  |
        |  |  |  |Y1|Y0|
        |  |  |Y3|Y2|F |
        ",
        );
        gs.init();
        gs.game.ruleset.name = GameMode::Solo;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Up);
        // assert_eq!(search.best_score, 100);
    }
    #[test]
    fn test_search_choose_open_space_02() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |Y0|  |  |  |  |  |  |
        |Y5|Y4|Y3|Y2|Y1|  |  |  |  |  |  |
        |Y6|  |  |  |  |  |  |  |  |  |  |
        |Y7|  |  |  |  |  |  |  |  |  |  |
        |Y8|  |  |  |  |  |  |  |  |  |  |
        |Y9|  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |F |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        ",
        );
        gs.init();
        gs.game.ruleset.name = GameMode::Solo;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Right);
        // assert_eq!(search.best_score.sum(), 100);
    }
    #[test]
    fn test_search_choose_open_space_03() {
        let mut gs = new_gamestate_from_text(
            "
        |  |A1|A0|  |  |  |  |  |  |  |  |
        |  |A2|  |  |  |  |  |  |  |  |  |
        |  |A3|  |  |  |  |  |  |  |  |  |
        |  |A4|A5|  |  |  |  |  |  |  |  |
        |  |  |A6|  |  |  |  |  |  |  |  |
        |  |  |A7|A8|A9|  |  |  |  |  |  |
        |Y1|Y0|  |  |  |  |  |  |  |  |  |
        |Y2|Y3|  |  |  |  |F |  |  |  |  |
        |  |Y4|  |  |  |  |  |  |  |  |  |
        |  |Y5|Y6|  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Right);
        // assert_eq!(search.best_score.sum(), 100);
    }
    #[test]
    fn test_search_choose_open_space_04() {
        let mut gs = new_gamestate_from_text(
            "
        |  |A1|A0|  |  |  |  |  |  |  |  |
        |  |A2|  |  |  |  |  |  |  |  |  |
        |  |A3|  |  |  |  |  |  |  |  |  |
        |  |A4|A5|  |  |  |  |  |  |  |  |
        |  |  |A6|  |  |  |  |  |  |  |  |
        |  |  |A7|A8|A9|  |  |  |  |  |  |
        |Y1|Y0|  |  |  |  |  |  |  |  |  |
        |Y2|Y3|  |  |  |  |F |  |  |  |  |
        |  |Y4|  |  |  |  |  |  |  |  |  |
        |  |Y5|Y6|  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        ",
        );
        gs.init();
        for snake in gs.board.snakes.iter_mut() {
            if snake.id != gs.you.id {
                continue;
            }
            snake.health = 10;
        }
        gs.you.health = 10;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Right);
        // assert_eq!(search.best_score.sum(), 100);
    }
    #[test]
    fn test_search_choose_open_space_05() {
        let mut gs = new_gamestate_from_text(
            "
        |H |H |A9|  |H |H |H |  |Y4|H |H |
        |H |  |A8|  |A0|H |  |  |Y3|  |H |
        |  |  |A7|  |A1|F |Y0|Y1|Y2|  |  |
        |  |  |A6|  |A2|H |  |  |  |  |  |
        |H |  |A5|A4|A3|H |  |  |  |  |H |
        |H |H |  |H |H |H |H |H |  |H |H |
        |H |  |  |  |  |H |  |  |  |  |H |
        |  |  |  |  |  |H |  |Y9|  |F |  |
        |  |  |  |  |  |  |  |Y8|  |  |  |
        |H |  |  |  |  |H |  |Y7|  |  |H |
        |H |H |  |  |H |H |H |Y6|Y5|H |H |
        ",
        );
        gs.init();
        gs.game.ruleset.name = GameMode::Wrapped;
        gs.game.ruleset.settings.hazard_damage_per_turn = 100;
        for snake in gs.board.snakes.iter_mut() {
            if snake.id != gs.you.id {
                continue;
            }
            snake.health = 80;
        }
        gs.you.health = 80;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Down);
        // assert_eq!(search.best_score.sum(), 100);
    }
    #[test]
    fn test_search_cutoff_enemy_01() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |Y0|F |  |  |  |  |  |  |  |  |
        |A0|Y1|  |  |  |  |  |  |  |  |  |
        |A1|Y2|  |  |  |  |  |  |  |  |  |
        |A2|Y3|Y4|  |  |  |  |  |  |  |  |
        |A3|A4|Y5|Y6|Y7|Y8|  |  |  |  |  |
        |  |A5|A6|A7|A8|A9|  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Up);
        assert_eq!(search.best_score.sum(), i32::MAX);
    }
    #[test]
    fn test_search_cutoff_enemy_02() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |Y1|Y0|  |F |  |  |  |  |  |  |
        |A0|Y2|  |  |  |  |  |  |  |  |  |
        |A1|Y3|Y4|  |  |  |  |  |  |  |  |
        |A2|A3|Y5|Y6|Y7|Y8|  |  |  |  |  |
        |  |A4|A5|A6|A7|A8|  |  |  |  |  |
        |  |  |  |  |  |A9|  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        ",
        );
        gs.init();
        gs.game.timeout = 1000;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Up);
        assert_eq!(search.best_score.sum(), i32::MAX);
    }
    #[test]
    fn test_search_stomp() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |A1|A2|
        |  |Y0|  |A0|  |
        |  |Y1|  |  |  |
        |  |Y2|  |  |  |
        |  |Y3|  |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Right);
        // assert_eq!(search.best_score, 100);
    }
    #[test]
    fn test_search_stomp_trapped() {
        let mut gs = new_gamestate_from_text(
            "
        |A0|  |  |  |  |
        |A1|Y0|F |  |  |
        |A2|Y1|  |  |  |
        |  |Y2|  |  |  |
        |  |Y3|  |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Up);
        assert_eq!(search.best_score.sum(), i32::MAX);
    }
    #[test]
    fn test_search_avoid() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |A0|A1|A2|
        |  |Y0|  |  |  |
        |  |Y1|  |  |  |
        |  |Y2|  |  |  |
        |  |  |  |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Left);
        // assert_eq!(search.best_score, 100);
    }
    #[test]
    fn test_search_avoid_with_food() {
        let mut gs = new_gamestate_from_text(
            "
        |  |F |A0|A1|A2|
        |  |Y0|F |  |  |
        |  |Y1|  |  |  |
        |  |Y2|  |  |  |
        |  |  |  |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Left);
        // assert_eq!(search.best_score, 100);
    }
    #[test]
    fn test_search_avoid_with_food_while_starving() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |
        |  |Y0|F |  |  |
        |  |Y1|  |A0|  |
        |  |Y2|  |A1|  |
        |  |  |  |A2|  |
        ",
        );
        gs.init();
        for snake in gs.board.snakes.iter_mut() {
            if snake.id != gs.you.id {
                continue;
            }
            snake.health = 1;
        }
        gs.you.health = 1;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Right);
        // assert_eq!(search.best_score, 100);
    }
    #[test]
    fn test_search_inveitable_loss_01() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |
        |  |Y0|F |A0|  |
        |  |Y1|  |A1|  |
        |  |Y2|  |A2|  |
        |  |  |  |A3|  |
        ",
        );
        gs.init();
        for snake in gs.board.snakes.iter_mut() {
            if snake.id != gs.you.id {
                continue;
            }
            snake.health = 1;
        }
        gs.you.health = 1;
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Right);
        assert_eq!(search.best_score.sum(), i32::MIN);
    }
    #[test]
    fn test_search_meeting_of_the_minds() {
        let mut gs = new_gamestate_from_text(
            "
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |B3|B4|  |  |  |  |
        |  |  |  |  |  |B2|  |  |  |  |  |
        |  |  |  |  |  |B1|  |  |  |  |  |
        |  |  |  |  |  |B0|  |  |  |  |  |
        |  |Y3|Y2|Y1|Y0|F |C0|C1|C2|C3|C4|
        |  |  |  |  |  |A0|  |  |  |  |C5|
        |  |  |  |  |  |A1|  |  |  |  |  |
        |  |  |  |  |  |A2|  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        |  |  |  |  |  |  |  |  |  |  |  |
        ",
        );
        gs.init();
        let mut search = Search::new(&gs);
        search.iterative_deepening(&gs, 100);
        assert_eq!(search.best_direction, Direction::Down);
        // assert_eq!(search.best_score, 100);
    }
}
