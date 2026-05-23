use macroquad::prelude::*;
use macroquad::rand as mq_rand;
use macroquad::audio;

// ── Constants ────────────────────────────────────────────────────────────────

const GRAVITY: f32 = 980.0;
const JUMP_VELOCITY: f32 = -590.0;
const MOVE_SPEED: f32 = 280.0;
const PLAYER_WIDTH: f32 = 28.0;
const PLAYER_HEIGHT: f32 = 36.0;

const COYOTE_FRAMES: i32 = 6;
const JUMP_BUFFER_FRAMES: i32 = 5;

const SPIKE_HEIGHT: f32 = 24.0;
const SPIKE_TOOTH_WIDTH: f32 = 20.0;

const BABY_SPEED: f32 = 65.0;
const BABY_SCALE: f32 = 1.5;

// ── Dog colours ──────────────────────────────────────────────────────────────

const FUR: Color = Color::new(0.77, 0.52, 0.23, 1.0);
const FUR_DARK: Color = Color::new(0.63, 0.41, 0.16, 1.0);
const FUR_LIGHT: Color = Color::new(0.91, 0.73, 0.40, 1.0);
const EAR_COLOR: Color = Color::new(0.55, 0.34, 0.12, 1.0);
const NOSE_COLOR: Color = Color::new(0.15, 0.10, 0.06, 1.0);
const EYE_WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const EYE_PUPIL: Color = Color::new(0.10, 0.07, 0.04, 1.0);
const COLLAR: Color = Color::new(0.80, 0.20, 0.20, 1.0);
const TONGUE: Color = Color::new(0.95, 0.40, 0.40, 1.0);

const DOG_SCALE: f32 = 1.3;

// ── Baby colours ─────────────────────────────────────────────────────────────

const BABY_SKIN: Color = Color::new(0.96, 0.82, 0.69, 1.0);
const BABY_SKIN_SHADOW: Color = Color::new(0.82, 0.68, 0.56, 1.0);
const BABY_DIAPER: Color = Color::new(0.40, 0.70, 0.95, 1.0);
const BABY_DIAPER_DARK: Color = Color::new(0.25, 0.55, 0.80, 1.0);
const BABY_HAIR: Color = Color::new(0.20, 0.15, 0.10, 1.0);
const BABY_CHEEK: Color = Color::new(0.98, 0.60, 0.60, 1.0);

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct Player {
    pos: Vec2,
    vel: Vec2,
    size: Vec2,
    grounded: bool,
    coyote_counter: i32,
    jump_buffer_counter: i32,
    facing_right: bool,
    dead: bool,
    walk_time: f32,
    super_mode: bool,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            pos: vec2(x, y),
            vel: vec2(0.0, 0.0),
            size: vec2(PLAYER_WIDTH, PLAYER_HEIGHT),
            grounded: false,
            coyote_counter: 0,
            jump_buffer_counter: 0,
            facing_right: true,
            dead: false,
            walk_time: 0.0,
            super_mode: false,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Platform {
    pos: Vec2,
    size: Vec2,
}

impl Platform {
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Spike {
    pos: Vec2,
    width: f32,
    height: f32,
}

impl Spike {
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.width, self.height)
    }

    fn draw(&self, offset: Vec2) {
        let sx = self.pos.x - offset.x;
        let sy = self.pos.y - offset.y;

        if sx + self.width < 0.0 || sx > screen_width() {
            return;
        }

        let tooth_count = (self.width / SPIKE_TOOTH_WIDTH).ceil() as usize;
        let spacing = self.width / tooth_count as f32;

        for i in 0..tooth_count {
            let cx = sx + i as f32 * spacing + spacing / 2.0;
            let left = cx - spacing / 2.0;
            let right = cx + spacing / 2.0;
            let bottom = sy + self.height;
            let tip_y = sy + 2.0;

            draw_triangle(
                vec2(left, bottom),
                vec2(right, bottom),
                vec2(cx, tip_y),
                Color::from_hex(0x8b0000),
            );
            draw_triangle(
                vec2(left + 2.0, bottom - 1.0),
                vec2(right - 2.0, bottom - 1.0),
                vec2(cx, tip_y + 3.0),
                Color::from_hex(0xcc0000),
            );
        }
    }
}

/// A pile of poop the dog can leave. Babies react to it.
#[derive(Debug, Clone, PartialEq)]
struct Poop {
    pos: Vec2,
    vel_y: f32,
    lifetime: f32,
    eaten: bool,
}

impl Poop {
    fn new(x: f32, y: f32) -> Self {
        Self { pos: vec2(x, y), vel_y: 200.0, lifetime: 3.0, eaten: false }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x - 5.0, self.pos.y - 5.0, 10.0, 10.0)
    }
}

/// Types of food the dog can collect.
#[derive(Debug, Clone, PartialEq)]
enum FoodType {
    Bacon,
    Chicken,
    Burger,
    Pizza,
}

/// A piece of food scattered around the level.
#[derive(Debug, Clone, PartialEq)]
struct Food {
    pos: Vec2,
    kind: FoodType,
    collected: bool,
}

impl Food {
    fn new(x: f32, y: f32, kind: FoodType) -> Self {
        Self { pos: vec2(x, y), kind, collected: false }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x - 6.0, self.pos.y - 6.0, 12.0, 12.0)
    }
}

/// A single death particle.
#[derive(Debug, Clone, PartialEq)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    lifetime: f32,
    size: f32,
    color_override: Option<Color>,
}

/// A crawling baby enemy that patrols back and forth on a platform.
#[derive(Debug, Clone, PartialEq)]
struct Baby {
    pos: Vec2,
    vel: Vec2,
    size: Vec2,
    facing_right: bool,
    /// Leftmost x the baby will walk to.
    min_x: f32,
    /// Rightmost x the baby will walk to.
    max_x: f32,
    /// Floor y — the baby stays grounded on this.
    floor_y: f32,
    /// Time accumulator for crawl animation.
    crawl_time: f32,
    flee_timer: f32,
    is_cat: bool,
}

impl Baby {
    fn new(x: f32, floor_y: f32, min_x: f32, max_x: f32) -> Self {
        Self {
            pos: vec2(x, floor_y - 18.0),
            vel: vec2(BABY_SPEED, 0.0),
            size: vec2(22.0, 18.0),
            facing_right: true,
            min_x,
            max_x,
            floor_y,
            crawl_time: 0.0,
            flee_timer: 0.0,
            is_cat: false,
        }
    }

    fn new_cat(x: f32, floor_y: f32, min_x: f32, max_x: f32) -> Self {
        let mut b = Self::new(x, floor_y, min_x, max_x);
        b.is_cat = true;
        b.size = vec2(22.0, 20.0);
        b
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    fn update(&mut self, dt: f32) {
        self.crawl_time += dt;

        // Normal patrol (only runs when not fleeing)
        self.pos.x += self.vel.x * dt;

        if self.pos.x <= self.min_x {
            self.pos.x = self.min_x;
            self.vel.x = BABY_SPEED;
            self.facing_right = true;
        } else if self.pos.x + self.size.x >= self.max_x {
            self.pos.x = self.max_x - self.size.x;
            self.vel.x = -BABY_SPEED;
            self.facing_right = false;
        }

        // Stay on the floor
        self.pos.y = self.floor_y - self.size.y;
    }
}

// ── Toddler Boss ────────────────────────────────────────────────────────────

/// A gigantic toddler boss — 1.5x taller than Super Dog.
struct ToddlerBoss {
    pos: Vec2,
    size: Vec2,
    health: f32,
    max_health: f32,
    facing_right: bool,
    walk_time: f32,
    attack_timer: f32,
    dead: bool,
    death_timer: f32,
}

impl ToddlerBoss {
    fn new(x: f32, y: f32) -> Self {
        let scale = 225.0;
        Self {
            pos: vec2(x, y),
            size: vec2(scale * 0.6, scale),
            health: 30.0,
            max_health: 30.0,
            facing_right: false,
            walk_time: 0.0,
            attack_timer: 0.0,
            dead: false,
            death_timer: 0.0,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    fn update(&mut self, dt: f32, player_pos: Vec2) {
        if self.dead {
            self.death_timer -= dt;
            return;
        }
        self.walk_time += dt;
        self.attack_timer -= dt;
        let dx = player_pos.x - self.pos.x;
        self.facing_right = dx > 0.0;
        let speed = 60.0;
        self.pos.x += dx.signum() * speed * dt;
        self.pos.x = self.pos.x.clamp(0.0, 8000.0);
    }

    fn take_damage(&mut self, amount: f32) {
        if self.dead { return; }
        self.health -= amount;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.dead = true;
            self.death_timer = 2.0;
        }
    }
}

/// Holds all the level data returned by build_level / level_1 / level_2.
type LevelData = (Vec<Platform>, Vec<Spike>, Vec<Baby>, Vec<Lava>, Vec<Food>, Option<GoalBall>);

/// The goal tennis ball the dog must fetch to complete the level.
#[derive(Debug, Clone, PartialEq)]
struct GoalBall {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    collected: bool,
}

impl GoalBall {
    fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            pos: vec2(x, y),
            vel: vec2(80.0, 1.0),
            color,
            collected: false,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x - 8.0, self.pos.y - 8.0, 16.0, 16.0)
    }
}

/// A lava pit that kills the player on contact (same shape as spike, orange glow).
#[derive(Debug, Clone, PartialEq)]
struct Lava {
    pos: Vec2,
    width: f32,
    height: f32,
}

