#![cfg_attr(feature = "win32_pixel", allow(dead_code))]

use std::io::{self, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tile {
    Floor,
    Wall,
    Goal,
    Key,
    Door,
    Ice,
    Trap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn delta(self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Clone, Debug)]
struct Stage {
    meta: StageMeta,
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    start: Pos,
    goal: Pos,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StageMeta {
    id: &'static str,
    name: &'static str,
    rule_set: RuleSet,
    goal_type: GoalType,
    turn_limit: Option<u32>,
    hint: &'static str,
    gimmick: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuleSet {
    Classic,
    KeyDoor,
    Ice,
    Trap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GoalType {
    Reach,
}

impl Stage {
    #[cfg(test)]
    fn parse(lines: &[&str]) -> Self {
        Self::from_lines(StageMeta::default(), lines, &[])
    }

    fn parse_pack(lines: &'static [&'static str]) -> Self {
        let mut meta = StageMeta::default();
        let mut rules = [TileRule::empty(); 8];
        let mut rule_count = 0;
        let mut map_start = 0;
        for (i, line) in lines.iter().enumerate() {
            if let Some(value) = line.strip_prefix("@stage ") {
                meta.id = value;
            } else if let Some(value) = line.strip_prefix("@name ") {
                meta.name = value;
            } else if let Some(value) = line.strip_prefix("@goal ") {
                meta.goal_type = GoalType::parse(value);
            } else if let Some(value) = line.strip_prefix("@rules ") {
                meta.rule_set = RuleSet::parse(value);
            } else if let Some(value) = line.strip_prefix("@limit ") {
                meta.turn_limit = parse_u32(value);
            } else if let Some(value) = line.strip_prefix("@hint ") {
                meta.hint = value;
            } else if let Some(value) = line.strip_prefix("@gimmick ") {
                meta.gimmick = value;
            } else if let Some(value) = line.strip_prefix("@tile ") {
                if rule_count < rules.len() {
                    rules[rule_count] = TileRule::parse(value);
                    rule_count += 1;
                }
            } else if !line.trim().is_empty() && !line.starts_with('@') {
                map_start = i;
                break;
            }
        }
        Self::from_lines(meta, &lines[map_start..], &rules[..rule_count])
    }

    fn from_lines(meta: StageMeta, lines: &[&str], rules: &[TileRule]) -> Self {
        let height = lines.len();
        let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let mut tiles = vec![Tile::Wall; width * height];
        let mut start = Pos { x: 0, y: 0 };
        let mut goal = Pos { x: 0, y: 0 };

        for (y, line) in lines.iter().enumerate() {
            for (x, b) in line.bytes().enumerate() {
                let pos = Pos { x, y };
                tiles[y * width + x] = match b {
                    b'#' => Tile::Wall,
                    b'P' => {
                        start = pos;
                        Tile::Floor
                    }
                    b'G' => {
                        goal = pos;
                        Tile::Goal
                    }
                    _ => {
                        let tile = tile_from_rules(b, rules);
                        if tile == Tile::Goal {
                            goal = pos;
                        }
                        tile
                    }
                };
            }
        }

        Self {
            meta,
            width,
            height,
            tiles,
            start,
            goal,
        }
    }

    fn tile(&self, pos: Pos) -> Tile {
        if pos.x >= self.width || pos.y >= self.height {
            Tile::Wall
        } else {
            self.tiles[pos.y * self.width + pos.x]
        }
    }
}

impl StageMeta {
    fn default() -> Self {
        Self {
            id: "00",
            name: "Untitled Stage",
            rule_set: RuleSet::Classic,
            goal_type: GoalType::Reach,
            turn_limit: None,
            hint: "",
            gimmick: "",
        }
    }
}

impl RuleSet {
    fn parse(value: &str) -> Self {
        match value {
            "keydoor" => RuleSet::KeyDoor,
            "ice" => RuleSet::Ice,
            "trap" => RuleSet::Trap,
            _ => RuleSet::Classic,
        }
    }

    fn label(self) -> &'static str {
        match self {
            RuleSet::Classic => "classic",
            RuleSet::KeyDoor => "keydoor",
            RuleSet::Ice => "ice",
            RuleSet::Trap => "trap",
        }
    }
}

#[derive(Clone, Copy)]
struct TileRule {
    byte: u8,
    tile: Tile,
    active: bool,
}

impl TileRule {
    const fn empty() -> Self {
        Self {
            byte: 0,
            tile: Tile::Floor,
            active: false,
        }
    }

    fn parse(value: &str) -> Self {
        let bytes = value.as_bytes();
        if bytes.len() < 3 {
            return Self::empty();
        }
        let tile = if value.ends_with(" wall") {
            Tile::Wall
        } else if value.ends_with(" goal") {
            Tile::Goal
        } else if value.ends_with(" key") {
            Tile::Key
        } else if value.ends_with(" door") {
            Tile::Door
        } else if value.ends_with(" ice") {
            Tile::Ice
        } else if value.ends_with(" trap") {
            Tile::Trap
        } else {
            Tile::Floor
        };
        Self {
            byte: bytes[0],
            tile,
            active: true,
        }
    }
}

fn tile_from_rules(byte: u8, rules: &[TileRule]) -> Tile {
    for rule in rules {
        if rule.active && rule.byte == byte {
            return rule.tile;
        }
    }
    Tile::Floor
}

impl GoalType {
    fn parse(value: &str) -> Self {
        match value {
            "reach" => GoalType::Reach,
            _ => GoalType::Reach,
        }
    }
}

fn parse_u32(value: &str) -> Option<u32> {
    let bytes = value.as_bytes();
    let mut n = 0u32;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if !b.is_ascii_digit() {
            return None;
        }
        n = n.saturating_mul(10).saturating_add((b - b'0') as u32);
        i += 1;
    }
    Some(n)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scene {
    Title,
    Help,
    Game,
    Clear,
    GameOver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Snapshot {
    stage_index: usize,
    player: Pos,
    moves: u32,
    has_key: bool,
    trapped: bool,
}

struct GameState {
    stages: Vec<Stage>,
    stage_index: usize,
    player: Pos,
    moves: u32,
    has_key: bool,
    trapped: bool,
    history: Vec<Snapshot>,
    scene: Scene,
}

impl GameState {
    fn new(stages: Vec<Stage>) -> Self {
        let player = stages[0].start;
        Self {
            stages,
            stage_index: 0,
            player,
            moves: 0,
            has_key: false,
            trapped: false,
            history: Vec::new(),
            scene: Scene::Title,
        }
    }

    fn current_stage(&self) -> &Stage {
        &self.stages[self.stage_index]
    }

    fn start_game(&mut self) {
        self.scene = Scene::Game;
    }

    fn step(&mut self, dir: Direction) {
        self.move_player(dir);
    }

    fn move_player(&mut self, dir: Direction) {
        if matches!(self.scene, Scene::Title | Scene::Help) {
            self.scene = Scene::Game;
        }

        if !matches!(self.scene, Scene::Game) {
            return;
        }

        let (dx, dy) = dir.delta();
        let next_x = self.player.x as isize + dx;
        let next_y = self.player.y as isize + dy;
        if next_x < 0 || next_y < 0 {
            return;
        }

        let next = Pos {
            x: next_x as usize,
            y: next_y as usize,
        };
        if !self.can_enter_tile(next) {
            return;
        }

        self.history.push(Snapshot {
            stage_index: self.stage_index,
            player: self.player,
            moves: self.moves,
            has_key: self.has_key,
            trapped: self.trapped,
        });
        self.player = next;
        self.on_enter_tile(next);
        if matches!(self.scene, Scene::GameOver) {
            return;
        }
        self.after_move(dir);
    }

    fn can_enter_tile(&self, pos: Pos) -> bool {
        match self.current_stage().tile(pos) {
            Tile::Wall => false,
            Tile::Door => self.current_stage().meta.rule_set != RuleSet::KeyDoor || self.has_key,
            _ => true,
        }
    }

    fn on_enter_tile(&mut self, pos: Pos) {
        let tile = self.current_stage().tile(pos);
        match (self.current_stage().meta.rule_set, tile) {
            (RuleSet::KeyDoor, Tile::Key) => self.has_key = true,
            (RuleSet::Trap, Tile::Trap) => {
                self.trapped = true;
                self.scene = Scene::GameOver;
            }
            _ => {}
        }
    }

    fn after_move(&mut self, dir: Direction) {
        if self.current_stage().meta.rule_set == RuleSet::Ice
            && self.current_stage().tile(self.player) == Tile::Ice
        {
            self.slide_on_ice(dir);
        }
        self.moves += 1;

        if self.check_game_over() {
            self.scene = Scene::GameOver;
            return;
        }

        if self.check_clear() {
            self.advance_stage();
        }
    }

    fn slide_on_ice(&mut self, dir: Direction) {
        loop {
            let (dx, dy) = dir.delta();
            let next_x = self.player.x as isize + dx;
            let next_y = self.player.y as isize + dy;
            if next_x < 0 || next_y < 0 {
                break;
            }
            let next = Pos {
                x: next_x as usize,
                y: next_y as usize,
            };
            if !self.can_enter_tile(next) {
                break;
            }
            self.player = next;
            self.on_enter_tile(next);
            if matches!(self.scene, Scene::GameOver) {
                break;
            }
        }
    }

    fn check_clear(&self) -> bool {
        matches!(self.current_stage().meta.goal_type, GoalType::Reach)
            && (self.player == self.current_stage().goal
                || self.current_stage().tile(self.player) == Tile::Goal)
    }

    fn check_game_over(&self) -> bool {
        self.trapped
            || self
                .current_stage()
                .meta
                .turn_limit
                .is_some_and(|limit| self.moves > limit)
    }

    fn advance_stage(&mut self) {
        if self.stage_index + 1 == self.stages.len() {
            self.scene = Scene::Clear;
        } else {
            self.stage_index += 1;
            self.player = self.current_stage().start;
            self.moves = 0;
            self.has_key = false;
            self.trapped = false;
            self.history.clear();
            self.scene = Scene::Game;
        }
    }

    fn undo(&mut self) {
        if let Some(snapshot) = self.history.pop() {
            self.stage_index = snapshot.stage_index;
            self.player = snapshot.player;
            self.moves = snapshot.moves;
            self.has_key = snapshot.has_key;
            self.trapped = snapshot.trapped;
            self.scene = Scene::Game;
        }
    }

    fn reset_stage(&mut self) {
        self.player = self.current_stage().start;
        self.moves = 0;
        self.has_key = false;
        self.trapped = false;
        self.history.clear();
        self.scene = Scene::Game;
    }
}

struct Screen {
    width: usize,
    height: usize,
    cells: Vec<char>,
}

impl Screen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![' '; width * height],
        }
    }

    fn clear(&mut self) {
        self.cells.fill(' ');
    }

    fn put(&mut self, x: usize, y: usize, ch: char) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = ch;
        }
    }

    fn text(&mut self, x: usize, y: usize, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            self.put(x + i, y, ch);
        }
    }

    fn frame(&mut self) {
        for x in 0..self.width {
            self.put(x, 0, '-');
            self.put(x, self.height - 1, '-');
        }
        for y in 0..self.height {
            self.put(0, y, '|');
            self.put(self.width - 1, y, '|');
        }
        self.put(0, 0, '+');
        self.put(self.width - 1, 0, '+');
        self.put(0, self.height - 1, '+');
        self.put(self.width - 1, self.height - 1, '+');
    }

    fn draw_game(&mut self, game: &GameState) {
        let stage = game.current_stage();
        for y in 0..stage.height {
            for x in 0..stage.width {
                let pos = Pos { x, y };
                let ch = if pos == game.player {
                    '@'
                } else if pos == stage.goal {
                    'G'
                } else {
                    match stage.tile(pos) {
                        Tile::Floor => '.',
                        Tile::Wall => '#',
                        Tile::Goal => 'G',
                        Tile::Key => 'K',
                        Tile::Door => 'D',
                        Tile::Ice => 'I',
                        Tile::Trap => 'T',
                    }
                };
                self.put(x + 2, y + 2, ch);
            }
        }
        self.text(
            2,
            stage.height + 3,
            &format!(
                "stage {}/{} {} moves {}",
                game.stage_index + 1,
                game.stages.len(),
                stage.meta.name,
                game.moves
            ),
        );
        if let Some(limit) = stage.meta.turn_limit {
            self.text(
                2,
                stage.height + 4,
                &format!("limit {}/{}", game.moves, limit),
            );
        }
        if !stage.meta.hint.is_empty() {
            self.text(2, stage.height + 5, stage.meta.hint);
        }
        if !stage.meta.gimmick.is_empty() {
            self.text(
                2,
                stage.height + 6,
                &format!("gimmick {}", stage.meta.gimmick),
            );
        }
        self.text(
            2,
            stage.height + 7,
            &format!(
                "rules {} key {} trap {}",
                stage.meta.rule_set.label(),
                if game.has_key { "yes" } else { "no" },
                if game.trapped { "yes" } else { "no" }
            ),
        );
    }

    fn render(&self) -> String {
        let mut out = String::with_capacity((self.width + 1) * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                out.push(self.cells[y * self.width + x]);
            }
            out.push('\n');
        }
        out
    }
}

