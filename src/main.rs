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

    #[cfg(feature = "pixel_tile")]
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
        "@name First Window",
        "@goal reach",
        "@rules classic",
        "@limit 40",
        "@hint The shortest path is not always the best.",
        "@gimmick classic",
        "@tile x wall",
        "",
        "########",
        "#P....G#",
        "#..##..#",
        "#......#",
        "########",
    ],
    &[
        "@stage 02",
        "@name Narrow Key",
        "@goal reach",
        "@rules classic",
        "@limit 32",
        "@hint Count corners before you commit.",
        "@gimmick narrow",
        "@tile ~ floor",
        "",
        "#########",
        "#P..#...#",
        "#.#.#.#G#",
        "#.......#",
        "#########",
    ],
    &[
        "@stage 03",
        "@name Long Hall",
        "@goal reach",
        "@rules classic",
        "@limit 28",
        "@hint The wall is a ruler.",
        "@gimmick long-hall",
        "@tile x wall",
        "",
        "##########",
        "#P.......#",
        "#.######.#",
        "#......#G#",
        "##########",
    ],
    &[
        "@stage 04",
        "@name Back Step",
        "@goal reach",
        "@rules classic",
        "@limit 34",
        "@hint Undo is part of the toolkit.",
        "@gimmick backtrack",
        "@tile x wall",
        "",
        "##########",
        "#P..#....#",
        "#.#.#.##G#",
        "#...#....#",
        "##########",
    ],
    &[
        "@stage 05",
        "@name Last Pane",
        "@goal reach",
        "@rules classic",
        "@limit 38",
        "@hint A small detour opens the window.",
        "@gimmick final",
        "@tile x wall",
        "",
        "###########",
        "#P....#...#",
        "###.#.#.#G#",
        "#...#.....#",
        "###########",
    ],
    &[
        "@stage 06",
        "@name Brass Key",
        "@goal reach",
        "@rules keydoor",
        "@limit 36",
        "@hint Take K before pushing through D.",
        "@gimmick keydoor",
        "@tile K key",
        "@tile D door",
        "",
        "###########",
        "#P.K.D..G#",
        "#.#####..#",
        "#........#",
        "###########",
    ],
    &[
        "@stage 07",
        "@name Cold Pane",
        "@goal reach",
        "@rules ice",
        "@limit 18",
        "@hint Step on I and commit to the slide.",
        "@gimmick ice",
        "@tile I ice",
        "",
        "###########",
        "#PII....G#",
        "#.#####..#",
        "#........#",
        "###########",
    ],
    &[
        "@stage 08",
        "@name Red Floor",
        "@goal reach",
        "@rules trap",
        "@limit 30",
        "@hint T ends the run.",
        "@gimmick trap",
        "@tile T trap",
        "",
        "###########",
        "#P..T...G#",
        "#.#####..#",
        "#........#",
        "###########",
    ],
    &[
        "@stage 09",
        "@name Rule Gallery",
        "@goal reach",
        "@rules keydoor",
        "@limit 42",
        "@hint This one is a tiny rule showcase.",
        "@gimmick mixed",
        "@tile K key",
        "@tile D door",
        "@tile x wall",
        "",
        "############",
        "#P.K.xD..G#",
        "#..x......#",
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
            screen.text(3, 2, "MadoCore 144");
            screen.text(3, 4, "Enter: start  H: help  Q: quit");
        }
        Scene::Help => {
            screen.text(3, 2, "WASD/arrows: move");
            screen.text(3, 3, "U: undo  R: reset  Esc/Q: quit");
            screen.text(3, 5, "Reach G in all 9 packed stages.");
        }
        Scene::Game => screen.draw_game(game),
        Scene::Clear => {
            screen.text(3, 2, "All stages clear!");
            screen.text(3, 4, "Q: quit  R: restart");
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

fn main() -> io::Result<()> {
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
    fn demo_uses_nine_stage_pack_entries_with_rule_examples() {
        let stages = demo_stages();
        assert_eq!(stages.len(), 9);
        assert_eq!(stages[0].meta.id, "01");
        assert_eq!(stages[4].meta.name, "Last Pane");
        assert_eq!(stages[5].meta.rule_set, RuleSet::KeyDoor);
        assert_eq!(stages[6].meta.rule_set, RuleSet::Ice);
        assert_eq!(stages[7].meta.rule_set, RuleSet::Trap);
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