impl Lava {
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.width, self.height)
    }

    fn draw(&self, offset: Vec2) {
        let sx = self.pos.x - offset.x;
        let sy = self.pos.y - offset.y;

        if sx + self.width < 0.0 || sx > screen_width() {
            return;
        }

        let tooth_count = (self.width / SPIKE_TOOTH_WIDTH).ceil() as usize;
        let spacing = self.width / tooth_count as f32;

        // Pulsing brightness
        let pulse = ((get_time() as f32) * 3.0).sin() * 0.3 + 0.7;
        let bright = Color::new(1.0, 0.4 * pulse, 0.0, 1.0);
        let hot = Color::new(1.0, 0.67 * pulse, 0.0, 1.0);
        let dark = Color::new(0.8, 0.27 * pulse, 0.0, 1.0);

        for i in 0..tooth_count {
            let cx = sx + i as f32 * spacing + spacing / 2.0;
            let left = cx - spacing / 2.0;
            let right = cx + spacing / 2.0;
            let bottom = sy + self.height;
            let tip_y = sy + 4.0;

            // Dark core
            draw_triangle(
                vec2(left, bottom),
                vec2(right, bottom),
                vec2(cx, tip_y),
                dark,
            );
            // Bright glow
            draw_triangle(
                vec2(left + 3.0, bottom - 2.0),
                vec2(right - 3.0, bottom - 2.0),
                vec2(cx, tip_y + 3.0),
                bright,
            );
            // Hot centre
            draw_circle(cx, sy + self.height * 0.4, spacing * 0.2,
                        hot);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Title,
    Playing,
    Tutorial,
    LevelSelect,
    Paused,
}

struct Game {
    player: Player,
    platforms: Vec<Platform>,
    spikes: Vec<Spike>,
    babies: Vec<Baby>,
    poops: Vec<Poop>,
    particles: Vec<Particle>,
    death_timer: f32,
    death_message: String,
    played_death_sound: bool,
    play_ouch: bool,
    play_cheer: bool,
    dev_mode: bool,
    hearts: u32,
    invincible_timer: f32,
    state: GameState,
    goal_ball: Option<GoalBall>,
    lava_pits: Vec<Lava>,
    level: u32,
    foods: Vec<Food>,
    food_collected: u32,
    food_total: u32,
    level_complete: bool,
    complete_timer: f32,
    pee_mode: bool,
    done_pee: bool,
    tree_x: f32,
    boss_mode: bool,
    boss: Option<ToddlerBoss>,
}

impl Game {
    fn new() -> Self {
        let floor_y = screen_height() - 40.0;

        // Build level 1 by default
        let start_x = 80.0;
        let start_y = floor_y - PLAYER_HEIGHT;
        let (platforms, spikes, babies, lava_pits, _foods, goal_ball) = Self::build_level(1, floor_y);

        Self {
            player: Player::new(start_x, start_y),
            platforms,
            spikes,
            babies,
            lava_pits,
            poops: vec![],
            particles: vec![],
            death_timer: 0.0,
            death_message: String::new(),
            played_death_sound: false,
            play_ouch: false,
            play_cheer: false,
            dev_mode: false,
            hearts: 3,
            invincible_timer: 0.0,
            state: GameState::Title,
            goal_ball,
            foods: vec![],
            food_collected: 0,
            food_total: 0,
            level: 1,
            level_complete: false,
            complete_timer: 0.0,
            pee_mode: false,
            done_pee: false,
            tree_x: 0.0,
            boss_mode: false,
            boss: None,
        }
    }

    fn build_level(level: u32, floor_y: f32) -> LevelData {
        let data = match level {
            1 => include_str!("../levels/level1.txt"),
            3 => include_str!("../levels/level3.txt"),
            4 => include_str!("../levels/level4.txt"),
            _ => include_str!("../levels/level2.txt"),
        };
        parse_level(data, floor_y)
    }

    fn reset(&mut self) {
        let current_level = self.level;
        *self = Self::new();
        self.start_level(current_level.max(1));
    }

    fn next_level(&mut self) {
        let next = self.level + 1;
        let floor_y = screen_height() - 40.0;
        let (platforms, spikes, babies, lava_pits, foods, goal_ball) = Self::build_level(next, floor_y);
        self.player = Player::new(80.0, floor_y - PLAYER_HEIGHT);
        self.platforms = platforms;
        self.spikes = spikes;
        self.babies = babies;
        self.lava_pits = lava_pits;
        self.foods = foods;
        self.food_collected = 0;
        self.food_total = self.foods.len() as u32;
        let tree_x = goal_ball.as_ref().map(|b| b.pos.x).unwrap_or(0.0);
        self.goal_ball = goal_ball;
        self.tree_x = tree_x;
        self.level = next;
        self.poops.clear();
        self.particles.clear();
        self.level_complete = false;
        self.complete_timer = 0.0;
        self.player.dead = false;
        self.state = GameState::Playing;
    }

    fn start_level(&mut self, level: u32) {
        let floor_y = screen_height() - 40.0;
        let (platforms, spikes, babies, lava_pits, foods, goal_ball) = Self::build_level(level, floor_y);
        self.player = Player::new(80.0, floor_y - PLAYER_HEIGHT);
        self.platforms = platforms;
        self.spikes = spikes;
        self.babies = babies;
        self.lava_pits = lava_pits;
        self.foods = foods;
        self.food_collected = 0;
        self.food_total = self.foods.len() as u32;
        let tree_x = goal_ball.as_ref().map(|b| b.pos.x).unwrap_or(0.0);
        self.goal_ball = goal_ball;
        self.tree_x = tree_x;
        self.level = level;
        self.poops.clear();
        self.particles.clear();
        self.level_complete = false;
        self.complete_timer = 0.0;
        self.player.dead = false;
        self.state = GameState::Playing;
    }

    fn die(&mut self) {
        self.player.dead = true;
        self.death_timer = 0.3;
        self.played_death_sound = false;
        let px = self.player.pos.x + self.player.size.x / 2.0;
        let py = self.player.pos.y + self.player.size.y / 2.0;
        for _ in 0..30 {
            let angle = (mq_rand::rand() as f32 / u32::MAX as f32) * std::f32::consts::TAU;
            let speed = (mq_rand::rand() as f32 / u32::MAX as f32) * 250.0 + 80.0;
            let size = (mq_rand::rand() as f32 / u32::MAX as f32) * 5.0 + 3.0;
            self.particles.push(Particle {
                pos: vec2(px, py),
                vel: vec2(angle.cos() * speed, angle.sin() * speed),
                lifetime: (mq_rand::rand() as f32 / u32::MAX as f32) * 0.6 + 0.3,
                size,
                color_override: None,
            });
        }
    }

    fn update_player(&mut self, dt: f32) {
        // ── Input ────────────────────────────────────────────────────────
        let mut move_x = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            move_x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            move_x += 1.0;
        }

        if move_x > 0.0 {
            self.player.facing_right = true;
        } else if move_x < 0.0 {
            self.player.facing_right = false;
        }

        if self.player.grounded && move_x != 0.0 {
            self.player.walk_time += dt;
        } else if self.player.grounded {
            self.player.walk_time = 0.0;
        }

        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            self.player.jump_buffer_counter = JUMP_BUFFER_FRAMES;
        } else {
            self.player.jump_buffer_counter = self.player.jump_buffer_counter.saturating_sub(1);
        }

        // ── Apply movement & physics ─────────────────────────────────────
        self.player.vel.x = move_x * MOVE_SPEED;
        self.player.vel.y += GRAVITY * dt;
        self.player.vel.y = self.player.vel.y.clamp(-1200.0, 1200.0);

        if self.player.grounded {
            self.player.coyote_counter = COYOTE_FRAMES;
        } else {
            self.player.coyote_counter = self.player.coyote_counter.saturating_sub(1);
        }

        if self.player.jump_buffer_counter > 0 && self.player.coyote_counter > 0 {
            self.player.vel.y = JUMP_VELOCITY;
            self.player.grounded = false;
            self.player.coyote_counter = 0;
            self.player.jump_buffer_counter = 0;
        }

        let jump_released = is_key_released(KeyCode::Space)
            || is_key_released(KeyCode::W)
            || is_key_released(KeyCode::Up);
        if self.player.vel.y < 0.0 && jump_released {
            self.player.vel.y *= 0.5;
        }

        // ── Dev mode: hold Space/W/Up to fly upward ────────────────
        if self.dev_mode && (is_key_down(KeyCode::Space) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)) {
            self.player.vel.y = JUMP_VELOCITY;
            self.player.grounded = false;
        }

        self.player.pos += self.player.vel * dt;

        // ── Platform collisions ─────────────────────────────────────────
        self.player.grounded = false;

        for plat in &self.platforms {
            if let Some(collision) = self.player.rect().intersect(plat.rect()) {
                let overlap_x = collision.w;
                let overlap_y = collision.h;

                if overlap_x < overlap_y {
                    if self.player.vel.x > 0.0 {
                        self.player.pos.x -= overlap_x;
                    } else if self.player.vel.x < 0.0 {
                        self.player.pos.x += overlap_x;
                    }
                    self.player.vel.x = 0.0;
                } else {
                    if self.player.vel.y > 0.0 {
                        self.player.pos.y -= overlap_y;
                        self.player.vel.y = 0.0;
                        self.player.grounded = true;
                    } else if self.player.vel.y < 0.0 {
                        self.player.pos.y += overlap_y;
                        self.player.vel.y = 0.0;
                    }
                }
            }
        }

        // ── Spike collisions ────────────────────────────────────────────
        for spike in &self.spikes {
            if self.player.rect().intersect(spike.rect()).is_some() {
                if !self.dev_mode {
                    self.death_message = "Ouch! Spikes are not food".to_string();
                    self.die();
                }
                break;
            }
        }

        // ── Lava collisions ─────────────────────────────────────────────
        for lava in &self.lava_pits {
            if self.player.rect().intersect(lava.rect()).is_some() {
                if !self.dev_mode {
                    self.death_message = "Fire is great till it burns you".to_string();
                    self.die();
                }
                break;
            }
        }

        // ── Food collisions ────────────────────────────────────────────
        for food in &mut self.foods {
            if !food.collected && self.player.rect().intersect(food.rect()).is_some() {
                food.collected = true;
                self.food_collected += 1;
            }
        }

        // ── Baby collisions ─────────────────────────────────────────────
        if !self.player.dead && !self.pee_mode {
            for baby in &self.babies {
                if self.player.rect().intersect(baby.rect()).is_some() {
                    if !self.dev_mode && self.invincible_timer <= 0.0 {
                        let msg = if baby.is_cat {
                            "The cat scratched your nose"
                        } else {
                            "The baby pulled your tail"
                        };
                        if baby.is_cat {
                            // Cats deal 2 hearts of damage
                            if self.hearts > 2 {
                                self.hearts -= 2;
                                self.play_ouch = true;
                                self.invincible_timer = 2.0;
                                self.death_message = msg.to_string();
                            } else {
                                self.death_message = msg.to_string();
                                self.die();
                            }
                        } else {
                            if self.hearts > 1 {
                                self.hearts -= 1;
                                self.play_ouch = true;
                                self.invincible_timer = 2.0;
                                self.death_message = msg.to_string();
                            } else {
                                self.death_message = msg.to_string();
                                self.die();
                            }
                        }
                    }
                    break;
                }
            }
        }

        // ── Clamp to world bounds ───────────────────────────────────────
        if self.player.pos.x < 0.0 {
            self.player.pos.x = 0.0;
            self.player.vel.x = 0.0;
        }
    }

    fn camera_offset(&self) -> Vec2 {
        let target_x = self.player.pos.x + self.player.size.x / 2.0 - screen_width() / 2.0;
        let target_y = self.player.pos.y + self.player.size.y / 2.0 - screen_height() / 2.0;
        vec2(target_x.max(0.0), target_y.max(0.0))
    }

    fn draw(&self) {
        // ── Background ──────────────────────────────────────────────────
        clear_background(Color::from_hex(0x1a1a2e));

        // ── Boss mode background ────────────────────────────────
        if self.boss_mode {
            let cw = screen_width();
            let ch = screen_height();
            // Sky gradient
            for i in 0..20 {
                let t = i as f32 / 20.0;
                let r = (20.0 * (1.0 - t) + 60.0 * t) as u8;
                let g = (30.0 * (1.0 - t) + 120.0 * t) as u8;
                let b = (60.0 * (1.0 - t) + 180.0 * t) as u8;
                draw_rectangle(0.0, ch * t / 2.0, cw, ch / 40.0 + 1.0,
                               Color::from_rgba(r, g, b, 255));
            }
            // Sun
            draw_circle(cw * 0.85, ch * 0.12, 40.0, Color::from_hex(0xffdd44));
            draw_circle(cw * 0.85, ch * 0.12, 35.0, Color::from_hex(0xffee66));
            // Clouds
            for (cx, cy, r) in &[(0.15, 0.18, 35.0), (0.35, 0.12, 30.0), (0.65, 0.20, 25.0)] {
                draw_circle(cw * cx, ch * cy, *r, Color::from_rgba(200, 200, 220, 180));
                draw_circle(cw * cx - r * 0.5, ch * cy + r * 0.2, *r * 0.7, Color::from_rgba(200, 200, 220, 180));
                draw_circle(cw * cx + r * 0.5, ch * cy + r * 0.1, *r * 0.8, Color::from_rgba(200, 200, 220, 180));
            }
            // Green hills
            draw_circle(cw * 0.3, ch * 0.95, 350.0, Color::from_hex(0x3a7a3a));
            draw_circle(cw * 0.7, ch * 0.95, 400.0, Color::from_hex(0x4a8a3a));
            draw_circle(cw * 1.0, ch * 0.95, 300.0, Color::from_hex(0x2a6a2a));
            // Grass
            draw_rectangle(0.0, ch * 0.85, cw, ch * 0.15, Color::from_hex(0x4a9a3a));
            draw_rectangle(0.0, ch * 0.88, cw, ch * 0.12, Color::from_hex(0x3a8a2a));
        }

        match self.state {
            GameState::Title => {
                self.draw_title_screen();
                return;
            }
            GameState::Tutorial => {
                self.draw_tutorial_screen();
                return;
            }
            GameState::LevelSelect => {
                self.draw_level_select_screen();
                return;
            }
            GameState::Paused => {}
            GameState::Playing => {}
        }

        let cam = self.camera_offset();

        // ── Platforms ───────────────────────────────────────────────────
        if !self.pee_mode {
            for plat in &self.platforms {
                let sx = plat.pos.x - cam.x;
                let sy = plat.pos.y - cam.y;
                draw_rectangle(sx, sy, plat.size.x, plat.size.y, Color::from_hex(0x16213e));
                draw_rectangle(sx + 2.0, sy + 2.0, plat.size.x - 4.0, plat.size.y - 4.0, Color::from_hex(0x0f3460));
                draw_line(sx + 4.0, sy + 1.0, sx + plat.size.x - 4.0, sy + 1.0, 2.0, Color::from_hex(0x533483));
            }

            // ── Spikes ──────────────────────────────────────────────────────
            for spike in &self.spikes {
                spike.draw(cam);
            }

            // ── Lava pits ──────────────────────────────────────────────────
            for lava in &self.lava_pits {
                lava.draw(cam);
            }

            // ── Level 3 instruction sign (behind everything) ───────────────
            if self.level == 3 && self.state == GameState::Playing {
            let sign_x = 30.0 - cam.x;
            let sign_y = 700.0 - cam.y;
            let sign_w = 520.0;
            let sign_h = 50.0;
            draw_rectangle(sign_x, sign_y, sign_w, sign_h, Color::from_rgba(20, 15, 40, 220));
            draw_rectangle(sign_x + 2.0, sign_y + 2.0, sign_w - 4.0, sign_h - 4.0, Color::from_rgba(30, 25, 55, 220));
            let msg = "Pick up all the food to collect the Goal Ball!";
            let msg_size = measure_text(msg, None, 18, 1.0);
            draw_text(msg, sign_x + (sign_w - msg_size.width) / 2.0, sign_y + sign_h / 2.0 + 6.0,
                      18.0, Color::from_hex(0xf0c860));
        }
        }

        // ── Babies ──────────────────────────────────────────────────────
        if !self.pee_mode {
            for baby in &self.babies {
                let bx = baby.pos.x + baby.size.x / 2.0 - cam.x;
                let by = baby.pos.y + baby.size.y / 2.0 - cam.y;
                if baby.is_cat {
                    draw_cat_sprite(bx, by, baby);
                } else {
                    draw_baby_sprite(bx, by, baby);
                }
            }
        }

        // ── Goal ball ──────────────────────────────────────────────────
        if let Some(ball) = &self.goal_ball
            && !ball.collected
        {
            let sx = ball.pos.x - cam.x;
            let sy = ball.pos.y - cam.y;
            // Tree at fixed position near the ball on the same platform
            let tree_h = PLAYER_HEIGHT * DOG_SCALE * 3.0;
            let tree_cx = self.tree_x - cam.x;
            // Find the platform the ball is on
            let mut tree_ground = (screen_height() - 40.0) - cam.y;
            if let Some(b) = &self.goal_ball {
                for plat in &self.platforms {
                    if b.pos.x > plat.pos.x && b.pos.x < plat.pos.x + plat.size.x {
                        tree_ground = plat.pos.y - cam.y;
                        break;
                    }
                }
            }
            draw_tree(tree_cx - 30.0, tree_ground, tree_h);
            let all_food = self.food_total == 0 || self.food_collected >= self.food_total;
            let display_color = if all_food {
                ball.color
            } else {
                Color::from_hex(0x888888)
            };
            let display_highlight = if all_food {
                Color::new(
                    (ball.color.r * 1.1).min(1.0),
                    (ball.color.g * 1.1).min(1.0),
                    (ball.color.b * 1.1).min(1.0),
                    1.0,
                )
            } else {
                Color::from_hex(0xaaaaaa)
            };
            draw_tennis_ball(sx, sy, 12.0, display_color, display_highlight);
        }

        // ── Food ───────────────────────────────────────────────────────
        if !self.pee_mode {
        for food in &self.foods {
            if !food.collected {
                let sx = food.pos.x - cam.x;
                let sy = food.pos.y - cam.y;
                draw_food_sprite(sx, sy, &food.kind);
            }
        }
        }

        // ── Poops ──────────────────────────────────────────────────────
        if !self.pee_mode {
        for poop in &self.poops {
            if !poop.eaten {
                let sx = poop.pos.x - cam.x;
                let sy = poop.pos.y - cam.y;
                draw_poop_sprite(sx, sy);
            }
        }
        }

        // ── Player ──────────────────────────────────────────────────────
        if !self.player.dead {
            let flash = self.invincible_timer <= 0.0 || (self.invincible_timer * 10.0).floor() as i32 % 2 == 0;
            if flash {
                let psx = self.player.pos.x + self.player.size.x / 2.0 - cam.x;
                let psy = self.player.pos.y + self.player.size.y / 2.0 - cam.y;
                if self.player.super_mode {
                    draw_super_dog_sprite(psx, psy, &self.player, DOG_SCALE);
                } else {
                    draw_dog_sprite(psx, psy, &self.player, DOG_SCALE);
                }
            }
        }

        // ── Tree in pee mode (level 4) ────────────────────────────
        if self.pee_mode {
            let tree_h = PLAYER_HEIGHT * DOG_SCALE * 3.0;
            let tree_cx = self.tree_x - cam.x;
            let mut tree_ground = (screen_height() - 40.0) - cam.y;
            for plat in &self.platforms {
                if self.tree_x > plat.pos.x && self.tree_x < plat.pos.x + plat.size.x {
                    tree_ground = plat.pos.y - cam.y;
                    // Draw the platform under the tree
                    let sx = plat.pos.x - cam.x;
                    let sy = plat.pos.y - cam.y;
                    draw_rectangle(sx, sy, plat.size.x, plat.size.y, Color::from_hex(0x16213e));
                    draw_rectangle(sx + 2.0, sy + 2.0, plat.size.x - 4.0, plat.size.y - 4.0, Color::from_hex(0x0f3460));
                    draw_line(sx + 4.0, sy + 1.0, sx + plat.size.x - 4.0, sy + 1.0, 2.0, Color::from_hex(0x533483));
                    break;
                }
            }
            draw_tree(tree_cx - 30.0, tree_ground, tree_h);
        }

        // ── Pee stream is only visible via falling particle droplets ─
        //     (spawned in the main loop when holding E in pee_mode)

        // ── Toddler boss ───────────────────────────────────────────────
        if self.boss_mode {
            if let Some(boss) = &self.boss {
                let bx = boss.pos.x + boss.size.x / 2.0 - cam.x;
                let by = boss.pos.y + boss.size.y / 2.0 - cam.y;
                draw_toddler_boss_sprite(bx, by, boss);

                // Health bar
                if !boss.dead {
                    let hb_w = 200.0;
                    let hb_h = 18.0;
                    let hb_x = screen_width() / 2.0 - hb_w / 2.0;
                    let hb_y = 20.0;
                    draw_rectangle(hb_x, hb_y, hb_w, hb_h, Color::from_hex(0x333333));
                    let hp_ratio = boss.health / boss.max_health;
                    let hp_color = if hp_ratio > 0.5 { Color::from_hex(0xdd3333) } else { Color::from_hex(0xcc2222) };
                    draw_rectangle(hb_x + 2.0, hb_y + 2.0, (hb_w - 4.0) * hp_ratio, hb_h - 4.0, hp_color);
                    let boss_label = format!("ANGRY TODDLER");
                    let bls = measure_text(&boss_label, None, 14, 1.0);
                    draw_text(&boss_label, hb_x + (hb_w - bls.width) / 2.0, hb_y + hb_h - 4.0,
                              14.0, Color::from_hex(0xffffff));
                }
            }
        }

        // ── Particles ──────────────────────────────────────────────────
        for p in &self.particles {
            let sx = p.pos.x - cam.x;
            let sy = p.pos.y - cam.y;
            let alpha = (p.lifetime / 0.9).clamp(0.0, 1.0);
            let base = p.color_override.unwrap_or(Color::new(0.9, 0.1, 0.1, 1.0));
            let color = Color::new(base.r, base.g, base.b, alpha);
            draw_rectangle(sx - p.size / 2.0, sy - p.size / 2.0, p.size, p.size, color);
        }

        // ── HUD ─────────────────────────────────────────────────────────
        if !self.pee_mode {
        let food_info = if self.food_total > 0 {
            format!("Food: {}/{}  ", self.food_collected, self.food_total)
        } else {
            String::new()
        };
        let level_text = format!("Level {}", self.level);
        draw_text(&level_text, 16.0, 36.0, 26.0, Color::from_hex(0xaaaaaa));
        if self.food_total > 0 {
            draw_text(&food_info, 16.0, 68.0, 26.0, Color::from_hex(0xf0c860));
        }
        // Hearts (filled and empty) — 4th slot appears only after getting it
        let max_hearts = if self.done_pee { 4 } else { 3 };
        for i in 0..max_hearts {
            let hx = 130.0 + i as f32 * 36.0;
            let hy = 30.0;
            if i < self.hearts {
                draw_heart(hx, hy, 13.0, Color::from_hex(0xe94560));
            } else {
                draw_heart_outline(hx, hy, 13.0, Color::from_hex(0x663333));
            }
        }

        draw_text("Arrow keys / WASD to move, Space to jump  |  Q to poop  |  R to reset", 16.0, screen_height() - 16.0, 21.0, Color::from_hex(0x666666));
        }

        // ── Death overlay ───────────────────────────────────────────────
        if self.player.dead && self.death_timer <= 0.0 {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                           Color::from_rgba(0, 0, 0, 180));

            let title_size = measure_text(&self.death_message, None, 48, 1.0);
            draw_text(&self.death_message, screen_width() / 2.0 - title_size.width / 2.0, screen_height() / 2.0 - 20.0,
                      48.0, Color::from_hex(0xcc0000));

            let subtitle = "Press Space to respawn";
            let sub_size = measure_text(subtitle, None, 22, 1.0);
            draw_text(subtitle, screen_width() / 2.0 - sub_size.width / 2.0, screen_height() / 2.0 + 30.0,
                      22.0, Color::from_hex(0xaaaaaa));
        }

        // ── Level complete overlay ──────────────────────────────────────
        if self.level_complete {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                           Color::from_rgba(0, 0, 0, 180));

            let ball_color = self.goal_ball.as_ref().map(|b| b.color).unwrap_or(Color::from_hex(0x3cb371));
            let level_str = format!("LEVEL {} COMPLETE!", self.level);
            let title_size = measure_text(&level_str, None, 48, 1.0);
            draw_text(&level_str, screen_width() / 2.0 - title_size.width / 2.0, screen_height() / 2.0 - 20.0,
                      48.0, ball_color);

            let subtitle = if self.level == 4 {
                "The dog fetched the ball!"
            } else {
                "The dog fetched the ball!  Press Space for next level"
            };
            let sub_size = measure_text(subtitle, None, 22, 1.0);
            draw_text(subtitle, screen_width() / 2.0 - sub_size.width / 2.0, screen_height() / 2.0 + 30.0,
                      22.0, Color::from_hex(0xaaaaaa));
        }

        // ── Find tree ground for sign/chicken placement ──────────
        let tree_h = PLAYER_HEIGHT * DOG_SCALE * 3.0;
        let mut tree_ground = (screen_height() - 40.0) - cam.y;
        for plat in &self.platforms {
            if self.tree_x > plat.pos.x && self.tree_x < plat.pos.x + plat.size.x {
                tree_ground = plat.pos.y - cam.y;
                break;
            }
        }

        // ── Pee mode sign (above tree, like level 3) ──────────────
        if self.pee_mode && !self.done_pee {
            let sign_x = (self.tree_x - 30.0) - cam.x - 260.0;
            let sign_y = tree_ground - tree_h - 70.0;
            let sign_w = 520.0;
            let sign_h = 50.0;
            draw_rectangle(sign_x, sign_y, sign_w, sign_h, Color::from_rgba(20, 15, 40, 220));
            draw_rectangle(sign_x + 2.0, sign_y + 2.0, sign_w - 4.0, sign_h - 4.0, Color::from_rgba(30, 25, 55, 220));
            let msg = "You can pee on anything using E!  Let's start with this tree";
            let msg_size = measure_text(msg, None, 18, 1.0);
            draw_text(msg, sign_x + (sign_w - msg_size.width) / 2.0, sign_y + sign_h / 2.0 + 6.0,
                      18.0, Color::from_hex(0xf0c860));
        }

        // ── Done pee: reward sign + roast chicken ──────────────────
        if self.done_pee {
            // Sign above tree
            let sign_x = (self.tree_x - 30.0) - cam.x - 330.0;
            let sign_y = tree_ground - tree_h - 70.0;
            let sign_w = 660.0;
            let sign_h = 50.0;
            draw_rectangle(sign_x, sign_y, sign_w, sign_h, Color::from_rgba(20, 15, 40, 220));
            draw_rectangle(sign_x + 2.0, sign_y + 2.0, sign_w - 4.0, sign_h - 4.0, Color::from_rgba(30, 25, 55, 220));
            let msg = "This roast chicken will turn you into a way more powerful dog";
            let msg_size = measure_text(msg, None, 18, 1.0);
            draw_text(msg, sign_x + (sign_w - msg_size.width) / 2.0, sign_y + sign_h / 2.0 + 6.0,
                      18.0, Color::from_hex(0xf0c860));

            // Roast chicken on the ground near the tree (until eaten)
            if !self.player.super_mode {
                let chicken_x = (self.tree_x - 80.0) - cam.x;
                let chicken_y = tree_ground - 20.0;
                draw_rotisserie_chicken(chicken_x, chicken_y);
            }
        }

        // ── Pause overlay ───────────────────────────────────────────────
        if self.state == GameState::Paused {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                           Color::from_rgba(0, 0, 0, 160));

            let cw = screen_width();
            let ch = screen_height();

            let title = "PAUSED";
            let ts = measure_text(title, None, 56, 1.0);
            draw_text(title, (cw - ts.width) / 2.0, ch * 0.22, 56.0, Color::from_hex(0xe94560));

            let bw = 220.0;
            let bh = 50.0;
            let bx = (cw - bw) / 2.0;
            let start_y = ch * 0.38;
            let gap = 60.0;
            let items = ["Resume", "Main Menu", "Quit", "Dev Mode"];

            for (i, name) in items.iter().enumerate() {
                let iy = start_y + i as f32 * gap;
                let hovered = is_mouse_over(bx, iy, bw, bh);
                let bg = if hovered { Color::from_hex(0x533483) } else { Color::from_hex(0x16213e) };
                draw_rectangle(bx, iy, bw, bh, bg);
                draw_rectangle(bx + 2.0, iy + 2.0, bw - 4.0, bh - 4.0, Color::from_hex(0x0f3460));
                // Show ON/OFF state for Dev Mode
                let label = if *name == "Dev Mode" {
                    if self.dev_mode { "Dev Mode: ON" } else { "Dev Mode: OFF" }
                } else {
                    name
                };
                let color = if *name == "Dev Mode" && self.dev_mode {
                    Color::from_hex(0x55dd55)
                } else {
                    Color::from_hex(0xcccccc)
                };
                let ls = measure_text(label, None, 28, 1.0);
                draw_text(label, bx + (bw - ls.width) / 2.0, iy + bh / 2.0 + 10.0,
                          28.0, color);
            }
        }
    }

    // ── Title screen ────────────────────────────────────────────────────
    fn draw_title_screen(&self) {
        let cw = screen_width();
        let ch = screen_height();
        let time = get_time() as f32;
        let bob_offset = (time * 1.5).sin() * 6.0;

        // ── Golden particles (behind dog and ball) ─────────────────────
        for p in &self.particles {
            let alpha = (p.lifetime / 0.9).clamp(0.0, 1.0);
            let base = p.color_override.unwrap_or(Color::new(0.9, 0.1, 0.1, 1.0));
            let color = Color::new(base.r, base.g, base.b, alpha);
            draw_rectangle(p.pos.x - p.size / 2.0, p.pos.y - p.size / 2.0, p.size, p.size, color);
        }

        // Title text
        let title_font_size = 72.0;
        let title = "DOG ADVENTURE";
        let ts = measure_text(title, None, title_font_size as _, 1.0);
        draw_text(title, (cw - ts.width) / 2.0, ch * 0.18, title_font_size, Color::from_hex(0xe94560));

        // Subtitle
        let sub = "A dog and his tennis ball";
        let sub_font = 22.0;
        let ss = measure_text(sub, None, sub_font as _, 1.0);
        draw_text(sub, (cw - ss.width) / 2.0, ch * 0.28, sub_font, Color::from_hex(0xaaaaaa));

        // Dog sprite (left of centre) — gentle bounce
        let dog_cx = cw * 0.35;
        let dog_cy = ch * 0.48 + bob_offset;
        let dummy = Player::new(0.0, 0.0);
        draw_dog_sprite(dog_cx, dog_cy, &dummy, DOG_SCALE * 3.2);

        // Tennis ball (right of centre) — gentle bounce (opposite phase)
        let ball_cx = cw * 0.65;
        let ball_cy = ch * 0.48 - bob_offset;
        draw_golden_tennis_ball(ball_cx, ball_cy, 8.0 * DOG_SCALE * 3.2 * 1.1);

        // Buttons
        let bw = 220.0;
        let bh = 50.0;
        let bx = (cw - bw) / 2.0;
        let by = ch * 0.62;
        let gap = 64.0;
        let button_names = ["Play", "Levels", "Tutorial", "Quit"];

        for (i, name) in button_names.iter().enumerate() {
            let iy = by + i as f32 * gap;
            let hovered = is_mouse_over(bx, iy, bw, bh);
            let bg = if hovered { Color::from_hex(0x533483) } else { Color::from_hex(0x16213e) };
            draw_rectangle(bx, iy, bw, bh, bg);
            draw_rectangle(bx + 2.0, iy + 2.0, bw - 4.0, bh - 4.0, Color::from_hex(0x0f3460));
            let label_size = measure_text(name, None, 28, 1.0);
            draw_text(name, bx + (bw - label_size.width) / 2.0, iy + bh / 2.0 + 10.0,
                      28.0, Color::from_hex(0xcccccc));
        }
    }

    fn draw_tutorial_screen(&self) {
        let cw = screen_width();
        let ch = screen_height();

        draw_text("TUTORIAL", (cw - measure_text("TUTORIAL", None, 48, 1.0).width) / 2.0,
                  ch * 0.15, 48.0, Color::from_hex(0xe94560));

        let lines = [
            "Arrow keys / WASD — Move left and right",
            "Space / W / Up — Jump",
            "Q — Drop a poop to scare babies",
            "R — Reset the level",
            "",
            "Avoid the spike pits and crawling babies.",
            "If a baby pulls your tail, you lose!",
        ];
        let line_h = 32.0;
        let start_y = ch * 0.28;
        for (i, line) in lines.iter().enumerate() {
            let ls = measure_text(line, None, 20, 1.0);
            draw_text(line, (cw - ls.width) / 2.0, start_y + i as f32 * line_h,
                      20.0, Color::from_hex(0xaaaaaa));
        }

        // Back button
        let bw = 180.0;
        let bh = 44.0;
        let bx = (cw - bw) / 2.0;
        let by = ch * 0.78;
        draw_rectangle(bx, by, bw, bh, Color::from_hex(0x16213e));
        draw_rectangle(bx + 2.0, by + 2.0, bw - 4.0, bh - 4.0, Color::from_hex(0x0f3460));
        let label = "Back (Escape)";
        let ls = measure_text(label, None, 24, 1.0);
        draw_text(label, bx + (bw - ls.width) / 2.0, by + bh / 2.0 + 8.0, 24.0, Color::from_hex(0xcccccc));
    }

    // ── Level select screen ─────────────────────────────────────────────
    fn draw_level_select_screen(&self) {
        let cw = screen_width();
        let ch = screen_height();

        draw_text("SELECT LEVEL", (cw - measure_text("SELECT LEVEL", None, 48, 1.0).width) / 2.0,
                  ch * 0.15, 48.0, Color::from_hex(0xe94560));

        let bw = 160.0;
        let bh = 120.0;
        let cols = 3;
        let spacing = 40.0;
        let total_w = cols as f32 * bw + (cols - 1) as f32 * spacing;
        let start_x = (cw - total_w) / 2.0;
        let start_y = ch * 0.35;

        for i in 0..4 {
            let col = i % cols;
            let row = i / cols;
            let bx = start_x + col as f32 * (bw + spacing);
            let by = start_y + row as f32 * (bh + spacing);
            let hovered = is_mouse_over(bx, by, bw, bh);
            let bg = if hovered { Color::from_hex(0x533483) } else { Color::from_hex(0x16213e) };
            draw_rectangle(bx, by, bw, bh, bg);
            draw_rectangle(bx + 2.0, by + 2.0, bw - 4.0, bh - 4.0, Color::from_hex(0x0f3460));
            let label = format!("Level {}", i + 1);
            let ls = measure_text(&label, None, 28, 1.0);
            draw_text(&label, bx + (bw - ls.width) / 2.0, by + bh / 2.0 + 10.0, 28.0, Color::from_hex(0xcccccc));
        }
        let back_bw = 180.0;
        let back_bh = 44.0;
        let back_bx = (cw - back_bw) / 2.0;
        let back_by = ch * 0.82;
        draw_rectangle(back_bx, back_by, back_bw, back_bh, Color::from_hex(0x16213e));
        draw_rectangle(back_bx + 2.0, back_by + 2.0, back_bw - 4.0, back_bh - 4.0, Color::from_hex(0x0f3460));
        let label = "Back (Escape)";
        let ls = measure_text(label, None, 24, 1.0);
        draw_text(label, back_bx + (back_bw - ls.width) / 2.0, back_by + back_bh / 2.0 + 8.0, 24.0, Color::from_hex(0xcccccc));
    }
}