struct PixelLayer {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    palette: [(u8, u8, u8); 16],
}

impl PixelLayer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            palette: [
                (15, 18, 24),
                (55, 78, 107),
                (112, 146, 190),
                (230, 236, 242),
                (40, 120, 88),
                (88, 184, 112),
                (224, 196, 88),
                (208, 88, 64),
                (96, 56, 120),
                (176, 96, 176),
                (80, 160, 168),
                (224, 128, 80),
                (48, 48, 64),
                (104, 104, 128),
                (168, 168, 184),
                (248, 248, 248),
            ],
        }
    }

    fn clear(&mut self, color: u8) {
        self.pixels.fill(color & 15);
    }

    fn pixel(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            0
        }
    }

    #[cfg(feature = "win32_pixel")]
    fn bgra_word(&self, color: u8) -> u32 {
        let (r, g, b) = self.palette[(color & 15) as usize];
        (b as u32) << 16 | (g as u32) << 8 | r as u32
    }

    #[cfg(feature = "win32_pixel")]
    fn write_bgra(&self, out: &mut [u32]) {
        for (dst, color) in out.iter_mut().zip(self.pixels.iter()) {
            *dst = self.bgra_word(*color);
        }
    }

    #[cfg(any(feature = "pixel_tile", feature = "win32_pixel"))]
    fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color & 15;
        }
    }

    #[cfg(feature = "pixel_tile")]
    fn draw_tile(&mut self, x: usize, y: usize, tile: &[u8; 64]) {
        for ty in 0..8 {
            for tx in 0..8 {
                self.set_pixel(x + tx, y + ty, tile[ty * 8 + tx]);
            }
        }
    }

    #[cfg(feature = "pixel_tile")]
    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8; 256], transparent: u8) {
        for sy in 0..16 {
            for sx in 0..16 {
                let color = sprite[sy * 16 + sx];
                if color != transparent {
                    self.set_pixel(x + sx, y + sy, color);
                }
            }
        }
    }

    #[cfg(feature = "pixel_tile")]
    fn draw_tilemap(
        &mut self,
        x: usize,
        y: usize,
        map: &[u8],
        map_width: usize,
        tiles: &[[u8; 64]],
    ) {
        if map_width == 0 {
            return;
        }
        for (i, tile_index) in map.iter().enumerate() {
            let tx = i % map_width;
            let ty = i / map_width;
            if let Some(tile) = tiles.get(*tile_index as usize) {
                self.draw_tile(x + tx * 8, y + ty * 8, tile);
            }
        }
    }

    fn terminal_preview(&self, max_w: usize, max_h: usize) -> String {
        let mut out = String::new();
        let h = self.height.min(max_h);
        let w = self.width.min(max_w);
        for y in 0..h {
            for x in 0..w {
                let color = self.pixel(x, y) as usize;
                let bright = self.palette[color].0 as u16
                    + self.palette[color].1 as u16
                    + self.palette[color].2 as u16;
                out.push(match bright {
                    0..=120 => ' ',
                    121..=300 => '.',
                    301..=560 => '*',
                    _ => '#',
                });
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(feature = "win32_pixel")]
#[derive(Clone, Copy, Default)]
struct ProbeInput {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

#[cfg(feature = "win32_pixel")]
const PROBE_LAYER_WIDTH: usize = 160;
#[cfg(feature = "win32_pixel")]
const PROBE_LAYER_HEIGHT: usize = 144;
#[cfg(feature = "win32_pixel")]
const PROBE_SPRITE_SIZE: usize = 16;
#[cfg(feature = "win32_pixel")]
const PROBE_TICK_HZ_LABEL: &str = "60Hz-ish tick";
#[cfg(feature = "win32_pixel")]
const PROBE_TILE_SIZE: usize = 8;
#[cfg(feature = "win32_pixel")]
const PROBE_MAP_WIDTH: usize = 21;
#[cfg(feature = "win32_pixel")]
const PROBE_MAP_HEIGHT: usize = 18;

#[cfg(feature = "win32_pixel")]
struct ProbeGame {
    sprite_x: usize,
    sprite_y: usize,
    tick_count: u32,
}

#[cfg(feature = "win32_pixel")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProbeCamera {
    x: usize,
    y: usize,
}

#[cfg(feature = "win32_pixel")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProbeTile {
    Floor,
    Wall,
    Goal,
}

#[cfg(feature = "win32_pixel")]
impl ProbeGame {
    fn new() -> Self {
        Self {
            sprite_x: 72,
            sprite_y: 60,
            tick_count: 0,
        }
    }

    fn tick(&mut self, input: ProbeInput) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.sprite_x = self.sprite_x.min(probe_walk_width() - PROBE_SPRITE_SIZE);
        self.sprite_y = self.sprite_y.min(probe_walk_height() - PROBE_SPRITE_SIZE);
        let next_x = step_axis(self.sprite_x, input.left, input.right, probe_walk_width());
        let next_y = step_axis(self.sprite_y, input.up, input.down, probe_walk_height());
        if !probe_hits_wall(next_x, self.sprite_y) {
            self.sprite_x = next_x;
        }
        if !probe_hits_wall(self.sprite_x, next_y) {
            self.sprite_y = next_y;
        }
    }

    fn camera(&self) -> ProbeCamera {
        ProbeCamera {
            x: camera_axis(
                self.sprite_x,
                PROBE_LAYER_WIDTH,
                probe_world_width(),
                PROBE_SPRITE_SIZE,
            ),
            y: camera_axis(
                self.sprite_y,
                PROBE_LAYER_HEIGHT,
                probe_world_height(),
                PROBE_SPRITE_SIZE,
            ),
        }
    }

    fn title(&self) -> String {
        format!(
            "win32-pixel-probe v0.3 | {}x{} | {} | pos {},{} | tick {}",
            PROBE_LAYER_WIDTH,
            PROBE_LAYER_HEIGHT,
            PROBE_TICK_HZ_LABEL,
            self.sprite_x,
            self.sprite_y,
            self.tick_count
        )
    }
}