// ── Tennis ball drawing ──────────────────────────────────────────────────────

fn draw_tennis_ball(cx: f32, cy: f32, radius: f32, main_color: Color, highlight_color: Color) {
    draw_circle(cx, cy, radius, main_color);
    draw_circle(cx, cy, radius - 1.5, highlight_color);

    // Seam lines
    let r = radius * 0.82;
    draw_circle_lines(cx, cy, r, 2.5, Color::from_hex(0xf0f0f0));
    draw_circle_lines(cx + 2.0, cy, r * 0.7, 2.0, Color::from_hex(0xf0f0f0));

    // Highlight
    draw_circle(cx - radius * 0.25, cy - radius * 0.25, radius * 0.15,
                Color::from_rgba(255, 255, 255, 60));
}

/// Draw a golden tennis ball (original title screen variant).
fn draw_golden_tennis_ball(cx: f32, cy: f32, radius: f32) {
    draw_tennis_ball(cx, cy, radius, Color::from_hex(0xd4c73c), Color::from_hex(0xe8da4a));
}

/// Draw a simple tree (trunk + green canopy).
fn draw_tree(x: f32, y: f32, h: f32) {
    let trunk_w = h * 0.12;
    let trunk_h = h * 0.5;
    let canopy_r = h * 0.25;
    // Trunk
    draw_rectangle(x - trunk_w / 2.0, y - trunk_h, trunk_w, trunk_h, Color::from_hex(0x8b5e3c));
    // Canopy
    draw_circle(x, y - trunk_h - canopy_r * 0.2, canopy_r, Color::from_hex(0x3a7d3a));
    draw_circle(x, y - trunk_h - canopy_r * 0.4, canopy_r * 0.8, Color::from_hex(0x4a9d4a));
}