#[cfg(feature = "win32_pixel")]
fn step_axis(value: usize, negative: bool, positive: bool, limit: usize) -> usize {
    let max = limit.saturating_sub(PROBE_SPRITE_SIZE);
    if positive && !negative {
        value.saturating_add(1).min(max)
    } else if negative && !positive {
        value.saturating_sub(1).min(max)
    } else {
        value.min(max)
    }
}

#[cfg(feature = "win32_pixel")]
fn camera_axis(sprite: usize, viewport: usize, world: usize, sprite_size: usize) -> usize {
    if world <= viewport {
        return 0;
    }
    let center = sprite.saturating_add(sprite_size / 2);
    center
        .saturating_sub(viewport / 2)
        .min(world.saturating_sub(viewport))
}

#[cfg(feature = "win32_pixel")]
fn probe_world_width() -> usize {
    PROBE_MAP_WIDTH * PROBE_TILE_SIZE
}

#[cfg(feature = "win32_pixel")]
fn probe_world_height() -> usize {
    PROBE_MAP_HEIGHT * PROBE_TILE_SIZE
}

#[cfg(feature = "win32_pixel")]
fn probe_walk_width() -> usize {
    probe_world_width().saturating_sub(PROBE_TILE_SIZE)
}

#[cfg(feature = "win32_pixel")]
fn probe_walk_height() -> usize {
    probe_world_height().saturating_sub(PROBE_TILE_SIZE)
}

#[cfg(feature = "win32_pixel")]
fn probe_tile_at(tx: usize, ty: usize) -> ProbeTile {
    if tx >= PROBE_MAP_WIDTH || ty >= PROBE_MAP_HEIGHT {
        return ProbeTile::Wall;
    }
    if tx == 17 && ty == 16 {
        return ProbeTile::Goal;
    }
    if tx == 0
        || ty == 0
        || tx + 1 == PROBE_MAP_WIDTH
        || ty + 1 == PROBE_MAP_HEIGHT
        || (ty == 2 && (tx == 1 || tx == 2 || tx == 3))
        || (tx == 7 && (4..=13).contains(&ty))
        || (ty == 10 && (9..=16).contains(&tx))
        || (tx == 15 && (3..=7).contains(&ty))
    {
        ProbeTile::Wall
    } else {
        ProbeTile::Floor
    }
}

#[cfg(feature = "win32_pixel")]
fn probe_hits_wall(x: usize, y: usize) -> bool {
    let right = x + PROBE_SPRITE_SIZE - 1;
    let bottom = y + PROBE_SPRITE_SIZE - 1;
    let points = [(x, y), (right, y), (x, bottom), (right, bottom)];
    points.iter().any(|(px, py)| {
        matches!(
            probe_tile_at(px / PROBE_TILE_SIZE, py / PROBE_TILE_SIZE),
            ProbeTile::Wall
        )
    })
}

#[cfg(feature = "win32_pixel")]
fn render_win32_probe_frame(layer: &mut PixelLayer, frame: u32, input: ProbeInput) {
    let mut game = ProbeGame::new();
    game.tick_count = frame;
    game.tick(input);
    render_probe_scene(layer, &game);
}

#[cfg(feature = "win32_pixel")]
fn render_probe_scene(layer: &mut PixelLayer, game: &ProbeGame) {
    layer.clear(0);
    let camera = game.camera();

    for y in 0..layer.height {
        for x in 0..layer.width {
            let wx = x + camera.x;
            let wy = y + camera.y;
            let tx = wx / PROBE_TILE_SIZE;
            let ty = wy / PROBE_TILE_SIZE;
            let color = match probe_tile_at(tx, ty) {
                ProbeTile::Wall => 13,
                ProbeTile::Goal => {
                    if (game.tick_count / 12) & 1 == 0 {
                        6
                    } else {
                        15
                    }
                }
                ProbeTile::Floor => {
                    if ((tx + ty) & 1) == 0 {
                        1
                    } else {
                        2
                    }
                }
            };
            layer.set_pixel(x, y, color);
        }
    }

    draw_probe_hud(layer, game);

    let mut sprite = [0u8; 256];
    for y in 0..16 {
        for x in 0..16 {
            let edge = x == 0 || y == 0 || x == 15 || y == 15;
            let core = (4..12).contains(&x) && (4..12).contains(&y);
            sprite[y * 16 + x] = if edge {
                7
            } else if core {
                15
            } else {
                0
            };
        }
    }

    layer.draw_sprite(
        game.sprite_x.saturating_sub(camera.x),
        game.sprite_y.saturating_sub(camera.y),
        &sprite,
        0,
    );
}

#[cfg(feature = "win32_pixel")]
fn draw_probe_hud(layer: &mut PixelLayer, game: &ProbeGame) {
    for y in 0..16 {
        for x in 0..layer.width {
            layer.set_pixel(x, y, if y == 15 { 14 } else { 12 });
        }
    }

    let pulse = ((game.tick_count / 8) & 7) as usize;
    for i in 0..8 {
        let color = if i <= pulse { 6 } else { 13 };
        for y in 4..12 {
            layer.set_pixel(8 + i * 5, y, color);
        }
    }

    let camera = game.camera();
    let marker_x = 128 + (camera.x * 24 / (probe_world_width() - PROBE_LAYER_WIDTH).max(1));
    for y in 5..11 {
        layer.set_pixel(marker_x.min(PROBE_LAYER_WIDTH - 1), y, 10);
    }
}

#[cfg(all(feature = "win32_pixel", windows))]
mod win32_pixel {
    use super::{PixelLayer, ProbeInput};
    use std::ffi::c_void;
    use std::io;
    use std::mem::{size_of, zeroed};
    use std::ptr::{null, null_mut};

    type Hinstance = *mut c_void;
    type Hwnd = *mut c_void;
    type Hdc = *mut c_void;
    type Hicon = *mut c_void;
    type Hcursor = *mut c_void;
    type Hbrush = *mut c_void;
    type Lpcwstr = *const u16;
    type Lparam = isize;
    type Wparam = usize;
    type Lresult = isize;
    type Uint = u32;
    type Dword = u32;
    type Long = i32;
    type Bool = i32;

    const WIDTH: usize = super::PROBE_LAYER_WIDTH;
    const HEIGHT: usize = super::PROBE_LAYER_HEIGHT;
    const SCALE: i32 = 4;
    const TIMER_ID: usize = 1;
    const TIMER_MS: u32 = 16;

    const CS_VREDRAW: Uint = 0x0001;
    const CS_HREDRAW: Uint = 0x0002;
    const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;
    const WS_OVERLAPPEDWINDOW: Dword = 0x00cf_0000;
    const WS_VISIBLE: Dword = 0x1000_0000;
    const SW_SHOW: i32 = 5;

    const WM_DESTROY: Uint = 0x0002;
    const WM_PAINT: Uint = 0x000f;
    const WM_CLOSE: Uint = 0x0010;
    const WM_KEYDOWN: Uint = 0x0100;
    const WM_KEYUP: Uint = 0x0101;
    const WM_TIMER: Uint = 0x0113;

    const VK_ESCAPE: Wparam = 0x1b;
    const VK_LEFT: Wparam = 0x25;
    const VK_UP: Wparam = 0x26;
    const VK_RIGHT: Wparam = 0x27;
    const VK_DOWN: Wparam = 0x28;
    const VK_A: Wparam = 0x41;
    const VK_D: Wparam = 0x44;
    const VK_S: Wparam = 0x53;
    const VK_W: Wparam = 0x57;

    const BI_RGB: Dword = 0;
    const DIB_RGB_COLORS: Uint = 0;
    const SRCCOPY: Dword = 0x00cc_0020;

    #[repr(C)]
    struct WndClassW {
        style: Uint,
        lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: Hinstance,
        h_icon: Hicon,
        h_cursor: Hcursor,
        hbr_background: Hbrush,
        lpsz_menu_name: Lpcwstr,
        lpsz_class_name: Lpcwstr,
    }

    #[repr(C)]
    struct Point {
        x: Long,
        y: Long,
    }

    #[repr(C)]
    struct Msg {
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
        time: Dword,
        pt: Point,
    }

    #[repr(C)]
    struct Rect {
        left: Long,
        top: Long,
        right: Long,
        bottom: Long,
    }

    #[repr(C)]
    struct PaintStruct {
        hdc: Hdc,
        f_erase: Bool,
        rc_paint: Rect,
        f_restore: Bool,
        f_inc_update: Bool,
        rgb_reserved: [u8; 32],
    }

    #[repr(C)]
    struct BitmapInfoHeader {
        bi_size: Dword,
        bi_width: Long,
        bi_height: Long,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: Dword,
        bi_size_image: Dword,
        bi_x_pels_per_meter: Long,
        bi_y_pels_per_meter: Long,
        bi_clr_used: Dword,
        bi_clr_important: Dword,
    }

    #[repr(C)]
    struct RgbQuad {
        rgb_blue: u8,
        rgb_green: u8,
        rgb_red: u8,
        rgb_reserved: u8,
    }

    #[repr(C)]
    struct BitmapInfo {
        bmi_header: BitmapInfoHeader,
        bmi_colors: [RgbQuad; 1],
    }

    struct ProbeWindow {
        layer: PixelLayer,
        bgra: Vec<u32>,
        input: ProbeInput,
        game: super::ProbeGame,
    }

    impl ProbeWindow {
        fn new() -> Self {
            Self {
                layer: PixelLayer::new(WIDTH, HEIGHT),
                bgra: vec![0; WIDTH * HEIGHT],
                input: ProbeInput::default(),
                game: super::ProbeGame::new(),
            }
        }

        fn update(&mut self) {
            self.game.tick(self.input);
            super::render_probe_scene(&mut self.layer, &self.game);
            self.layer.write_bgra(&mut self.bgra);
        }

        fn title(&self) -> Vec<u16> {
            wide(&self.game.title())
        }
    }

    static mut PROBE_WINDOW: *mut ProbeWindow = null_mut();

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassW(lp_wnd_class: *const WndClassW) -> u16;
        fn CreateWindowExW(
            dw_ex_style: Dword,
            lp_class_name: Lpcwstr,
            lp_window_name: Lpcwstr,
            dw_style: Dword,
            x: i32,
            y: i32,
            n_width: i32,
            n_height: i32,
            h_wnd_parent: Hwnd,
            h_menu: *mut c_void,
            h_instance: Hinstance,
            lp_param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn DestroyWindow(hwnd: Hwnd) -> Bool;
        fn DispatchMessageW(lp_msg: *const Msg) -> Lresult;
        fn GetMessageW(lp_msg: *mut Msg, hwnd: Hwnd, min: Uint, max: Uint) -> Bool;
        fn GetModuleHandleW(lp_module_name: Lpcwstr) -> Hinstance;
        fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
        fn LoadCursorW(h_instance: Hinstance, lp_cursor_name: Lpcwstr) -> Hcursor;
        fn PostQuitMessage(exit_code: i32);
        fn SetTimer(hwnd: Hwnd, id_event: usize, elapse: Uint, timer_func: *mut c_void) -> usize;
        fn SetWindowTextW(hwnd: Hwnd, lp_string: Lpcwstr) -> Bool;
        fn ShowWindow(hwnd: Hwnd, n_cmd_show: i32) -> Bool;
        fn TranslateMessage(lp_msg: *const Msg) -> Bool;
        fn UpdateWindow(hwnd: Hwnd) -> Bool;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn BeginPaint(hwnd: Hwnd, lp_paint: *mut PaintStruct) -> Hdc;
        fn EndPaint(hwnd: Hwnd, lp_paint: *const PaintStruct) -> Bool;
        fn StretchDIBits(
            hdc: Hdc,
            x_dest: i32,
            y_dest: i32,
            dest_width: i32,
            dest_height: i32,
            x_src: i32,
            y_src: i32,
            src_width: i32,
            src_height: i32,
            bits: *const c_void,
            bmi: *const BitmapInfo,
            usage: Uint,
            rop: Dword,
        ) -> i32;
    }

    pub fn run() -> io::Result<()> {
        let class_name = wide("Win32PixelProbe");
        let mut state = Box::new(ProbeWindow::new());
        state.update();
        let title = state.title();

        unsafe {
            PROBE_WINDOW = state.as_mut();
            let instance = GetModuleHandleW(null());
            let class = WndClassW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfn_wnd_proc: Some(window_proc),
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: instance,
                h_icon: null_mut(),
                h_cursor: LoadCursorW(null_mut(), 32512usize as Lpcwstr),
                hbr_background: null_mut(),
                lpsz_menu_name: null(),
                lpsz_class_name: class_name.as_ptr(),
            };

            if RegisterClassW(&class) == 0 {
                return Err(io::Error::last_os_error());
            }

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                WIDTH as i32 * SCALE + 16,
                HEIGHT as i32 * SCALE + 39,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            );
            if hwnd.is_null() {
                return Err(io::Error::last_os_error());
            }

            if SetTimer(hwnd, TIMER_ID, TIMER_MS, null_mut()) == 0 {
                return Err(io::Error::last_os_error());
            }
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            let mut msg: Msg = zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            PROBE_WINDOW = null_mut();
        }

        Ok(())
    }

    unsafe extern "system" fn window_proc(
        hwnd: Hwnd,
        msg: Uint,
        w_param: Wparam,
        l_param: Lparam,
    ) -> Lresult {
        match msg {
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_KEYDOWN | WM_KEYUP => {
                if w_param == VK_ESCAPE && msg == WM_KEYDOWN {
                    DestroyWindow(hwnd);
                    return 0;
                }
                if let Some(state) = PROBE_WINDOW.as_mut() {
                    let pressed = msg == WM_KEYDOWN;
                    match w_param {
                        VK_LEFT | VK_A => state.input.left = pressed,
                        VK_RIGHT | VK_D => state.input.right = pressed,
                        VK_UP | VK_W => state.input.up = pressed,
                        VK_DOWN | VK_S => state.input.down = pressed,
                        _ => {}
                    }
                }
                0
            }
            WM_TIMER => {
                if let Some(state) = PROBE_WINDOW.as_mut() {
                    state.update();
                    let title = state.title();
                    SetWindowTextW(hwnd, title.as_ptr());
                }
                InvalidateRect(hwnd, null(), 0);
                0
            }
            WM_PAINT => {
                let mut ps: PaintStruct = zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if let Some(state) = PROBE_WINDOW.as_mut() {
                    let bmi = bitmap_info();
                    StretchDIBits(
                        hdc,
                        0,
                        0,
                        WIDTH as i32 * SCALE,
                        HEIGHT as i32 * SCALE,
                        0,
                        0,
                        WIDTH as i32,
                        HEIGHT as i32,
                        state.bgra.as_ptr().cast(),
                        &bmi,
                        DIB_RGB_COLORS,
                        SRCCOPY,
                    );
                }
                EndPaint(hwnd, &ps);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }

    fn bitmap_info() -> BitmapInfo {
        BitmapInfo {
            bmi_header: BitmapInfoHeader {
                bi_size: size_of::<BitmapInfoHeader>() as Dword,
                bi_width: WIDTH as Long,
                bi_height: -(HEIGHT as Long),
                bi_planes: 1,
                bi_bit_count: 32,
                bi_compression: BI_RGB,
                bi_size_image: (WIDTH * HEIGHT * 4) as Dword,
                bi_x_pels_per_meter: 0,
                bi_y_pels_per_meter: 0,
                bi_clr_used: 0,
                bi_clr_important: 0,
            },
            bmi_colors: [RgbQuad {
                rgb_blue: 0,
                rgb_green: 0,
                rgb_red: 0,
                rgb_reserved: 0,
            }],
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain([0]).collect()
    }
}

#[cfg(all(feature = "win32_pixel", not(windows)))]
mod win32_pixel {
    use std::io;

    pub fn run() -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "win32_pixel requires Windows",
        ))
    }
}