/// Draw a heart shape using overlapping circles.
fn draw_heart(cx: f32, cy: f32, size: f32, color: Color) {
    let r = size * 0.45;
    let off = size * 0.28;
    // Top bumps lifted high for deep cleft between them
    draw_circle(cx - off, cy - off * 0.8, r, color);
    draw_circle(cx + off, cy - off * 0.8, r, color);
    // Lower fill between bumps and point
    draw_circle(cx, cy + off * 0.2, r * 0.8, color);
    // Point
    draw_triangle(
        vec2(cx - off * 1.3, cy + off * 0.1),
        vec2(cx + off * 1.3, cy + off * 0.1),
        vec2(cx, cy + size * 0.75),
        color,
    );
}

/// Draw a heart outline (empty heart) at the given position.
fn draw_heart_outline(cx: f32, cy: f32, size: f32, color: Color) {
    let r = size * 0.45;
    let off = size * 0.28;
    draw_circle_lines(cx - off, cy - off * 0.8, r, 1.5, color);
    draw_circle_lines(cx + off, cy - off * 0.8, r, 1.5, color);
    draw_circle_lines(cx, cy + off * 0.2, r * 0.8, 1.5, color);
    let tip = vec2(cx, cy + size * 0.75);
    let left = vec2(cx - off * 1.3, cy + off * 0.1);
    let right = vec2(cx + off * 1.3, cy + off * 0.1);
    draw_line(left.x, left.y, tip.x, tip.y, 1.5, color);
    draw_line(right.x, right.y, tip.x, tip.y, 1.5, color);
}

/// Draw a piece of food at the given position.
fn draw_food_sprite(x: f32, y: f32, kind: &FoodType) {
    let s = 1.3;
    match kind {
        FoodType::Bacon => {
            // Wavy pink/red bacon strip with white fat streaks
            draw_rectangle(x - 8.0 * s, y - 3.0 * s, 16.0 * s, 7.0 * s, Color::from_hex(0xb84530));
            draw_rectangle(x - 7.0 * s, y - 4.0 * s, 14.0 * s, 8.0 * s, Color::from_hex(0xd06050));
            draw_rectangle(x - 6.0 * s, y - 2.0 * s, 12.0 * s, 4.0 * s, Color::from_hex(0xe88070));
            // Fat streaks
            draw_rectangle(x - 5.0 * s, y - 3.0 * s, 3.0 * s, 6.0 * s, Color::from_hex(0xf0b0a0));
            draw_rectangle(x + 1.0 * s, y - 3.0 * s, 3.0 * s, 6.0 * s, Color::from_hex(0xf0b0a0));
        }
        FoodType::Chicken => {
            // Drumstick — elongated meat with bone
            let cx = x + 1.0 * s;
            // Meat (rounded oblong, wider at top, tapering down)
            draw_circle(cx, y - 1.0 * s, 5.0 * s, Color::from_hex(0xc89028));
            draw_circle(cx - 1.0 * s, y + 2.0 * s, 4.0 * s, Color::from_hex(0xc89028));
            draw_rectangle(cx - 4.0 * s, y - 4.0 * s, 8.0 * s, 8.0 * s, Color::from_hex(0xd4a030));
            draw_circle(cx, y - 1.0 * s, 4.0 * s, Color::from_hex(0xe0b040));
            draw_circle(cx - 1.0 * s, y + 2.0 * s, 3.0 * s, Color::from_hex(0xe0b040));
            // Breading texture
            draw_circle(cx + 2.0 * s, y - 2.0 * s, 1.5 * s, Color::from_hex(0xe8c860));
            draw_circle(cx - 3.0 * s, y + 1.0 * s, 1.0 * s, Color::from_hex(0xe8c860));
            // Bone sticking out the bottom
            draw_rectangle(cx - 1.5 * s, y + 5.0 * s, 3.0 * s, 5.0 * s, Color::from_hex(0xe8e8d8));
            draw_circle(cx, y + 10.0 * s, 2.0 * s, Color::from_hex(0xf0f0e0));
            draw_circle(cx, y + 9.0 * s, 1.5 * s, Color::from_hex(0xf8f8f0));
        }
        FoodType::Burger => {
            // Top bun
            draw_circle(x, y - 2.0 * s, 8.0 * s, Color::from_hex(0xc07830));
            draw_circle(x, y - 2.0 * s, 7.0 * s, Color::from_hex(0xd48840));
            draw_circle(x, y - 3.0 * s, 4.0 * s, Color::from_hex(0xe8a050));
            // Lettuce
            draw_rectangle(x - 7.0 * s, y - 1.0 * s, 14.0 * s, 3.0 * s, Color::from_hex(0x6a994e));
            // Cheese
            draw_rectangle(x - 6.0 * s, y + 1.0 * s, 12.0 * s, 2.0 * s, Color::from_hex(0xe8c040));
            // Patty
            draw_circle(x, y + 4.0 * s, 6.0 * s, Color::from_hex(0x6b3a20));
            draw_circle(x, y + 4.0 * s, 5.0 * s, Color::from_hex(0x7a4828));
            // Bottom bun
            draw_rectangle(x - 5.0 * s, y + 6.0 * s, 10.0 * s, 4.0 * s, Color::from_hex(0xb06828));
            draw_rectangle(x - 4.0 * s, y + 7.0 * s, 8.0 * s, 2.0 * s, Color::from_hex(0xc07830));
        }
        FoodType::Pizza => {
            // Crust triangle
            draw_triangle(
                vec2(x - 8.0 * s, y + 6.0 * s),
                vec2(x + 8.0 * s, y + 6.0 * s),
                vec2(x, y - 7.0 * s),
                Color::from_hex(0xd49030),
            );
            // Cheese layer
            draw_triangle(
                vec2(x - 6.0 * s, y + 5.0 * s),
                vec2(x + 6.0 * s, y + 5.0 * s),
                vec2(x, y - 5.0 * s),
                Color::from_hex(0xe8b840),
            );
            // Crust edge
            draw_line(x - 8.0 * s, y + 6.0 * s, x + 8.0 * s, y + 6.0 * s, 3.0 * s, Color::from_hex(0xc07820));
            // Pepperoni
            draw_circle(x, y - 2.0 * s, 3.0 * s, Color::from_hex(0xc04030));
            draw_circle(x - 3.0 * s, y + 2.0 * s, 2.5 * s, Color::from_hex(0xc04030));
            draw_circle(x + 3.0 * s, y + 2.0 * s, 2.5 * s, Color::from_hex(0xc04030));
            draw_circle(x - 4.0 * s, y - 1.0 * s, 2.0 * s, Color::from_hex(0xc04030));
            draw_circle(x + 4.0 * s, y - 1.0 * s, 2.0 * s, Color::from_hex(0xc04030));
            // Grease spots
            draw_circle(x - 1.0 * s, y - 3.0 * s, 1.0 * s, Color::from_rgba(255, 200, 50, 80));
            draw_circle(x + 2.0 * s, y + 3.0 * s, 1.0 * s, Color::from_rgba(255, 200, 50, 80));
        }
    }
}