#[cfg(feature = "pixel_sound")]
#[derive(Clone, Copy)]
struct SfxStep {
    hz: u16,
    frames: u8,
}

#[cfg(feature = "pixel_sound")]
struct SoundData {
    win: &'static [SfxStep],
    walk: &'static [SfxStep],
    bgm: &'static str,
}

#[cfg(feature = "pixel_sound")]
const SOUND: SoundData = SoundData {
    win: &[
        SfxStep { hz: 660, frames: 5 },
        SfxStep { hz: 880, frames: 7 },
        SfxStep {
            hz: 1320,
            frames: 10,
        },
    ],
    walk: &[
        SfxStep { hz: 220, frames: 2 },
        SfxStep { hz: 180, frames: 2 },
    ],
    bgm: "T120 O4 C8 E8 G8 C5",
};

#[cfg(feature = "asset_tiles")]
static CAPACITY_TILES: [[u8; 64]; 64] = make_capacity_tiles();

#[cfg(feature = "asset_sprites")]
static CAPACITY_SPRITES: [[u8; 256]; 16] = make_capacity_sprites();

#[cfg(feature = "asset_maps")]
static CAPACITY_MAPS: [[u8; 20 * 15]; 10] = make_capacity_maps();

#[cfg(feature = "asset_sound")]
static CAPACITY_BGM: [&str; 3] = [
    "T120 O4 C8 D8 E8 G8 A8 G8 E8 D8 C4 R8",
    "T096 O3 A8 C4 E8 A8 G8 E8 C8 A4 R8",
    "T144 O5 G16 E16 C16 D16 E8 G8 C6 R8 G5",
];

#[cfg(feature = "asset_sound")]
static CAPACITY_SE: [&[SfxStep]; 8] = [
    &[
        SfxStep { hz: 110, frames: 3 },
        SfxStep { hz: 165, frames: 3 },
    ],
    &[SfxStep { hz: 220, frames: 2 }, SfxStep { hz: 0, frames: 1 }],
    &[
        SfxStep { hz: 330, frames: 4 },
        SfxStep { hz: 440, frames: 4 },
    ],
    &[
        SfxStep { hz: 550, frames: 2 },
        SfxStep { hz: 660, frames: 2 },
    ],
    &[SfxStep { hz: 880, frames: 6 }],
    &[
        SfxStep { hz: 440, frames: 2 },
        SfxStep { hz: 220, frames: 5 },
    ],
    &[
        SfxStep {
            hz: 1320,
            frames: 1,
        },
        SfxStep { hz: 660, frames: 3 },
    ],
    &[SfxStep { hz: 75, frames: 8 }, SfxStep { hz: 55, frames: 8 }],
];

#[cfg(feature = "asset_tiles")]
const fn make_capacity_tiles() -> [[u8; 64]; 64] {
    let mut tiles = [[0u8; 64]; 64];
    let mut t = 0;
    while t < 64 {
        let mut p = 0;
        while p < 64 {
            let x = p % 8;
            let y = p / 8;
            tiles[t][p] = ((t + x * 3 + y * 5 + (x ^ y)) & 15) as u8;
            p += 1;
        }
        t += 1;
    }
    tiles
}

#[cfg(feature = "asset_sprites")]
const fn make_capacity_sprites() -> [[u8; 256]; 16] {
    let mut sprites = [[0u8; 256]; 16];
    let mut s = 0;
    while s < 16 {
        let mut p = 0;
        while p < 256 {
            let x = p % 16;
            let y = p / 16;
            let edge = x == 0 || y == 0 || x == 15 || y == 15;
            sprites[s][p] = if edge {
                ((s + 1) & 15) as u8
            } else {
                ((s * 2 + x + y * 2) & 15) as u8
            };
            p += 1;
        }
        s += 1;
    }
    sprites
}

#[cfg(feature = "asset_maps")]
const fn make_capacity_maps() -> [[u8; 20 * 15]; 10] {
    let mut maps = [[0u8; 20 * 15]; 10];
    let mut m = 0;
    while m < 10 {
        let mut p = 0;
        while p < 20 * 15 {
            let x = p % 20;
            let y = p / 20;
            let border = x == 0 || y == 0 || x == 19 || y == 14;
            maps[m][p] = if border {
                1
            } else {
                ((m * 7 + x * 2 + y * 3) & 63) as u8
            };
            p += 1;
        }
        m += 1;
    }
    maps
}

fn capacity_demo_checksum() -> u64 {
    #[cfg(not(any(
        feature = "asset_tiles",
        feature = "asset_sprites",
        feature = "asset_maps",
        feature = "asset_sound"
    )))]
    {
        144
    }

    #[cfg(any(
        feature = "asset_tiles",
        feature = "asset_sprites",
        feature = "asset_maps",
        feature = "asset_sound"
    ))]
    {
        let mut sum = 144u64;
        #[cfg(feature = "asset_tiles")]
        for tile in std::hint::black_box(&CAPACITY_TILES).iter() {
            for px in tile.iter() {
                sum = sum.wrapping_mul(31).wrapping_add(*px as u64 + 1);
            }
        }

        #[cfg(feature = "asset_sprites")]
        for sprite in std::hint::black_box(&CAPACITY_SPRITES).iter() {
            for px in sprite.iter() {
                sum = sum.wrapping_mul(33).wrapping_add(*px as u64 + 1);
            }
        }

        #[cfg(feature = "asset_maps")]
        for map in std::hint::black_box(&CAPACITY_MAPS).iter() {
            for tile in map.iter() {
                sum = sum.wrapping_mul(37).wrapping_add(*tile as u64 + 1);
            }
        }

        #[cfg(feature = "asset_sound")]
        {
            for song in std::hint::black_box(&CAPACITY_BGM).iter() {
                for b in song.bytes() {
                    sum = sum.wrapping_mul(39).wrapping_add(b as u64 + 1);
                }
            }
            for se in std::hint::black_box(&CAPACITY_SE).iter() {
                for step in se.iter() {
                    sum = sum
                        .wrapping_mul(41)
                        .wrapping_add(step.hz as u64 + step.frames as u64);
                }
            }
        }

        sum
    }
}

const STAGES: &[&[&str]] = &[
    &[
        "@stage 01",
        "@name First Step",
        "@goal reach",
        "@rules classic",
        "@limit 12",
        "@hint Walk straight to the window.",
        "@gimmick warmup",
        "",
        "#######",
        "#P...G#",
        "#######",
    ],
    &[
        "@stage 02",
        "@name Corner Light",
        "@goal reach",
        "@rules classic",
        "@limit 16",
        "@hint Turn once, then keep going.",
        "@gimmick corner",
        "",
        "#######",
        "#P....#",
        "####..#",
        "#G....#",
        "#######",
    ],
    &[
        "@stage 03",
        "@name Quiet Wall",
        "@goal reach",
        "@rules classic",
        "@limit 20",
        "@hint The wall teaches the route.",
        "@gimmick wall-read",
        "",
        "########",
        "#P..#G.#",
        "#...#..#",
        "#......#",
        "########",
    ],
    &[
        "@stage 04",
        "@name First Window",
        "@goal reach",
        "@rules classic",
        "@limit 24",
        "@hint The shortest-looking path is closed.",
        "@gimmick detour",
        "",
        "#########",
        "#P..#...#",
        "#.#.#.#G#",
        "#.......#",
        "#########",
    ],
    &[
        "@stage 05",
        "@name Brass Key",
        "@goal reach",
        "@rules keydoor",
        "@limit 18",
        "@hint Take K, then open D.",
        "@gimmick keydoor",
        "@tile K key",
        "@tile D door",
        "",
        "#########",
        "#P.K.D.G#",
        "#########",
    ],
    &[
        "@stage 06",
        "@name Locked Bend",
        "@goal reach",
        "@rules keydoor",
        "@limit 24",
        "@hint The door waits after the bend.",
        "@gimmick keydoor",
        "@tile K key",
        "@tile D door",
        "",
        "##########",
        "#P..D...G#",
        "#.####.#.#",
        "#K.....#.#",
        "##########",
    ],
    &[
        "@stage 07",
        "@name Spare Key",
        "@goal reach",
        "@rules keydoor",
        "@limit 28",
        "@hint The key is not on the direct line.",
        "@gimmick side-key",
        "@tile K key",
        "@tile D door",
        "",
        "##########",
        "#P....D.G#",
        "#.####...#",
        "#K.......#",
        "##########",
    ],
    &[
        "@stage 08",
        "@name Double Lock",
        "@goal reach",
        "@rules keydoor",
        "@limit 32",
        "@hint One key opens every D.",
        "@gimmick double-door",
        "@tile K key",
        "@tile D door",
        "",
        "###########",
        "#P.K.D.DG#",
        "#.......##",
        "##########",
    ],
    &[
        "@stage 09",
        "@name Cold Pane",
        "@goal reach",
        "@rules ice",
        "@limit 10",
        "@hint I slides until the wall says stop.",
        "@gimmick ice",
        "@tile I ice",
        "",
        "########",
        "#PII..G#",
        "########",
    ],
    &[
        "@stage 10",
        "@name North Wind",
        "@goal reach",
        "@rules ice",
        "@limit 14",
        "@hint Approach the ice from below.",
        "@gimmick ice-turn",
        "@tile I ice",
        "",
        "########",
        "#....G.#",
        "#.####.#",
        "#P.I...#",
        "########",
    ],
    &[
        "@stage 11",
        "@name Long Slide",
        "@goal reach",
        "@rules ice",
        "@limit 16",
        "@hint The slide is longer than it looks.",
        "@gimmick long-ice",
        "@tile I ice",
        "",
        "##########",
        "#PIIII..G#",
        "#........#",
        "##########",
    ],
    &[
        "@stage 12",
        "@name Ice Pocket",
        "@goal reach",
        "@rules ice",
        "@limit 18",
        "@hint Stop at the pocket, then turn.",
        "@gimmick pocket",
        "@tile I ice",
        "",
        "##########",
        "#P.I....G#",
        "#..###...#",
        "#........#",
        "##########",
    ],
    &[
        "@stage 13",
        "@name Last Frost",
        "@goal reach",
        "@rules ice",
        "@limit 20",
        "@hint Use the lower lane to aim the slide.",
        "@gimmick aim",
        "@tile I ice",
        "",
        "###########",
        "#P..#....G#",
        "#...#I....#",
        "#.........#",
        "###########",
    ],
    &[
        "@stage 14",
        "@name Red Floor",
        "@goal reach",
        "@rules trap",
        "@limit 14",
        "@hint T ends the run.",
        "@gimmick trap",
        "@tile T trap",
        "",
        "########",
        "#P.T.G#",
        "#.....#",
        "########",
    ],
    &[
        "@stage 15",
        "@name Safe Edge",
        "@goal reach",
        "@rules trap",
        "@limit 18",
        "@hint The edge is safer than the center.",
        "@gimmick edge",
        "@tile T trap",
        "",
        "#########",
        "#P..T..G#",
        "#.......#",
        "#########",
    ],
    &[
        "@stage 16",
        "@name Red Cross",
        "@goal reach",
        "@rules trap",
        "@limit 22",
        "@hint Read the red cross before moving.",
        "@gimmick cross",
        "@tile T trap",
        "",
        "##########",
        "#P...T..G#",
        "#..TTT...#",
        "#........#",
        "##########",
    ],
    &[
        "@stage 17",
        "@name Last Trap",
        "@goal reach",
        "@rules trap",
        "@limit 24",
        "@hint There is room around the danger.",
        "@gimmick bypass",
        "@tile T trap",
        "",
        "###########",
        "#P..T....G#",
        "#.#####...#",
        "#.........#",
        "###########",
    ],
    &[
        "@stage 18",
        "@name Memory Key",
        "@goal reach",
        "@rules keydoor",
        "@limit 34",
        "@hint It feels mixed, but the key still rules.",
        "@gimmick finale-key",
        "@tile K key",
        "@tile D door",
        "@tile T wall",
        "",
        "############",
        "#P.K..T.DG#",
        "#...TTT...#",
        "#.........#",
        "############",
    ],
    &[
        "@stage 19",
        "@name Memory Ice",
        "@goal reach",
        "@rules ice",
        "@limit 26",
        "@hint The old window trick returns as ice.",
        "@gimmick finale-ice",
        "@tile I ice",
        "@tile T wall",
        "",
        "############",
        "#P.I..T..G#",
        "#...TTT...#",
        "#.........#",
        "############",
    ],
    &[
        "@stage 20",
        "@name Final Window",
        "@goal reach",
        "@rules trap",
        "@limit 30",
        "@hint One calm path remains.",
        "@gimmick finale-trap",
        "@tile T trap",
        "",
        "############",
        "#P..T....G#",
        "#.TTT.TTT.#",
        "#.........#",
        "############",
    ],
];

fn demo_stages() -> Vec<Stage> {
    STAGES
        .iter()
        .map(|lines| Stage::parse_pack(lines))
        .collect()
}

fn read_input() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn parse_direction(input: &str) -> Option<Direction> {
    match input.as_bytes() {
        [b'w', ..] | [b'W', ..] | [27, 91, 65, ..] => Some(Direction::Up),
        [b's', ..] | [b'S', ..] | [27, 91, 66, ..] => Some(Direction::Down),
        [b'a', ..] | [b'A', ..] | [27, 91, 68, ..] => Some(Direction::Left),
        [b'd', ..] | [b'D', ..] | [27, 91, 67, ..] => Some(Direction::Right),
        _ => None,
    }
}

fn draw_scene(screen: &mut Screen, game: &GameState) {
    screen.clear();
    screen.frame();
    match game.scene {
        Scene::Title => {
            screen.text(3, 2, "FIRST WINDOW");
            screen.text(3, 4, "A 1.44MB puzzle made with MadoCore 144");
            screen.text(3, 6, "Enter: start  H: help  Q: quit");
        }
        Scene::Help => {
            screen.text(3, 2, "Reach G in 20 one-screen puzzles.");
            screen.text(3, 3, "K opens D. I slides. T ends the run.");
            screen.text(3, 4, "WASD/arrows: move  U: undo  R: reset");
            screen.text(3, 5, "Enter: start  Q/Esc: quit");
        }
        Scene::Game => screen.draw_game(game),
        Scene::Clear => {
            screen.text(3, 2, "The first window opens.");
            screen.text(3, 4, "You carried the light through every pane.");
            screen.text(3, 6, "Q: quit  R: restart");
        }
        Scene::GameOver => {
            screen.text(3, 2, "Game over");
            screen.text(3, 4, "Q: quit");
        }
    }
}

fn print_screen(screen: &Screen) -> io::Result<()> {
    let mut out = io::stdout();
    write!(out, "\x1B[2J\x1B[H{}", screen.render())?;
    out.flush()
}

#[cfg(not(feature = "win32_pixel"))]
fn main() -> io::Result<()> {
    run_terminal_game()
}

#[cfg(feature = "win32_pixel")]
fn main() -> io::Result<()> {
    win32_pixel::run()
}