/// Check if the mouse is currently over the given rectangle.
fn is_mouse_over(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

// ── Dog sprite drawing ───────────────────────────────────────────────────────

fn draw_dog_sprite(cx: f32, cy: f32, p: &Player, s: f32) {
    let flip = if p.facing_right { 1.0 } else { -1.0 };
    let t = p.walk_time;

    let (leg_phase, body_bob_x, body_bob_y, tail_angle, ear_tilt, tongue_out);

    if p.grounded {
        if p.vel.x != 0.0 {
            leg_phase = t * 8.0;
            body_bob_x = (t * 10.0).sin() * 0.6 * s;
            body_bob_y = (t * 10.0).sin() * 1.2 * s;
            tail_angle = (t * 12.0).sin() * 0.6;
            ear_tilt = (t * 8.0).sin() * 0.06;
            tongue_out = true;
        } else {
            leg_phase = 0.0;
            body_bob_x = 0.0;
            body_bob_y = (t * 2.5).sin() * 0.4 * s;
            tail_angle = (t * 3.0).sin() * 0.3;
            ear_tilt = 0.0;
            tongue_out = (t * 2.0).sin() > 0.3;
        }
    } else {
        leg_phase = std::f32::consts::PI;
        body_bob_x = 0.0;
        body_bob_y = 0.0;
        tail_angle = -0.8;
        ear_tilt = -0.15;
        tongue_out = true;
    }

    let bx = cx + body_bob_x * flip;
    let by = cy + 1.0 * s + body_bob_y;
    let ox = |dx: f32| bx + dx * flip * s;

    let hbw = 10.0 * s;
    let hbh = 6.0 * s;
    draw_rectangle(bx - hbw, by - hbh, hbw * 2.0, hbh * 2.0, FUR);

    draw_circle(ox(-9.0), by - 1.0 * s, 7.0 * s, FUR);
    draw_circle(ox(9.0), by - 2.0 * s, 8.0 * s, FUR);
    draw_rectangle(bx - 8.0 * s, by - 1.0 * s, 16.0 * s, 6.0 * s, FUR_LIGHT);
    draw_circle(bx - 6.0 * s, by + 2.0 * s, 3.0 * s, FUR_LIGHT);
    draw_circle(bx + 6.0 * s, by + 2.0 * s, 3.0 * s, FUR_LIGHT);

    let tail_anchor_x = -12.0 * s;
    let tail_anchor_y = by - 3.0 * s;
    let tail_dir = tail_angle * 0.7 - 0.6;
    let tip_local_x = tail_anchor_x + tail_dir.cos() * 10.0 * s;
    let tip_y = tail_anchor_y + tail_dir.sin() * 10.0 * s - 6.0 * s;
    draw_line(ox(tail_anchor_x / s), tail_anchor_y, ox(tip_local_x / s), tip_y, 5.0 * s, FUR);
    draw_circle(ox(tip_local_x / s), tip_y, 4.0 * s, FUR_LIGHT);
    draw_circle(ox((tip_local_x - 1.0 * s) / s), tip_y - 1.0 * s, 2.5 * s, FUR_LIGHT);

    let bl_swing = (leg_phase + 0.5).sin() * 4.0 * s;
    let bl_off = if p.grounded && p.vel.x != 0.0 { bl_swing } else { 0.0 };
    draw_back_leg(ox(-6.0 + bl_off / s * 0.3), by + 7.0 * s, p.grounded, s);

    let fl_swing = (leg_phase).sin() * 4.0 * s;
    let fl_off = if p.grounded && p.vel.x != 0.0 { fl_swing } else { 0.0 };
    draw_front_leg(ox(6.5 + fl_off / s * 0.3), by + 7.0 * s, p.grounded, s);

    let head_dx = 16.0 * s;
    let head_dy = -4.0 * s;
    let hx = ox(head_dx / s);
    let hy = by + head_dy;
    let hx_off = |dx: f32| hx + dx * flip * s;

    let ear_angle = std::f32::consts::FRAC_PI_4 + ear_tilt;
    let ear_dir = ear_angle.cos();
    let ear_drop = (ear_angle.sin().abs() + 0.2) * 7.0 * s;
    draw_triangle(
        vec2(hx_off(3.0), hy - 6.0 * s),
        vec2(hx_off(3.0 + ear_dir * 8.0), hy - 6.0 * s + ear_drop),
        vec2(hx_off(8.0), hy - 2.0 * s),
        EAR_COLOR,
    );
    draw_triangle(
        vec2(hx_off(-2.0), hy - 6.0 * s),
        vec2(hx_off(-2.0 - ear_dir * 8.0), hy - 6.0 * s + ear_drop),
        vec2(hx_off(2.0), hy - 2.0 * s),
        EAR_COLOR,
    );

    draw_circle(hx, hy, 8.0 * s, FUR);
    draw_circle(hx_off(5.0), hy + 2.0 * s, 5.0 * s, FUR);
    draw_circle(hx_off(6.0), hy + 2.0 * s, 3.5 * s, FUR_LIGHT);

    draw_circle(hx_off(2.0), hy - 1.5 * s, 3.0 * s, EYE_WHITE);
    draw_circle(hx_off(5.5), hy - 1.5 * s, 3.0 * s, EYE_WHITE);
    let p_off = if p.facing_right { s } else { -s };
    draw_circle(hx_off(2.0 + p_off / s), hy - 1.5 * s, 1.5 * s, EYE_PUPIL);
    draw_circle(hx_off(5.5 + p_off / s), hy - 1.5 * s, 1.5 * s, EYE_PUPIL);
    draw_circle(hx_off(2.5 + p_off * 0.5 / s), hy - 2.5 * s, 0.7 * s, WHITE);
    draw_circle(hx_off(6.0 + p_off * 0.5 / s), hy - 2.5 * s, 0.7 * s, WHITE);

    draw_circle(hx_off(8.0), hy + 3.0 * s, 2.0 * s, NOSE_COLOR);
    draw_circle(hx_off(7.8), hy + 2.5 * s, 0.5 * s, Color::from_hex(0x3a2510));

    draw_line(hx_off(3.0), hy + 5.0 * s, hx_off(7.5), hy + 5.0 * s, 1.5 * s, FUR_DARK);
    if tongue_out {
        draw_rectangle(hx_off(4.5), hy + 5.0 * s, 2.5 * s, 4.0 * s, TONGUE);
        draw_circle(hx_off(5.75), hy + 5.0 * s, 1.5 * s, TONGUE);
    }

    draw_line(ox(8.0), by - 7.0 * s, ox(15.0), hy + 5.0 * s, 3.0 * s, COLLAR);
    draw_circle(ox(12.0), by - 3.0 * s, 2.5 * s, Color::from_hex(0xffd700));
    draw_circle(ox(11.5), by - 3.5 * s, 0.8 * s, Color::from_hex(0xfff8dc));
}

fn draw_front_leg(x: f32, y: f32, grounded: bool, s: f32) {
    if grounded {
        draw_rectangle(x - 2.0 * s, y, 4.0 * s, 6.0 * s, FUR);
        draw_rectangle(x - 3.0 * s, y + 5.0 * s, 6.0 * s, 2.5 * s, FUR_DARK);
    } else {
        draw_rectangle(x - 1.5 * s, y - 1.0 * s, 3.0 * s, 4.0 * s, FUR);
        draw_circle(x, y + 4.0 * s, 2.5 * s, FUR_DARK);
    }
}

fn draw_back_leg(x: f32, y: f32, grounded: bool, s: f32) {
    if grounded {
        draw_rectangle(x - 2.0 * s, y, 4.0 * s, 6.0 * s, FUR_DARK);
        draw_rectangle(x - 3.0 * s, y + 5.0 * s, 6.0 * s, 2.5 * s, FUR_DARK);
    } else {
        draw_rectangle(x - 1.5 * s, y - 1.0 * s, 3.0 * s, 4.0 * s, FUR_DARK);
        draw_circle(x, y + 4.0 * s, 2.5 * s, FUR_DARK);
    }
}

// ── Super Dog sprite drawing ────────────────────────────────────────────────

/// Draw the Super Dog — big, muscular, standing upright like a superhero.
fn draw_super_dog_sprite(cx: f32, cy: f32, p: &Player, s: f32) {
    let flip = if p.facing_right { 1.0 } else { -1.0 };
    let t = p.walk_time;

    // Tree-sized: scale to match tree height (PLAYER_HEIGHT * DOG_SCALE * 3.0)
    let hero_scale = s * 2.8;
    let bx = cx;
    // Offset so feet touch bottom of player rect (cy + PLAYER_HEIGHT/2 = cy + 18)
    let by = cy + 18.0 - 10.0 * hero_scale;
    let ox = |dx: f32| bx + dx * flip * hero_scale;
    let bob = (t * 3.0).sin() * 1.0 * hero_scale;

    let chest_top = by - 16.0 * hero_scale + bob;
    let chest_bot = by - 2.0 * hero_scale + bob;
    let chest_w = 18.0 * hero_scale;
    let waist_w = 12.0 * hero_scale;
    let ab_h = (chest_bot - chest_top) / 6.0;

    let super_fur = Color::from_hex(0xd4a050);
    let super_fur_light = Color::from_hex(0xe8c070);
    let super_fur_shadow = Color::from_hex(0xb07830);

    // ── Cape (behind body) ──────────────────────────────────────────
    let cape_color = Color::from_hex(0xcc2233);
    let cape_flutter = (t * 6.0).sin() * 4.0 * hero_scale;
    draw_triangle(
        vec2(ox(-4.0), chest_top),
        vec2(ox(-12.0 + cape_flutter / hero_scale * 0.3 * flip), chest_bot + 8.0 * hero_scale),
        vec2(ox(4.0), chest_bot + 6.0 * hero_scale),
        cape_color,
    );
    draw_triangle(
        vec2(ox(-6.0), chest_top),
        vec2(ox(-16.0 + cape_flutter / hero_scale * 0.5 * flip), chest_bot + 4.0 * hero_scale),
        vec2(ox(-8.0), chest_bot + 2.0 * hero_scale),
        Color::from_hex(0xaa1122),
    );

    // ── Torso: broad chest tapering to waist ────────────────────────
    draw_triangle(
        vec2(ox(-chest_w / 2.0 / hero_scale), chest_top),
        vec2(ox(chest_w / 2.0 / hero_scale), chest_top),
        vec2(ox(-waist_w / 2.0 / hero_scale), chest_bot),
        super_fur,
    );
    draw_triangle(
        vec2(ox(chest_w / 2.0 / hero_scale), chest_top),
        vec2(ox(waist_w / 2.0 / hero_scale), chest_bot),
        vec2(ox(-waist_w / 2.0 / hero_scale), chest_bot),
        super_fur,
    );

    // Chest highlight
    draw_triangle(
        vec2(ox(-4.0), chest_top + 2.0 * hero_scale),
        vec2(ox(4.0), chest_top + 2.0 * hero_scale),
        vec2(ox(-2.0), chest_bot - 4.0 * hero_scale),
        super_fur_light,
    );

    // ── Six-pack abs ────────────────────────────────────────────────
    let ab_center = (chest_top + chest_bot) / 2.0 + 2.0 * hero_scale;
    for row in 0..3 {
        for col in 0..2 {
            let ab_x = ox(-1.5 + col as f32 * 3.0);
            let ab_y = ab_center + row as f32 * ab_h;
            draw_rectangle(ab_x - 2.0 * hero_scale, ab_y - 1.5 * hero_scale,
                           4.0 * hero_scale, 3.0 * hero_scale, super_fur_shadow);
            draw_rectangle(ab_x - 1.5 * hero_scale, ab_y - 1.0 * hero_scale,
                           3.0 * hero_scale, 2.0 * hero_scale, super_fur_light);
        }
    }

    // ── Shoulders ───────────────────────────────────────────────────
    draw_circle(ox(-chest_w / 2.0 / hero_scale), chest_top + 2.0 * hero_scale,
                6.0 * hero_scale, super_fur);
    draw_circle(ox(chest_w / 2.0 / hero_scale), chest_top + 2.0 * hero_scale,
                6.0 * hero_scale, super_fur);
    draw_circle(ox(-chest_w / 2.0 / hero_scale), chest_top + 2.0 * hero_scale,
                5.0 * hero_scale, super_fur_light);
    draw_circle(ox(chest_w / 2.0 / hero_scale), chest_top + 2.0 * hero_scale,
                5.0 * hero_scale, super_fur_light);

    // ── Chest emblem ────────────────────────────────────────────────
    let emblem_y = chest_top + 3.0 * hero_scale;
    draw_circle(ox(0.0), emblem_y, 5.0 * hero_scale, Color::from_hex(0xdd4444));
    draw_circle(ox(0.0), emblem_y, 4.0 * hero_scale, Color::from_hex(0xffcc00));
    draw_text("S", ox(0.0) - 3.0 * hero_scale, emblem_y + 3.5 * hero_scale,
              5.0 * hero_scale, Color::from_hex(0xdd4444));

    // ── Arms (muscular front legs, raised hero pose) ────────────────
    let arm_swing = (t * 4.0).sin() * 3.0 * hero_scale;
    // Right arm (raised up)
    draw_rectangle(ox(9.0) - 2.0 * hero_scale, chest_top - 4.0 * hero_scale + arm_swing,
                   4.0 * hero_scale, 10.0 * hero_scale, super_fur);
    draw_circle(ox(9.0), chest_top - 4.0 * hero_scale + arm_swing, 3.5 * hero_scale, super_fur_light);
    draw_rectangle(ox(3.0) - 1.5 * hero_scale, chest_bot - 6.0 * hero_scale,
                   3.0 * hero_scale, 4.0 * hero_scale, super_fur);
    // Left arm (raised up)
    draw_rectangle(ox(-9.0) - 2.0 * hero_scale, chest_top - 4.0 * hero_scale - arm_swing,
                   4.0 * hero_scale, 10.0 * hero_scale, super_fur);
    draw_circle(ox(-9.0), chest_top - 4.0 * hero_scale - arm_swing, 3.5 * hero_scale, super_fur_light);
    draw_rectangle(ox(-3.0) - 1.5 * hero_scale, chest_bot - 6.0 * hero_scale,
                   3.0 * hero_scale, 4.0 * hero_scale, super_fur);

    // ── Legs (thick hind legs, standing) ─────────────────────────────
    let leg_swing = (t * 4.0).sin() * 2.0 * hero_scale;
    // Right leg
    let rl_x = ox(5.0);
    draw_rectangle(rl_x - 3.0 * hero_scale, chest_bot + leg_swing,
                   6.0 * hero_scale, 12.0 * hero_scale, super_fur);
    draw_circle(rl_x, chest_bot + 12.0 * hero_scale, 4.5 * hero_scale, super_fur_shadow);
    draw_circle(rl_x, chest_bot + 12.0 * hero_scale, 3.5 * hero_scale, super_fur);
    // Left leg
    let ll_x = ox(-5.0);
    draw_rectangle(ll_x - 3.0 * hero_scale, chest_bot - leg_swing,
                   6.0 * hero_scale, 12.0 * hero_scale, super_fur);
    draw_circle(ll_x, chest_bot - leg_swing + 12.0 * hero_scale, 4.5 * hero_scale, super_fur_shadow);
    draw_circle(ll_x, chest_bot - leg_swing + 12.0 * hero_scale, 3.5 * hero_scale, super_fur);

    // ── Tail (wagging behind) ────────────────────────────────────────
    let tail_wag = (t * 6.0).sin() * 6.0 * hero_scale;
    draw_line(ox(-10.0), chest_bot - 2.0 * hero_scale,
              ox(-10.0 - 2.0), chest_bot - 6.0 * hero_scale - tail_wag,
              4.0 * hero_scale, super_fur);
    draw_circle(ox(-10.0 - 2.0), chest_bot - 6.0 * hero_scale - tail_wag,
                3.0 * hero_scale, super_fur_light);

    // ── Head ────────────────────────────────────────────────────────
    let head_cx = ox(0.0);
    let head_cy = chest_top - 6.0 * hero_scale + bob;
    let hx_off = |dx: f32| head_cx + dx * flip * hero_scale;
    let head_r = 9.0 * hero_scale;

    // Head base (drawn before ears so ears sit on top)
    draw_circle(head_cx, head_cy, head_r, super_fur);
    draw_circle(hx_off(5.0), head_cy + 2.0 * hero_scale, 5.0 * hero_scale, super_fur);
    draw_circle(hx_off(6.0), head_cy + 2.0 * hero_scale, 3.5 * hero_scale, super_fur_light);
    // Snout
    draw_circle(hx_off(7.0), head_cy + 3.0 * hero_scale, 4.0 * hero_scale, super_fur);
    draw_circle(hx_off(7.0), head_cy + 4.0 * hero_scale, 3.0 * hero_scale, super_fur_light);

    // Ears (pointy, heroic — drawn on top of head)
    let ear_wag = (t * 5.0).sin() * 3.0 * hero_scale;
    // Right ear
    draw_triangle(
        vec2(hx_off(5.0), head_cy - head_r + 3.0 * hero_scale),
        vec2(hx_off(12.0 + ear_wag / hero_scale), head_cy - head_r - 8.0 * hero_scale),
        vec2(hx_off(10.0), head_cy - 2.0 * hero_scale),
        EAR_COLOR,
    );
    // Left ear
    draw_triangle(
        vec2(hx_off(-5.0), head_cy - head_r + 3.0 * hero_scale),
        vec2(hx_off(-12.0 - ear_wag / hero_scale), head_cy - head_r - 8.0 * hero_scale),
        vec2(hx_off(-4.0), head_cy - 2.0 * hero_scale),
        EAR_COLOR,
    );

    // ── Hero mask ───────────────────────────────────────────────────
    let mask_color = Color::from_hex(0x1a1a2e);
    // Centered mask covering both eyes
    draw_rectangle(hx_off(-5.0), head_cy - 4.0 * hero_scale,
                   13.0 * hero_scale, 4.5 * hero_scale, mask_color);
    // Mask edges curve slightly
    draw_circle(hx_off(-5.0), head_cy - 2.0 * hero_scale, 3.0 * hero_scale, mask_color);
    draw_circle(hx_off(8.0), head_cy - 2.0 * hero_scale, 3.0 * hero_scale, mask_color);
    // Eye holes (showing fur through mask)
    draw_circle(hx_off(-0.5), head_cy - 1.5 * hero_scale, 3.0 * hero_scale, super_fur);
    draw_circle(hx_off(5.5), head_cy - 1.5 * hero_scale, 3.0 * hero_scale, super_fur);

    // Eyes (inside the mask eye holes)
    draw_circle(hx_off(-0.5), head_cy - 1.5 * hero_scale, 2.5 * hero_scale, EYE_WHITE);
    draw_circle(hx_off(5.5), head_cy - 1.5 * hero_scale, 2.5 * hero_scale, EYE_WHITE);
    let p_off = if p.facing_right { hero_scale } else { -hero_scale };
    draw_circle(hx_off(-0.5 + p_off / hero_scale), head_cy - 1.5 * hero_scale,
                1.8 * hero_scale, EYE_PUPIL);
    draw_circle(hx_off(5.5 + p_off / hero_scale), head_cy - 1.5 * hero_scale,
                1.8 * hero_scale, EYE_PUPIL);
    draw_circle(hx_off(0.0 + p_off * 0.5 / hero_scale), head_cy - 3.0 * hero_scale,
                0.8 * hero_scale, WHITE);
    draw_circle(hx_off(6.0 + p_off * 0.5 / hero_scale), head_cy - 3.0 * hero_scale,
                0.8 * hero_scale, WHITE);

    // Nose
    draw_circle(hx_off(8.5), head_cy + 3.0 * hero_scale, 2.5 * hero_scale, NOSE_COLOR);
    draw_circle(hx_off(8.3), head_cy + 2.5 * hero_scale, 0.7 * hero_scale, Color::from_hex(0x3a2510));

    // Mouth (confident grin)
    draw_line(hx_off(3.0), head_cy + 6.0 * hero_scale,
              hx_off(8.0), head_cy + 6.0 * hero_scale, 2.0 * hero_scale, FUR_DARK);
    // Confident smirk curve
    draw_line(hx_off(5.0), head_cy + 6.0 * hero_scale,
              hx_off(6.5), head_cy + 5.0 * hero_scale, 1.5 * hero_scale, FUR_DARK);

    // ── Heroic collar ────────────────────────────────────────────────
    let collar_color = Color::from_hex(0xdd4444);
    draw_rectangle(hx_off(-3.0), head_cy + 8.0 * hero_scale,
                   10.0 * hero_scale, 3.0 * hero_scale, collar_color);
    // Gold badge
    draw_circle(hx_off(2.0), head_cy + 9.5 * hero_scale,
                2.5 * hero_scale, Color::from_hex(0xffcc00));
}

// ── Baby sprite drawing ──────────────────────────────────────────────────────

/// Draw a crawling baby at the given centre position.
fn draw_baby_sprite(cx: f32, cy: f32, b: &Baby) {
    let flip = if b.facing_right { 1.0 } else { -1.0 };
    let t = b.crawl_time;
    let s = BABY_SCALE;

    let crawl = (t * 7.0).sin();

    let ox = |dx: f32| cx + dx * flip * s;

    // ── Body (torso — small, slightly tilted) ──────────────────────────
    let body_cy = cy + 1.0;
    draw_rectangle(cx - 5.0 * s, body_cy - 4.0 * s, 10.0 * s, 8.0 * s, BABY_SKIN);

    // ── Diaper (round bottom) ──────────────────────────────────────────
    draw_circle(ox(0.0), body_cy + 4.0 * s, 6.0 * s, BABY_DIAPER);
    draw_circle(ox(0.0), body_cy + 4.0 * s, 5.0 * s, BABY_DIAPER_DARK);

    // ── Back arm (left side, behind body) ──────────────────────────────
    let back_arm_x = ox(-5.0) + crawl * 2.0 * s;
    draw_rectangle(back_arm_x - 1.5 * s, body_cy + 2.0 * s, 3.0 * s, 6.0 * s, BABY_SKIN_SHADOW);
    draw_circle(back_arm_x, body_cy + 8.0 * s, 2.5 * s, BABY_SKIN_SHADOW);

    // ── Back leg ────────────────────────────────────────────────────────
    let back_leg_x = ox(-3.0) - crawl * 1.5 * s;
    draw_rectangle(back_leg_x - 1.5 * s, body_cy + 4.0 * s, 3.0 * s, 5.0 * s, BABY_SKIN_SHADOW);
    draw_circle(back_leg_x, body_cy + 9.0 * s, 2.0 * s, BABY_SKIN_SHADOW);

    // ── Front arm (visible side) ────────────────────────────────────────
    let front_arm_x = ox(5.0) - crawl * 2.0 * s;
    draw_rectangle(front_arm_x - 1.5 * s, body_cy + 2.0 * s, 3.0 * s, 6.0 * s, BABY_SKIN);
    draw_circle(front_arm_x, body_cy + 8.0 * s, 2.5 * s, BABY_SKIN);

    // ── Front leg ───────────────────────────────────────────────────────
    let front_leg_x = ox(3.0) + crawl * 1.5 * s;
    draw_rectangle(front_leg_x - 1.5 * s, body_cy + 4.0 * s, 3.0 * s, 5.0 * s, BABY_SKIN);
    draw_circle(front_leg_x, body_cy + 9.0 * s, 2.0 * s, BABY_SKIN);

    // ── Head (big round head, slightly forward) ─────────────────────────
    let head_x = ox(6.0);
    let head_y = cy - 4.0;

    // Hair (a few tufts on top)
    draw_circle(head_x + 1.0, head_y - 6.0, 3.5, BABY_HAIR);
    draw_circle(head_x - 2.0, head_y - 6.0, 3.0, BABY_HAIR);
    draw_circle(head_x + 4.0, head_y - 5.0, 2.5, BABY_HAIR);

    // Head
    draw_circle(head_x, head_y, 7.0, BABY_SKIN);

    // Cheeks (rosy)
    draw_circle(head_x - 3.0, head_y + 2.0, 2.5, BABY_CHEEK);
    draw_circle(head_x + 4.5, head_y + 2.0, 2.5, BABY_CHEEK);

    // Eyes (big baby eyes)
    draw_circle(head_x - 1.5, head_y - 0.5, 3.0, EYE_WHITE);
    draw_circle(head_x + 3.5, head_y - 0.5, 3.0, EYE_WHITE);
    let p_off = if b.facing_right { 1.0 } else { -1.0 };
    draw_circle(head_x - 1.5 + p_off * 0.5, head_y - 0.5, 1.5, EYE_PUPIL);
    draw_circle(head_x + 3.5 + p_off * 0.5, head_y - 0.5, 1.5, EYE_PUPIL);
    draw_circle(head_x - 1.0 + p_off * 0.3, head_y - 1.5, 0.6, WHITE);
    draw_circle(head_x + 4.0 + p_off * 0.3, head_y - 1.5, 0.6, WHITE);

    // Mouth (open — waaah!)
    draw_circle(head_x + 1.0, head_y + 4.0, 2.0, Color::from_hex(0x4a2010));
    draw_circle(head_x + 1.0, head_y + 4.0, 1.2, Color::from_hex(0xcc3333));
}
/// Draw a cat at the given centre position (replaces baby sprite for cats).
fn draw_cat_sprite(cx: f32, cy: f32, b: &Baby) {
    let flip = if b.facing_right { 1.0 } else { -1.0 };
    let t = b.crawl_time;
    let s = BABY_SCALE;
    let crawl = (t * 7.0).sin();
    let ox = |dx: f32| cx + dx * flip * s;

    // ── Body (orange tabby) ──────────────────────────────────────────
    let body_cy = cy + 1.0;
    draw_rectangle(cx - 5.0 * s, body_cy - 4.0 * s, 10.0 * s, 8.0 * s, Color::from_hex(0xe09040));
    draw_rectangle(cx - 5.0 * s, body_cy - 4.0 * s, 10.0 * s, 3.0 * s, Color::from_hex(0xf0a060));

    // ── Tail (curled up) ─────────────────────────────────────────────
    let tail_base_x = ox(-6.0);
    let tail_tip_x = ox(-9.0) + crawl * 1.5;
    draw_line(tail_base_x, body_cy, tail_tip_x, body_cy - 8.0, 3.0, Color::from_hex(0xe09040));
    draw_circle(tail_tip_x, body_cy - 8.0, 2.5, Color::from_hex(0xf0a060));

    // ── Back legs ────────────────────────────────────────────────────
    let bl_x = ox(-3.0) - crawl * 2.0 * s;
    draw_rectangle(bl_x - 1.5 * s, body_cy + 3.0 * s, 3.0 * s, 5.0 * s, Color::from_hex(0xe09040));
    draw_circle(bl_x, body_cy + 8.0 * s, 2.0 * s, Color::from_hex(0xd08030));

    // ── Front legs ───────────────────────────────────────────────────
    let fl_x = ox(3.0) + crawl * 2.0 * s;
    draw_rectangle(fl_x - 1.5 * s, body_cy + 3.0 * s, 3.0 * s, 5.0 * s, Color::from_hex(0xe09040));
    draw_circle(fl_x, body_cy + 8.0 * s, 2.0 * s, Color::from_hex(0xd08030));

    // ── Head ─────────────────────────────────────────────────────────
    let head_x = ox(6.0);
    let head_y = cy - 4.0;

    // Pointy ears
    // Back ear
    draw_triangle(
        vec2(ox(3.0), head_y - 3.0),
        vec2(ox(5.0), head_y - 3.0),
        vec2(ox(4.0), head_y - 9.0),
        Color::from_hex(0xd08030),
    );
    // Front ear
    draw_triangle(
        vec2(ox(7.0), head_y - 3.0),
        vec2(ox(9.0), head_y - 3.0),
        vec2(ox(8.0), head_y - 9.0),
        Color::from_hex(0xd08030),
    );

    // Head
    draw_circle(head_x, head_y, 6.0, Color::from_hex(0xe09040));
    draw_circle(head_x, head_y - 1.0, 4.5, Color::from_hex(0xf0a060));

    // Eyes (slanted cat eyes)
    draw_circle(head_x - 1.5, head_y - 1.5, 2.5, Color::from_hex(0x90e060));
    draw_circle(head_x + 3.5, head_y - 1.5, 2.5, Color::from_hex(0x90e060));
    let p_off = if b.facing_right { 1.0 } else { -1.0 };
    draw_rectangle(head_x - 2.0 + p_off * 0.5, head_y - 2.0, 3.0, 1.5, EYE_PUPIL);
    draw_rectangle(head_x + 2.5 + p_off * 0.5, head_y - 2.0, 3.0, 1.5, EYE_PUPIL);

    // Whiskers
    draw_line(ox(2.0), head_y + 1.0, ox(-2.0), head_y + 2.0, 1.0, Color::from_hex(0xcccccc));
    draw_line(ox(2.0), head_y + 2.0, ox(-2.0), head_y + 4.0, 1.0, Color::from_hex(0xcccccc));
    draw_line(ox(10.0), head_y + 1.0, ox(14.0), head_y + 2.0, 1.0, Color::from_hex(0xcccccc));
    draw_line(ox(10.0), head_y + 2.0, ox(14.0), head_y + 4.0, 1.0, Color::from_hex(0xcccccc));

    // Nose (pink triangle)
    draw_circle(head_x + 1.0, head_y + 2.5, 1.5, Color::from_hex(0xff8888));

    // Mouth
    draw_line(head_x - 0.5, head_y + 3.5, head_x + 2.5, head_y + 3.5, 1.0, Color::from_hex(0x885555));
}

// ── Poop sprite drawing ─────────────────────────────────────────────────────

/// Draw a little pile of poop at the given position.
fn draw_poop_sprite(x: f32, y: f32) {
    // Brown mound made of overlapping circles
    draw_circle(x, y, 5.0, Color::from_hex(0x5c3a1e));
    draw_circle(x - 3.0, y - 1.0, 4.0, Color::from_hex(0x5c3a1e));
    draw_circle(x + 3.0, y - 1.0, 4.0, Color::from_hex(0x5c3a1e));
    // Highlight on top
    draw_circle(x, y - 2.0, 3.0, Color::from_hex(0x7a4e28));
    draw_circle(x - 1.0, y - 3.0, 2.0, Color::from_hex(0x8c5c30));
}


// ── Rotisserie chicken drawing ──────────────────────────────────────────────

/// Draw a whole golden rotisserie chicken on a spit at the given position.
fn draw_rotisserie_chicken(x: f32, y: f32) {
    // ── Spit (vertical rod) ─────────────────────────────────────────
    let spit_color = Color::from_hex(0x888888);
    draw_line(x, y - 18.0, x, y + 18.0, 2.0, spit_color);
    // Spit handle at top
    draw_rectangle(x - 4.0, y - 20.0, 8.0, 4.0, Color::from_hex(0x666666));

    // ── Body (oval, golden brown) ───────────────────────────────────
    draw_ellipse(x, y, 14.0, 10.0, 0.0, Color::from_hex(0xb07620));
    draw_ellipse(x, y, 12.0, 8.0, 0.0, Color::from_hex(0xc88828));
    draw_ellipse(x, y, 10.0, 6.5, 0.0, Color::from_hex(0xd49830));
    draw_ellipse(x, y, 8.0, 5.0, 0.0, Color::from_hex(0xe0a838));

    // ── Drumsticks protruding from sides ────────────────────────────
    // Left drumstick
    draw_ellipse(x - 12.0, y + 2.0, 5.0, 3.0, 0.3, Color::from_hex(0xa06818));
    draw_circle(x - 14.0, y + 4.0, 2.5, Color::from_hex(0x905010));
    // Right drumstick
    draw_ellipse(x + 12.0, y + 2.0, 5.0, 3.0, -0.3, Color::from_hex(0xa06818));
    draw_circle(x + 14.0, y + 4.0, 2.5, Color::from_hex(0x905010));

    // ── Wings tucked against body ───────────────────────────────────
    draw_ellipse(x - 9.0, y - 4.0, 4.0, 3.0, 0.5, Color::from_hex(0xb88028));
    draw_ellipse(x + 9.0, y - 4.0, 4.0, 3.0, -0.5, Color::from_hex(0xb88028));

    // ── Twine wrapping ──────────────────────────────────────────────
    draw_line(x - 9.0, y - 3.0, x + 9.0, y - 3.0, 1.0, Color::from_hex(0x886644));
    draw_line(x - 9.0, y + 2.0, x + 9.0, y + 2.0, 1.0, Color::from_hex(0x886644));
    draw_line(x - 8.0, y + 5.0, x + 8.0, y + 5.0, 1.0, Color::from_hex(0x886644));

    // ── Golden crispy skin highlights ───────────────────────────────
    draw_circle(x - 2.0, y - 1.0, 3.0, Color::from_hex(0xe8b848));
    draw_circle(x + 3.0, y + 1.0, 2.5, Color::from_hex(0xe8b848));
    draw_circle(x, y + 3.0, 2.0, Color::from_hex(0xf0c858));
    draw_circle(x - 4.0, y + 1.0, 1.5, Color::from_hex(0xf0c858));
    draw_circle(x + 2.0, y - 2.0, 1.5, Color::from_hex(0xf0c858));

    // ── Steam lines ─────────────────────────────────────────────────
    draw_line(x - 3.0, y - 16.0, x - 4.0, y - 20.0, 1.0, Color::from_rgba(200, 200, 200, 80));
    draw_line(x + 2.0, y - 14.0, x + 1.0, y - 18.0, 1.0, Color::from_rgba(200, 200, 200, 60));
}


// ── Toddler boss sprite drawing ──────────────────────────────────────────────

/// Draw the giant toddler boss at the given centre position.
fn draw_toddler_boss_sprite(cx: f32, cy: f32, b: &ToddlerBoss) {
    let flip = if b.facing_right { 1.0 } else { -1.0 };
    let t = b.walk_time;
    // Scale factor: the boss is 225px tall, so each unit = boss_height / 2
    let s = 112.5;
    let ox = |dx: f32| cx + dx * flip * s / 112.5;
    let wobble = if b.dead { 0.0 } else { (t * 3.0).sin() * 3.0 };

    // Skin color
    let skin = Color::from_hex(0xf0c8a0);
    let skin_shadow = Color::from_hex(0xd8a880);
    let diaper = Color::from_hex(0xffffff);
    let diaper_dark = Color::from_hex(0xdddddd);
    let hair = Color::from_hex(0x8b4513);
    let angry_red = Color::from_hex(0xcc3333);

    // ── Legs (stubby, wide stance) ──────────────────────────────────
    let leg_h = 30.0;
    let leg_w = 20.0;
    draw_rectangle(ox(-20.0) - leg_w / 2.0, cy + s * 0.3 + wobble, leg_w, leg_h, skin);
    draw_circle(ox(-20.0), cy + s * 0.3 + leg_h + wobble, 12.0, skin_shadow);
    draw_rectangle(ox(20.0) - leg_w / 2.0, cy + s * 0.3 + wobble, leg_w, leg_h, skin);
    draw_circle(ox(20.0), cy + s * 0.3 + leg_h + wobble, 12.0, skin_shadow);

    // ── Diaper ──────────────────────────────────────────────────────
    draw_rectangle(ox(-25.0), cy - s * 0.1 + wobble, 50.0, 35.0, diaper);
    draw_rectangle(ox(-22.0), cy - s * 0.05 + wobble, 44.0, 30.0, diaper_dark);
    // Diaper lines
    draw_line(ox(-10.0), cy - s * 0.1 + wobble, ox(-10.0), cy + s * 0.25 + wobble, 2.0, Color::from_hex(0xbbbbff));
    draw_line(ox(10.0), cy - s * 0.1 + wobble, ox(10.0), cy + s * 0.25 + wobble, 2.0, Color::from_hex(0xbbbbff));

    // ── Body ────────────────────────────────────────────────────────
    let body_r = 32.0;
    draw_circle(cx, cy - s * 0.2 + wobble, body_r, skin);
    draw_circle(cx, cy - s * 0.15 + wobble, body_r - 3.0, skin_shadow);
    // Angry chest pattern
    draw_circle(ox(-8.0), cy - s * 0.25 + wobble, 6.0, angry_red);
    draw_circle(ox(8.0), cy - s * 0.25 + wobble, 6.0, angry_red);

    // ── Arms ────────────────────────────────────────────────────────
    let arm_swing = (t * 4.0).sin() * 10.0;
    draw_rectangle(ox(-body_r - 8.0), cy - s * 0.2 + arm_swing + wobble, 12.0, 25.0, skin);
    draw_circle(ox(-body_r - 8.0), cy - s * 0.2 + arm_swing + wobble, 8.0, skin_shadow);
    draw_rectangle(ox(body_r - 4.0), cy - s * 0.2 - arm_swing + wobble, 12.0, 25.0, skin);
    draw_circle(ox(body_r - 4.0), cy - s * 0.2 - arm_swing + wobble, 8.0, skin_shadow);

    // ── Head ────────────────────────────────────────────────────────
    let head_cx = cx;
    let head_cy = cy - s * 0.9 + wobble;
    let head_r = 40.0;
    draw_circle(head_cx, head_cy, head_r, skin);
    draw_circle(head_cx, head_cy + 5.0, head_r - 2.0, skin_shadow);

    // Chubby cheeks
    draw_circle(head_cx - 25.0, head_cy + 8.0, 12.0, Color::from_hex(0xf0a0a0));
    draw_circle(head_cx + 25.0, head_cy + 8.0, 12.0, Color::from_hex(0xf0a0a0));

    // Hair (messy tufts)
    for i in 0..5 {
        let angle = -1.2 + i as f32 * 0.5 + (t * 2.0).sin() * 0.2;
        let hx = head_cx + angle.cos() * head_r * 0.7;
        let hy = head_cy - angle.sin() * head_r * 0.8;
        draw_circle(hx, hy, 10.0, hair);
    }
    draw_circle(head_cx - 5.0, head_cy - head_r + 5.0, 12.0, hair);

    // Angry eyebrows (dark thick lines)
    draw_line(head_cx - 18.0, head_cy - 12.0, head_cx - 5.0, head_cy - 8.0, 4.0, Color::from_hex(0x5a3a1a));
    draw_line(head_cx + 18.0, head_cy - 12.0, head_cx + 5.0, head_cy - 8.0, 4.0, Color::from_hex(0x5a3a1a));

    // Eyes
    draw_circle(head_cx - 12.0, head_cy, 8.0, Color::from_hex(0xffffff));
    draw_circle(head_cx + 12.0, head_cy, 8.0, Color::from_hex(0xffffff));
    draw_circle(head_cx - 11.0 + 2.0 * flip, head_cy, 5.0, Color::from_hex(0x2266aa));
    draw_circle(head_cx + 13.0 + 2.0 * flip, head_cy, 5.0, Color::from_hex(0x2266aa));
    draw_circle(head_cx - 11.0 + 2.0 * flip, head_cy - 1.0, 2.5, Color::from_hex(0x111111));
    draw_circle(head_cx + 13.0 + 2.0 * flip, head_cy - 1.0, 2.5, Color::from_hex(0x111111));

    // Angry mouth (open, teeth visible)
    draw_circle(head_cx, head_cy + 15.0, 10.0, Color::from_hex(0x331111));
    draw_rectangle(head_cx - 8.0, head_cy + 10.0, 16.0, 8.0, Color::from_hex(0x331111));
    // Teeth
    for i in 0..4 {
        let tx = head_cx - 7.0 + i as f32 * 4.5;
        draw_rectangle(tx, head_cy + 10.0, 3.0, 5.0, Color::from_hex(0xffffff));
    }

    // ── Tears (anger crying) ─────────────────────────────────────────
    let tear_phase = (t * 5.0).sin();
    if tear_phase > 0.0 {
        draw_circle(head_cx - 16.0, head_cy + 12.0 + tear_phase * 5.0, 3.0, Color::from_hex(0x4488cc));
        draw_circle(head_cx + 16.0, head_cy + 10.0 + tear_phase * 4.0, 2.5, Color::from_hex(0x4488cc));
    }
}


// ── Level file parser ───────────────────────────────────────────────────────

/// Parse a level definition from ASCII data into level components.
fn parse_level(data: &str, floor_y: f32) -> LevelData {
    let spike_y = floor_y + 40.0 - SPIKE_HEIGHT;
    let mut platforms: Vec<Platform> = vec![];
    let mut spikes: Vec<Spike> = vec![];
    let mut babies: Vec<Baby> = vec![];
    let mut lava_pits: Vec<Lava> = vec![];
    let mut goal_ball: Option<GoalBall> = None;
    let mut player_start: Option<(f32, f32)> = None;
    let mut foods: Vec<Food> = vec![];

    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "P" if parts.len() >= 3 => {
                let x: f32 = parts[1].parse().unwrap_or(80.0);
                let y: f32 = parts[2].parse().unwrap_or(floor_y - PLAYER_HEIGHT);
                player_start = Some((x, y));
            }
            "#" if parts.len() >= 5 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(floor_y);
                let w: f32 = parts[3].parse().unwrap_or(100.0);
                let h: f32 = parts[4].parse().unwrap_or(40.0);
                platforms.push(Platform { pos: vec2(x, y), size: vec2(w, h) });
            }
            "S" if parts.len() >= 5 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(spike_y);
                let w: f32 = parts[3].parse().unwrap_or(100.0);
                let h: f32 = parts[4].parse().unwrap_or(SPIKE_HEIGHT);
                spikes.push(Spike { pos: vec2(x, y), width: w, height: h });
            }
            "L" if parts.len() >= 5 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(spike_y);
                let w: f32 = parts[3].parse().unwrap_or(100.0);
                let h: f32 = parts[4].parse().unwrap_or(SPIKE_HEIGHT);
                lava_pits.push(Lava { pos: vec2(x, y), width: w, height: h });
            }
            "B" if parts.len() >= 5 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let fy: f32 = parts[2].parse().unwrap_or(floor_y);
                let min_x: f32 = parts[3].parse().unwrap_or(0.0);
                let max_x: f32 = parts[4].parse().unwrap_or(200.0);
                babies.push(Baby::new(x, fy, min_x, max_x));
            }
            "C" if parts.len() >= 5 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let fy: f32 = parts[2].parse().unwrap_or(floor_y);
                let min_x: f32 = parts[3].parse().unwrap_or(0.0);
                let max_x: f32 = parts[4].parse().unwrap_or(200.0);
                babies.push(Baby::new_cat(x, fy, min_x, max_x));
            }
            "F" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let kind = match parts[3] {
                    "chicken" => FoodType::Chicken,
                    "burger" => FoodType::Burger,
                    "pizza" => FoodType::Pizza,
                    _ => FoodType::Bacon,
                };
                foods.push(Food::new(x, y, kind));
            }
            "G" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(floor_y - 50.0);
                let color = match parts[3] {
                    "green" => Color::from_hex(0x3cb371),
                    "blue" => Color::from_hex(0x4a90d9),
                    "red" => Color::from_hex(0xe94560),
                    "gold" | "yellow" => Color::from_hex(0xd4c73c),
                    "purple" => Color::from_hex(0x9944cc),
                    _ => Color::from_hex(0x3cb371),
                };
                goal_ball = Some(GoalBall::new(x, y, color));
            }
            _ => {}
        }
    }

    // If no player start was specified, use a default
    let _start = player_start.unwrap_or((80.0, floor_y - PLAYER_HEIGHT));
    // Store player start for Game::new() to use - but we can't return it easily
    // The calling code in Game::new() uses hardcoded start_x/start_y
    // For now, the level file P line is advisory; Game::new() uses its own start

    (platforms, spikes, babies, lava_pits, foods, goal_ball)
}

// ── Platform helpers ─────────────────────────────────────────────────────────

/// Find the left edge of the platform whose top is at  and whose
/// span contains . Returns  as a fallback if no match is found.
fn plat_start_x(x: f32, platforms: &[Platform], floor_y: f32) -> f32 {
    for p in platforms {
        if (p.pos.y - floor_y).abs() < 2.0 && x >= p.pos.x && x <= p.pos.x + p.size.x {
            return p.pos.x;
        }
    }
    (x - 200.0).max(0.0)
}

/// Find the right edge of the platform whose top is at  and whose
/// span contains . Returns  as a fallback if no match is found.
fn plat_end_x(x: f32, platforms: &[Platform], floor_y: f32) -> f32 {
    for p in platforms {
        if (p.pos.y - floor_y).abs() < 2.0 && x >= p.pos.x && x <= p.pos.x + p.size.x {
            return p.pos.x + p.size.x;
        }
    }
    x + 200.0
}


/// Generate RGBA pixel data for a 16×16 pixel-art golden tennis ball.
/// This is the source art; larger sizes are scaled from it.
fn tennis_ball_icon_16() -> [u8; 1024] {
    let mut px = [0u8; 1024];
    // '#' = golden yellow, 'L' = lighter highlight, 'O' = white seam, '.' = transparent
    let art: [&str; 16] = [
        "................",
        ".....#####......",
        "...#########....",
        "..###########...",
        ".###LLLLL####..",
        ".###LLLLL#####.",
        ".####LLLL#####.",
        "#######O#######",
        "#######O#######",
        ".#####O########.",
        ".####OOOO#####.",
        "..###OOOO####..",
        "...#########...",
        "....#######....",
        ".....#####.....",
        "................",
    ];
    for (y, row) in art.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            let i = (y * 16 + x) * 4;
            match ch {
                b'#' => { px[i]=215; px[i+1]=182; px[i+2]=50;  px[i+3]=255; }
                b'L' => { px[i]=235; px[i+1]=208; px[i+2]=80;  px[i+3]=255; }
                b'O' => { px[i]=250; px[i+1]=250; px[i+2]=250; px[i+3]=255; }
                _   => {}
            }
        }
    }
    px
}