#[cfg(not(feature = "win32_pixel"))]
fn run_terminal_game() -> io::Result<()> {
    let mut game = GameState::new(demo_stages());
    let mut screen = Screen::new(42, 16);
    let mut pixels = PixelLayer::new(160, 144);
    pixels.clear(0);
    #[cfg(feature = "pixel_tile")]
    {
        pixels.draw_tilemap(0, 0, &[], 1, &[]);
        pixels.draw_sprite(0, 0, &[0; 256], 0);
    }
    let _preview = pixels.terminal_preview(1, 1);
    let _scene_count = [
        Scene::Title,
        Scene::Help,
        Scene::Game,
        Scene::Clear,
        Scene::GameOver,
    ]
    .len();
    #[cfg(feature = "pixel_sound")]
    let _sound_bytes = SOUND.bgm.len()
        + SOUND
            .win
            .iter()
            .map(|step| step.frames as usize + step.hz as usize / 1000)
            .sum::<usize>()
        + SOUND
            .walk
            .iter()
            .map(|step| step.frames as usize + step.hz as usize / 1000)
            .sum::<usize>();
    let _capacity_bytes = std::hint::black_box(capacity_demo_checksum());

    loop {
        draw_scene(&mut screen, &game);
        print_screen(&screen)?;
        print!("> ");
        io::stdout().flush()?;

        let input = read_input()?;
        let trimmed = input.trim();

        if trimmed.eq_ignore_ascii_case("q") || input.as_bytes().first() == Some(&27) {
            break;
        }

        if trimmed.eq_ignore_ascii_case("h") {
            game.scene = Scene::Help;
        } else if trimmed.eq_ignore_ascii_case("r") {
            if matches!(game.scene, Scene::Clear | Scene::GameOver) {
                game = GameState::new(demo_stages());
                game.start_game();
            } else {
                game.reset_stage();
            }
        } else if trimmed.eq_ignore_ascii_case("u") {
            game.undo();
        } else if trimmed.is_empty() {
            game.start_game();
        } else if let Some(dir) = parse_direction(&input) {
            game.step(dir);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stage_from_code_strings() {
        let stage = Stage::parse(&["#####", "#P.G#", "#####"]);
        assert_eq!(stage.width, 5);
        assert_eq!(stage.height, 3);
        assert_eq!(stage.start, Pos { x: 1, y: 1 });
        assert_eq!(stage.goal, Pos { x: 3, y: 1 });
        assert_eq!(stage.tile(Pos { x: 0, y: 0 }), Tile::Wall);
        assert_eq!(stage.tile(Pos { x: 2, y: 1 }), Tile::Floor);
    }

    #[test]
    fn parses_stage_pack_metadata() {
        static PACK: &[&str] = &[
            "@stage 07",
            "@name Glass Test",
            "@goal reach",
            "@limit 12",
            "@hint Watch the center.",
            "@gimmick mirror",
            "",
            "#####",
            "#P.G#",
            "#####",
        ];
        let stage = Stage::parse_pack(PACK);
        assert_eq!(stage.meta.id, "07");
        assert_eq!(stage.meta.name, "Glass Test");
        assert_eq!(stage.meta.goal_type, GoalType::Reach);
        assert_eq!(stage.meta.turn_limit, Some(12));
        assert_eq!(stage.meta.hint, "Watch the center.");
        assert_eq!(stage.meta.gimmick, "mirror");
        assert_eq!(stage.start, Pos { x: 1, y: 1 });
        assert_eq!(stage.goal, Pos { x: 3, y: 1 });
    }

    #[test]
    fn stage_pack_can_change_tile_meaning_per_stage() {
        static PACK: &[&str] = &[
            "@stage 08",
            "@name Rule Test",
            "@goal reach",
            "@limit 10",
            "@hint X is a wall here.",
            "@tile x wall",
            "",
            "#####",
            "#PxG#",
            "#####",
        ];
        let stage = Stage::parse_pack(PACK);
        assert_eq!(stage.tile(Pos { x: 2, y: 1 }), Tile::Wall);
    }

    #[test]
    fn parses_rule_set_metadata() {
        static PACK: &[&str] = &[
            "@stage 09",
            "@name Rules Test",
            "@goal reach",
            "@rules keydoor",
            "@limit 20",
            "@hint Pick the key.",
            "",
            "#####",
            "#PKG#",
            "#####",
        ];
        let stage = Stage::parse_pack(PACK);
        assert_eq!(stage.meta.rule_set, RuleSet::KeyDoor);
    }

    #[test]
    fn tile_rules_parse_key_door_ice_trap_and_goal() {
        static PACK: &[&str] = &[
            "@stage 10",
            "@name Tile Effect Test",
            "@goal reach",
            "@rules keydoor",
            "@tile k key",
            "@tile d door",
            "@tile i ice",
            "@tile t trap",
            "@tile z goal",
            "",
            "#########",
            "#Pkdizt.#",
            "#########",
        ];
        let stage = Stage::parse_pack(PACK);
        assert_eq!(stage.tile(Pos { x: 2, y: 1 }), Tile::Key);
        assert_eq!(stage.tile(Pos { x: 3, y: 1 }), Tile::Door);
        assert_eq!(stage.tile(Pos { x: 4, y: 1 }), Tile::Ice);
        assert_eq!(stage.tile(Pos { x: 6, y: 1 }), Tile::Trap);
        assert_eq!(stage.tile(Pos { x: 5, y: 1 }), Tile::Goal);
        assert_eq!(stage.goal, Pos { x: 5, y: 1 });
    }

    #[test]
    fn keydoor_gets_key() {
        static PACK: &[&str] = &[
            "@stage 11",
            "@name Key Test",
            "@goal reach",
            "@rules keydoor",
            "@tile K key",
            "",
            "#####",
            "#PKG#",
            "#####",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        assert!(game.has_key);
        assert_eq!(game.player, Pos { x: 2, y: 1 });
    }

    #[test]
    fn keydoor_blocks_door_without_key() {
        static PACK: &[&str] = &[
            "@stage 12",
            "@name Door Block Test",
            "@goal reach",
            "@rules keydoor",
            "@tile D door",
            "",
            "######",
            "#PDG.#",
            "######",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        assert_eq!(game.player, Pos { x: 1, y: 1 });
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn keydoor_allows_door_with_key() {
        static PACK: &[&str] = &[
            "@stage 13",
            "@name Door Open Test",
            "@goal reach",
            "@rules keydoor",
            "@tile K key",
            "@tile D door",
            "",
            "#######",
            "#PKDG.#",
            "#######",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        game.step(Direction::Right);
        assert!(game.has_key);
        assert_eq!(game.player, Pos { x: 3, y: 1 });
    }

    #[test]
    fn ice_rule_slides_until_before_wall() {
        static PACK: &[&str] = &[
            "@stage 14",
            "@name Ice Test",
            "@goal reach",
            "@rules ice",
            "@tile I ice",
            "",
            "########",
            "#PII..G#",
            "########",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        assert_eq!(game.player, Pos { x: 6, y: 1 });
        assert_eq!(game.scene, Scene::Clear);
    }

    #[test]
    fn trap_rule_enters_game_over() {
        static PACK: &[&str] = &[
            "@stage 15",
            "@name Trap Test",
            "@goal reach",
            "@rules trap",
            "@tile T trap",
            "",
            "#####",
            "#PTG#",
            "#####",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        assert!(game.trapped);
        assert_eq!(game.scene, Scene::GameOver);
    }

    #[test]
    fn classic_rule_keeps_existing_goal_behavior() {
        static PACK: &[&str] = &[
            "@stage 16",
            "@name Classic Test",
            "@goal reach",
            "@rules classic",
            "",
            "#####",
            "#PG.#",
            "#####",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        assert_eq!(game.scene, Scene::Clear);
    }

    #[test]
    fn undo_and_reset_restore_rule_flags() {
        static PACK: &[&str] = &[
            "@stage 17",
            "@name Flag Undo Test",
            "@goal reach",
            "@rules keydoor",
            "@tile K key",
            "@tile D door",
            "",
            "#######",
            "#PKDG.#",
            "#######",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        assert!(game.has_key);
        game.undo();
        assert!(!game.has_key);
        assert_eq!(game.player, Pos { x: 1, y: 1 });
        game.step(Direction::Right);
        assert!(game.has_key);
        game.reset_stage();
        assert!(!game.has_key);
        assert!(!game.trapped);
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn player_cannot_move_through_wall() {
        let stages = vec![Stage::parse(&["#####", "#P#G#", "#####"])];
        let mut game = GameState::new(stages);
        game.step(Direction::Right);
        assert_eq!(game.player, Pos { x: 1, y: 1 });
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn detects_goal_reached() {
        let stages = vec![Stage::parse(&["#####", "#PG.#", "#####"])];
        let mut game = GameState::new(stages);
        game.step(Direction::Right);
        assert_eq!(game.scene, Scene::Clear);
    }

    #[test]
    fn turn_limit_sets_game_over_after_limit_is_exceeded() {
        static PACK: &[&str] = &[
            "@stage 01",
            "@name Limit Test",
            "@goal reach",
            "@limit 1",
            "@hint Move once only.",
            "",
            "######",
            "#P..G#",
            "######",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        assert_eq!(game.scene, Scene::Game);
        assert_eq!(game.moves, 1);
        game.step(Direction::Right);
        assert_eq!(game.scene, Scene::GameOver);
        assert_eq!(game.moves, 2);
    }

    #[test]
    fn undo_restores_previous_position() {
        let stages = vec![Stage::parse(&["#####", "#P.G#", "#####"])];
        let mut game = GameState::new(stages);
        game.step(Direction::Right);
        assert_eq!(game.player, Pos { x: 2, y: 1 });
        game.undo();
        assert_eq!(game.player, Pos { x: 1, y: 1 });
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn undo_restores_turn_after_game_over() {
        static PACK: &[&str] = &[
            "@stage 01",
            "@name Undo Limit",
            "@goal reach",
            "@limit 1",
            "@hint Undo the bad move.",
            "",
            "######",
            "#P..G#",
            "######",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        game.step(Direction::Right);
        assert_eq!(game.scene, Scene::GameOver);
        game.undo();
        assert_eq!(game.scene, Scene::Game);
        assert_eq!(game.moves, 1);
        assert_eq!(game.player, Pos { x: 2, y: 1 });
    }

    #[test]
    fn reset_restores_current_stage_start() {
        let stages = vec![Stage::parse(&["#####", "#P.G#", "#####"])];
        let mut game = GameState::new(stages);
        game.step(Direction::Right);
        game.reset_stage();
        assert_eq!(game.player, Pos { x: 1, y: 1 });
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn reset_restores_turn_after_game_over() {
        static PACK: &[&str] = &[
            "@stage 01",
            "@name Reset Limit",
            "@goal reach",
            "@limit 1",
            "@hint Reset clears the clock.",
            "",
            "######",
            "#P..G#",
            "######",
        ];
        let mut game = GameState::new(vec![Stage::parse_pack(PACK)]);
        game.step(Direction::Right);
        game.step(Direction::Right);
        assert_eq!(game.scene, Scene::GameOver);
        game.reset_stage();
        assert_eq!(game.scene, Scene::Game);
        assert_eq!(game.moves, 0);
        assert_eq!(game.player, Pos { x: 1, y: 1 });
    }

    #[test]
    fn first_window_has_twenty_stage_pack_entries() {
        let stages = demo_stages();
        assert_eq!(stages.len(), 20);
        assert_eq!(stages[0].meta.id, "01");
        assert_eq!(stages[19].meta.name, "Final Window");
    }

    #[test]
    fn first_window_contains_all_rule_sets() {
        let stages = demo_stages();
        assert!(stages
            .iter()
            .any(|stage| stage.meta.rule_set == RuleSet::Classic));
        assert!(stages
            .iter()
            .any(|stage| stage.meta.rule_set == RuleSet::KeyDoor));
        assert!(stages
            .iter()
            .any(|stage| stage.meta.rule_set == RuleSet::Ice));
        assert!(stages
            .iter()
            .any(|stage| stage.meta.rule_set == RuleSet::Trap));
    }

    #[test]
    fn first_window_stages_have_required_metadata() {
        for stage in demo_stages() {
            assert!(!stage.meta.id.is_empty());
            assert!(!stage.meta.name.is_empty());
            assert!(stage.meta.turn_limit.is_some());
            assert!(!stage.meta.hint.is_empty());
            assert!(!stage.meta.gimmick.is_empty());
            assert_eq!(stage.meta.goal_type, GoalType::Reach);
        }
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn pixel_palette_converts_to_bgra_words() {
        let layer = PixelLayer::new(2, 1);
        assert_eq!(layer.bgra_word(0), 0x0018_120f);
        assert_eq!(layer.bgra_word(15), 0x00f8_f8f8);
        assert_eq!(layer.bgra_word(31), 0x00f8_f8f8);
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn win32_probe_frame_draws_checker_tiles_and_sprite() {
        let mut layer = PixelLayer::new(160, 144);
        render_win32_probe_frame(&mut layer, 0, ProbeInput::default());
        assert_eq!(layer.pixel(0, 0), 12);
        assert_eq!(layer.pixel(8, 16), 13);
        assert_eq!(layer.pixel(136, 128), 6);
        assert_eq!(layer.pixel(78, 66), 15);
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn win32_probe_arrow_input_moves_sprite() {
        let mut layer = PixelLayer::new(160, 144);
        render_win32_probe_frame(
            &mut layer,
            0,
            ProbeInput {
                right: true,
                down: true,
                ..ProbeInput::default()
            },
        );
        assert_eq!(layer.pixel(79, 67), 15);
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn probe_game_tick_moves_held_input_once_per_tick() {
        let mut game = ProbeGame::new();
        game.tick(ProbeInput {
            right: true,
            down: true,
            ..ProbeInput::default()
        });
        game.tick(ProbeInput {
            right: true,
            down: true,
            ..ProbeInput::default()
        });
        assert_eq!(game.tick_count, 2);
        assert_eq!(game.sprite_x, 74);
        assert_eq!(game.sprite_y, 62);
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn probe_game_clamps_sprite_to_screen_edges() {
        let mut game = ProbeGame {
            sprite_x: 159,
            sprite_y: 143,
            tick_count: 0,
        };
        game.tick(ProbeInput {
            right: true,
            down: true,
            ..ProbeInput::default()
        });
        assert_eq!(game.sprite_x, 144);
        assert_eq!(game.sprite_y, 120);
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn probe_game_title_reports_tick_and_timer() {
        let mut game = ProbeGame::new();
        game.tick(ProbeInput::default());
        assert_eq!(
            game.title(),
            "win32-pixel-probe v0.3 | 160x144 | 60Hz-ish tick | pos 72,60 | tick 1"
        );
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn probe_game_blocks_wall_tiles() {
        let mut game = ProbeGame::new();
        game.sprite_x = 8;
        game.sprite_y = 16;
        game.tick(ProbeInput {
            left: true,
            ..ProbeInput::default()
        });
        assert_eq!(game.sprite_x, 8);
        assert_eq!(game.sprite_y, 16);
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn probe_camera_tracks_sprite_inside_tilemap() {
        let mut game = ProbeGame::new();
        game.sprite_x = 132;
        game.sprite_y = 104;
        let camera = game.camera();
        assert_eq!(camera.x, 8);
        assert_eq!(camera.y, 0);
    }

    #[cfg(feature = "win32_pixel")]
    #[test]
    fn probe_scene_draws_hud_world_and_goal() {
        let mut layer = PixelLayer::new(160, 144);
        let game = ProbeGame::new();
        render_probe_scene(&mut layer, &game);
        assert_eq!(layer.pixel(0, 0), 12);
        assert_eq!(layer.pixel(8, 16), 13);
        assert_eq!(layer.pixel(136, 128), 6);
        assert_eq!(layer.pixel(game.sprite_x + 6, game.sprite_y + 6), 15);
    }

    #[cfg(feature = "pixel_tile")]
    #[test]
    fn pixel_layer_draws_8x8_tile() {
        let mut layer = PixelLayer::new(16, 16);
        let tile = [1u8; 64];
        layer.draw_tile(4, 4, &tile);
        assert_eq!(layer.pixel(4, 4), 1);
        assert_eq!(layer.pixel(11, 11), 1);
        assert_eq!(layer.pixel(12, 12), 0);
    }

    #[cfg(feature = "pixel_tile")]
    #[test]
    fn pixel_layer_draws_16x16_sprite_with_transparency() {
        let mut layer = PixelLayer::new(20, 20);
        let mut sprite = [0u8; 256];
        sprite[0] = 2;
        sprite[255] = 3;
        layer.draw_sprite(2, 2, &sprite, 0);
        assert_eq!(layer.pixel(2, 2), 2);
        assert_eq!(layer.pixel(17, 17), 3);
        assert_eq!(layer.pixel(3, 2), 0);
    }

    #[cfg(feature = "asset_tiles")]
    #[test]
    fn capacity_demo_has_64_tiles() {
        assert_eq!(CAPACITY_TILES.len(), 64);
        assert_eq!(CAPACITY_TILES[0].len(), 64);
    }

    #[cfg(feature = "asset_sprites")]
    #[test]
    fn capacity_demo_has_16_sprites() {
        assert_eq!(CAPACITY_SPRITES.len(), 16);
        assert_eq!(CAPACITY_SPRITES[0].len(), 256);
    }

    #[cfg(feature = "asset_maps")]
    #[test]
    fn capacity_demo_has_10_tilemaps() {
        assert_eq!(CAPACITY_MAPS.len(), 10);
        assert_eq!(CAPACITY_MAPS[0].len(), 20 * 15);
    }

    #[cfg(feature = "asset_sound")]
    #[test]
    fn capacity_demo_has_3_bgm_and_8_se() {
        assert_eq!(CAPACITY_BGM.len(), 3);
        assert_eq!(CAPACITY_SE.len(), 8);
    }

    #[test]
    fn capacity_demo_checksum_is_referenced() {
        let checksum = capacity_demo_checksum();
        assert!(checksum > 0);
    }
}