/// Nearest-neighbour upscale a 16×16 RGBA pixel buffer to a larger square size.
fn upscale_icon(src: &[u8; 1024], new_size: usize) -> Vec<u8> {
    let scale = new_size / 16;
    let mut dst = vec![0u8; new_size * new_size * 4];
    for y in 0..new_size {
        for x in 0..new_size {
            let si = ((y / scale) * 16 + (x / scale)) * 4;
            let di = (y * new_size + x) * 4;
            dst[di]     = src[si];
            dst[di + 1] = src[si + 1];
            dst[di + 2] = src[si + 2];
            dst[di + 3] = src[si + 3];
        }
    }
    dst
}

fn window_conf() -> Conf {
    let small = tennis_ball_icon_16();
    // 32×32 and 64×64 are nearest-neighbour upscales of the 16×16 design
    let medium_arr: [u8; 4096] = {
        let v = upscale_icon(&small, 32);
        std::array::from_fn(|i| v[i])
    };
    let big_arr: [u8; 16384] = {
        let v = upscale_icon(&small, 64);
        std::array::from_fn(|i| v[i])
    };

    Conf {
        window_title: String::from("Dog Adventure"),
        icon: Some(miniquad::conf::Icon {
            small,
            medium: medium_arr,
            big: big_arr,
        }),
        ..Default::default()
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[macroquad::main(window_conf)]
async fn main() {
    let poop_sound = audio::load_sound("assets/poop.wav").await.ok();
    let death_sound = audio::load_sound("assets/death.wav").await.ok();
    let ouch_sound = audio::load_sound("assets/ouch.wav").await.ok();
    let cheer_sound = audio::load_sound("assets/cheer.wav").await.ok();

    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(0.05);

        // ── Update particles and death timer (always) ───────────────────
        if game.death_timer > 0.0 {
            game.death_timer -= dt;
        }
        if game.player.dead && !game.played_death_sound {
            game.played_death_sound = true;
            if let Some(s) = &death_sound {
                audio::play_sound_once(s);
            }
        }
        if game.play_ouch {
            game.play_ouch = false;
            if let Some(s) = &ouch_sound {
                audio::play_sound_once(s);
            }
        }
        if game.play_cheer {
            game.play_cheer = false;
            if let Some(s) = &cheer_sound {
                audio::play_sound_once(s);
            }
        }
        if game.complete_timer > 0.0 {
            game.complete_timer -= dt;
        }
        if game.invincible_timer > 0.0 {
            game.invincible_timer -= dt;
        }
        game.particles.retain_mut(|p| {
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                return false;
            }
            p.pos += p.vel * dt;
            true
        });

        if game.state == GameState::Title || game.state == GameState::Tutorial || game.state == GameState::LevelSelect || game.state == GameState::Paused {
            // ── Title golden particles ─────────────────────────────────
            if game.state == GameState::Title {
                let cw = screen_width();
                let ch = screen_height();
                let dog_cx = cw * 0.35;
                let dog_cy = ch * 0.48;
                let ball_cx = cw * 0.65;
                let ball_cy = ch * 0.48;
                for _ in 0..2 {
                    let src_x = if mq_rand::rand().is_multiple_of(2) { dog_cx } else { ball_cx };
                    let src_y = if mq_rand::rand().is_multiple_of(2) { dog_cy } else { ball_cy };
                    let angle = (mq_rand::rand() as f32 / u32::MAX as f32) * std::f32::consts::TAU;
                    let speed = (mq_rand::rand() as f32 / u32::MAX as f32) * 60.0 + 20.0;
                    let size = (mq_rand::rand() as f32 / u32::MAX as f32) * 3.0 + 2.0;
                    game.particles.push(Particle {
                        pos: vec2(src_x, src_y),
                        vel: vec2(angle.cos() * speed, angle.sin() * speed),
                        lifetime: (mq_rand::rand() as f32 / u32::MAX as f32) * 1.5 + 0.8,
                        size,
                        color_override: Some(Color::from_hex(0xffd700)),
                    });
                }
            }

            if game.state == GameState::Title {
                // ── Title screen input ─────────────────────────────────
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    let cw = screen_width();
                    let ch = screen_height();
                    let bw = 220.0;
                    let bh = 50.0;
                    let bx = (cw - bw) / 2.0;
                    let by = ch * 0.62;
                    let gap = 64.0;

                    if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
                        game.reset(); // Play
                    }
                    if mx >= bx && mx <= bx + bw && my >= by + gap && my <= by + gap + bh {
                        game.state = GameState::LevelSelect; // Levels
                    }
                    if mx >= bx && mx <= bx + bw && my >= by + gap * 2.0 && my <= by + gap * 2.0 + bh {
                        game.state = GameState::Tutorial;
                    }
                    if mx >= bx && mx <= bx + bw && my >= by + gap * 3.0 && my <= by + gap * 3.0 + bh {
                        std::process::exit(0);
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
            } else if game.state == GameState::LevelSelect {
                // ── Level select input ─────────────────────────────────
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    let cw = screen_width();
                    let ch = screen_height();
                    let bw = 160.0;
                    let bh = 120.0;
                    let cols = 3;
                    let spacing = 40.0;
                    let total_w = cols as f32 * bw + (cols - 1) as f32 * spacing;
                    let start_x = (cw - total_w) / 2.0;
                    let start_y = ch * 0.35;

                    for i in 0..4 {
                        let col = i % cols;
                        let row = i / cols;
                        let bx = start_x + col as f32 * (bw + spacing);
                        let by = start_y + row as f32 * (bh + spacing);
                        if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
                            game.start_level(i as u32 + 1);
                            break;
                        }
                    }

                    // Back button
                    let back_bw = 180.0;
                    let back_bh = 44.0;
                    let back_bx = (cw - back_bw) / 2.0;
                    let back_by = ch * 0.82;
                    if mx >= back_bx && mx <= back_bx + back_bw && my >= back_by && my <= back_by + back_bh {
                        game.state = GameState::Title;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Title;
                }
            } else if game.state == GameState::Tutorial {
                // ── Tutorial screen input ──────────────────────────────
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Title;
                }
            }
            if game.state == GameState::Paused {
                // ── Pause menu input ────────────────────────────────────
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    let cw = screen_width();
                    let ch = screen_height();
                    let bw = 220.0;
                    let bh = 50.0;
                    let bx = (cw - bw) / 2.0;
                    let start_y = ch * 0.38;
                    let gap = 60.0;

                    if mx >= bx && mx <= bx + bw && my >= start_y && my <= start_y + bh {
                        game.state = GameState::Playing; // Resume
                    }
                    if mx >= bx && mx <= bx + bw && my >= start_y + gap && my <= start_y + gap + bh {
                        game.state = GameState::Title; // Main Menu
                        game.level_complete = false;
                    }
                    if mx >= bx && mx <= bx + bw && my >= start_y + gap * 2.0 && my <= start_y + gap * 2.0 + bh {
                        std::process::exit(0); // Quit
                    }
                    // Dev Mode toggle
                    if mx >= bx && mx <= bx + bw && my >= start_y + gap * 3.0 && my <= start_y + gap * 3.0 + bh {
                        game.dev_mode = !game.dev_mode;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing; // Resume
                }
            }
        } else {
            // Playing state
                if !game.player.dead && !game.level_complete {
                    game.update_player(dt);

                    if !game.pee_mode {
                    for baby in &mut game.babies {
                        if baby.flee_timer > 0.0 {
                            baby.flee_timer -= dt;
                            baby.vel.y += GRAVITY * dt;
                            baby.vel.y = baby.vel.y.clamp(-1200.0, 1200.0);
                            baby.pos.x += baby.vel.x * dt;
                            baby.pos.y += baby.vel.y * dt;

                            let mut landed = false;
                            'platforms: for plat in &game.platforms {
                                if let Some(collision) = baby.rect().intersect(plat.rect())
                                    && baby.vel.y > 0.0
                                {
                                    baby.pos.y -= collision.h;
                                    baby.vel.y = 0.0;
                                    baby.floor_y = plat.pos.y;
                                    landed = true;
                                    break 'platforms;
                                }
                            }

                            if baby.flee_timer <= 0.0 && landed {
                                let dir = if baby.facing_right { 1.0 } else { -1.0 };
                                baby.vel.x = dir * BABY_SPEED;
                                baby.min_x = plat_start_x(baby.pos.x, &game.platforms, baby.floor_y);
                                baby.max_x = plat_end_x(baby.pos.x, &game.platforms, baby.floor_y);
                            }
                        } else {
                            baby.update(dt);

                            let baby_cx = baby.pos.x + baby.size.x / 2.0;
                            for poop in &mut game.poops {
                                if poop.eaten { continue; }
                                let same_plat = (baby.floor_y - poop.pos.y).abs() < 30.0;
                                let dist = (baby_cx - poop.pos.x).abs();
                                if same_plat && dist < 100.0
                                    && !mq_rand::rand().is_multiple_of(4)
                                {
                                    // 75%: flee away from the poop
                                    let dir = if baby_cx < poop.pos.x { -1.0 } else { 1.0 };
                                    baby.vel.x = dir * BABY_SPEED * 2.0;
                                    baby.facing_right = dir > 0.0;
                                    baby.flee_timer = 2.5;
                                    baby.vel.y = 0.0;
                                    break;
                                }
                                // 25%: not scared — keep coming
                                // Eat the poop if touching it
                                if baby.rect().intersect(poop.rect()).is_some() {
                                    poop.eaten = true;
                                }
                            }
                        }
                    }
                    }

                    if is_key_pressed(KeyCode::Q) {
                        let flip: f32 = if game.player.facing_right { 1.0 } else { -1.0 };
                        let px = game.player.pos.x + game.player.size.x / 2.0 - 8.0 * flip;
                        let py = game.player.pos.y + game.player.size.y - 30.0;
                        game.poops.push(Poop::new(px, py));
                        if let Some(s) = &poop_sound {
                            audio::play_sound_once(s);
                        }
                    }

                    // ── Poop physics ────────────────────────────────────────
                    game.poops.retain_mut(|poop| {
                        if poop.eaten {
                            return false;
                        }
                        poop.lifetime -= dt;
                        if poop.lifetime <= 0.0 {
                            return false;
                        }
                        if poop.vel_y != 0.0 {
                            poop.vel_y += GRAVITY * dt;
                            poop.pos.y += poop.vel_y * dt;
                            for plat in &game.platforms {
                                let at_x = poop.pos.x > plat.pos.x && poop.pos.x < plat.pos.x + plat.size.x;
                                let hit = poop.pos.y + 5.0 >= plat.pos.y
                                    && poop.pos.y + 5.0 <= plat.pos.y + 10.0
                                    && poop.vel_y > 0.0;
                                if at_x && hit {
                                    poop.pos.y = plat.pos.y - 5.0;
                                    poop.vel_y = 0.0;
                                    break;
                                }
                            }
                        }
                        true
                    });
                } else if game.death_timer <= 0.0 && is_key_pressed(KeyCode::Space) {
                    game.reset();
                }

                // Pee whenever holding E (after done_pee too)
                if game.pee_mode && is_key_down(KeyCode::E) && !game.player.dead {
                    // Dropping yellow droplets from the dog's bum
                    let flip: f32 = if game.player.facing_right { 1.0 } else { -1.0 };
                    let dog_bum_x = game.player.pos.x + game.player.size.x / 2.0 - 15.0 * flip * DOG_SCALE;
                    let dog_bum_y = game.player.pos.y + game.player.size.y / 2.0 - 5.0;
                    // Spawn a falling droplet
                    game.particles.push(Particle {
                        pos: vec2(dog_bum_x, dog_bum_y),
                        vel: vec2(0.0, 120.0),
                        lifetime: 0.8,
                        size: 5.0,
                        color_override: Some(Color::from_hex(0xcccc00)),
                    });
                    // Only check tree collision before done_pee
                    if !game.done_pee {
                        // Check if stream hits the tree trunk (bottom of tree)
                        let stream_left = dog_bum_x - 6.0;
                        let stream_right = dog_bum_x + 6.0;
                        let stream_top = dog_bum_y;
                        let stream_bottom = dog_bum_y + 100.0;
                        // Find the platform under the tree (world coordinates)
                        let mut tree_ground_world = screen_height() as f32 - 40.0;
                        for plat in &game.platforms {
                            if game.tree_x > plat.pos.x && game.tree_x < plat.pos.x + plat.size.x {
                                tree_ground_world = plat.pos.y;
                                break;
                            }
                        }
                        let tree_trunk_h = PLAYER_HEIGHT * DOG_SCALE * 1.5;
                        let tree_trunk_w = PLAYER_HEIGHT * DOG_SCALE * 0.36;
                        let tree_left = game.tree_x - tree_trunk_w / 2.0;
                        let tree_right = game.tree_x + tree_trunk_w / 2.0;
                        let tree_top = tree_ground_world - tree_trunk_h;
                        let tree_bottom = tree_ground_world;
                        if stream_left < tree_right && stream_right > tree_left
                            && stream_top < tree_bottom && stream_bottom > tree_top
                        {
                            game.hearts = 4;
                            game.done_pee = true;
                        }
                    }
                }
                // Pick up the rotisserie chicken
                if game.done_pee && !game.player.super_mode {
                    let chicken_x = game.tree_x - 80.0;
                    let mut chicken_y = screen_height() - 40.0 - 30.0;
                    for plat in &game.platforms {
                        if game.tree_x > plat.pos.x && game.tree_x < plat.pos.x + plat.size.x {
                            chicken_y = plat.pos.y - 30.0;
                            break;
                        }
                    }
                    let player_rect = game.player.rect();
                    let chicken_rect = Rect::new(chicken_x - 12.0, chicken_y - 10.0, 24.0, 20.0);
                    if player_rect.intersect(chicken_rect).is_some() {
                        game.player.super_mode = true;
                        game.boss_mode = true;
                        game.boss = Some(ToddlerBoss::new(6800.0, 700.0));
                        // Clear all level objects
                        game.platforms.clear();
                        game.babies.clear();
                        game.spikes.clear();
                        game.lava_pits.clear();
                        game.foods.clear();
                        game.poops.clear();
                        game.goal_ball = None;
                    }
                }

                // ── Boss fight update ───────────────────────────────────────
                if game.boss_mode {
                    if let Some(boss) = &mut game.boss {
                        boss.update(dt, game.player.pos);

                        // Pee damages the boss
                        if game.pee_mode && is_key_down(KeyCode::E) && !boss.dead {
                            let flip: f32 = if game.player.facing_right { 1.0 } else { -1.0 };
                            let dog_bum_x = game.player.pos.x + game.player.size.x / 2.0 - 15.0 * flip * DOG_SCALE;
                            let dog_bum_y = game.player.pos.y + game.player.size.y / 2.0 - 5.0;
                            let stream_left = dog_bum_x - 6.0;
                            let stream_right = dog_bum_x + 6.0;
                            let stream_top = dog_bum_y;
                            let stream_bottom = dog_bum_y + 100.0;
                            let boss_rect = boss.rect();
                            if stream_left < boss_rect.right() && stream_right > boss_rect.left()
                                && stream_top < boss_rect.bottom() && stream_bottom > boss_rect.top()
                            {
                                boss.take_damage(0.2);
                            }
                        }

                        // Boss death
                        if boss.dead && boss.death_timer <= 0.0 {
                            // Victory! Show congratulations
                        }
                    }
                }

                // Boss contact damage (outside borrow)
                if game.boss_mode {
                    if let Some(boss) = &game.boss {
                        if !boss.dead && game.invincible_timer <= 0.0 {
                            if game.player.rect().intersect(boss.rect()).is_some() {
                                if game.hearts > 1 {
                                    game.hearts -= 1;
                                    game.play_ouch = true;
                                    game.invincible_timer = 1.5;
                                    game.death_message = "The toddler stepped on you!".to_string();
                                } else if !game.player.dead {
                                    game.death_message = "The toddler stepped on you!".to_string();
                                    game.die();
                                }
                            }
                        }
                    }
                }
                // Level 4: auto-transition to pee mode after timer expires
                if game.level_complete && game.complete_timer <= 0.0 && game.level == 4 {
                    game.pee_mode = true;
                    game.level_complete = false;
                }
                // Other levels: Space to continue (or skip timer)
                if game.level_complete && is_key_pressed(KeyCode::Space) && game.level != 4 {
                    game.next_level();
                }

                // Lock R and Escape during level 4 completion fade
                if !(game.level == 4 && game.level_complete) {
                    if is_key_pressed(KeyCode::R) {
                        game.reset();
                    }

                    if is_key_pressed(KeyCode::Escape) {
                        game.state = GameState::Paused;
                    }
                }

                // ── Goal ball update ────────────────────────────────────────
                if let Some(ball) = &mut game.goal_ball
                    && !game.level_complete && !ball.collected
                {
                    // Bounce physics
                    ball.vel.y += GRAVITY * dt;
                    ball.pos += ball.vel * dt;

                    // Bounce off platforms (find the platform directly below the ball)
                    for plat in &game.platforms {
                        let ball_bottom = ball.pos.y + 8.0;
                        let plat_top = plat.pos.y;
                        let overlaps_x = ball.pos.x + 8.0 > plat.pos.x
                            && ball.pos.x - 8.0 < plat.pos.x + plat.size.x;
                        if overlaps_x
                            && ball_bottom >= plat_top
                            && ball_bottom <= plat_top + 12.0
                            && ball.vel.y > 0.0
                        {
                            ball.pos.y = plat_top - 8.0;
                            ball.vel.y *= -0.6;
                            break;
                        }
                    }

                    // Bounce off walls (use the full width of the platform the ball is on)
                    let plat_bounds = game.platforms.iter().find_map(|p| {
                        let overlaps_x = ball.pos.x + 8.0 > p.pos.x && ball.pos.x - 8.0 < p.pos.x + p.size.x;
                        let near_surface = (ball.pos.y - p.pos.y).abs() < 60.0;
                        if overlaps_x && near_surface { Some((p.pos.x, p.pos.x + p.size.x)) } else { None }
                    });
                    if let Some((min_x, max_x)) = plat_bounds {
                        if ball.pos.x - 8.0 < min_x { ball.pos.x = min_x + 8.0; ball.vel.x = 80.0; }
                        if ball.pos.x + 8.0 > max_x { ball.pos.x = max_x - 8.0; ball.vel.x = -80.0; }
                    }

                    // Player collision -> fetch! (only if all food is collected when food is present)
                    let can_fetch = game.food_total == 0 || game.food_collected >= game.food_total;
                    if game.player.rect().intersect(ball.rect()).is_some() && can_fetch {
                        ball.collected = true;
                        game.level_complete = true;
                        game.complete_timer = if game.level == 4 { 2.0 } else { 0.5 };
                        game.play_cheer = true;
                        for _ in 0..35 {
                            let angle = (mq_rand::rand() as f32 / u32::MAX as f32) * std::f32::consts::TAU;
                            let speed = (mq_rand::rand() as f32 / u32::MAX as f32) * 300.0 + 100.0;
                            let size = (mq_rand::rand() as f32 / u32::MAX as f32) * 6.0 + 3.0;
                            game.particles.push(Particle {
                                pos: ball.pos,
                                vel: vec2(angle.cos() * speed, angle.sin() * speed),
                                lifetime: (mq_rand::rand() as f32 / u32::MAX as f32) * 0.8 + 0.3,
                                size,
                                color_override: Some(ball.color),
                            });
                        }
                    }
                }
        }

        // ── Draw ────────────────────────────────────────────────────────
        game.draw();

        next_frame().await
    }
}
