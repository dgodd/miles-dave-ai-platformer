use macroquad::prelude::*;
use macroquad::rand as mq_rand;

// ── Constants ────────────────────────────────────────────────────────────────

const GRAVITY: f32 = 980.0;
const JUMP_VELOCITY: f32 = -520.0;
const MOVE_SPEED: f32 = 280.0;
const PLAYER_WIDTH: f32 = 28.0;
const PLAYER_HEIGHT: f32 = 36.0;

const COYOTE_FRAMES: i32 = 6;
const JUMP_BUFFER_FRAMES: i32 = 5;

const SPIKE_HEIGHT: f32 = 24.0;
const SPIKE_TOOTH_WIDTH: f32 = 20.0;

const BABY_SPEED: f32 = 65.0;

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
    eaten: bool,
}

impl Poop {
    fn new(x: f32, y: f32) -> Self {
        Self { pos: vec2(x, y), eaten: false }
    }
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
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    fn update(&mut self, dt: f32) {
        self.crawl_time += dt;

        if self.flee_timer > 0.0 {
            // Fleeing from a poop — run double speed, ignore patrol bounds
            self.flee_timer -= dt;
            self.pos.x += self.vel.x * dt;
            if self.flee_timer <= 0.0 {
                // Resume normal patrol
                let dir = if self.facing_right { 1.0 } else { -1.0 };
                self.vel.x = dir * BABY_SPEED;
            }
        } else {
            // Normal patrol
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
        }

        // Stay on the floor
        self.pos.y = self.floor_y - self.size.y;
    }
}

// ── Game state ───────────────────────────────────────────────────────────────

struct Game {
    player: Player,
    platforms: Vec<Platform>,
    spikes: Vec<Spike>,
    babies: Vec<Baby>,
    poops: Vec<Poop>,
}

impl Game {
    fn new() -> Self {
        let floor_y = screen_height() - 40.0;

        let platforms = vec![
            Platform { pos: vec2(0.0, floor_y), size: vec2(800.0, 40.0) },
            Platform { pos: vec2(220.0, screen_height() - 130.0), size: vec2(140.0, 20.0) },
            Platform { pos: vec2(420.0, screen_height() - 240.0), size: vec2(140.0, 20.0) },
            Platform { pos: vec2(620.0, screen_height() - 350.0), size: vec2(140.0, 20.0) },
            Platform { pos: vec2(850.0, screen_height() - 180.0), size: vec2(130.0, 20.0) },
            Platform { pos: vec2(1050.0, screen_height() - 280.0), size: vec2(130.0, 20.0) },
            Platform { pos: vec2(1250.0, screen_height() - 400.0), size: vec2(250.0, 20.0) },
            Platform { pos: vec2(1600.0, screen_height() - 300.0), size: vec2(100.0, 20.0) },
            Platform { pos: vec2(1800.0, floor_y), size: vec2(400.0, 40.0) },
        ];

        let spike_y = floor_y + 40.0 - SPIKE_HEIGHT;
        let spikes = vec![
            Spike { pos: vec2(800.0, spike_y), width: 1000.0, height: SPIKE_HEIGHT },
            Spike { pos: vec2(2200.0, spike_y), width: 200.0, height: SPIKE_HEIGHT },
        ];

        // Babies patrol the full width of their platform
        let babies = vec![
            Baby::new(550.0, floor_y, 0.0, 800.0),
            Baby::new(290.0, screen_height() - 130.0, 220.0, 360.0),
            Baby::new(490.0, screen_height() - 240.0, 420.0, 560.0),
            Baby::new(250.0, floor_y, 0.0, 800.0),
            Baby::new(700.0, floor_y, 0.0, 800.0),
            Baby::new(1950.0, floor_y, 1800.0, 2200.0),
        ];

        let start_x = 80.0;
        let start_y = floor_y - PLAYER_HEIGHT;

        Self {
            player: Player::new(start_x, start_y),
            platforms,
            spikes,
            babies,
            poops: vec![],
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
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
                self.player.dead = true;
                break;
            }
        }

        // ── Baby collisions ─────────────────────────────────────────────
        if !self.player.dead {
            for baby in &self.babies {
                if self.player.rect().intersect(baby.rect()).is_some() {
                    self.player.dead = true;
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
        let cam = self.camera_offset();

        // ── Background ──────────────────────────────────────────────────
        clear_background(Color::from_hex(0x1a1a2e));

        // ── Platforms ───────────────────────────────────────────────────
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

        // ── Babies ──────────────────────────────────────────────────────
        for baby in &self.babies {
            let bx = baby.pos.x + baby.size.x / 2.0 - cam.x;
            let by = baby.pos.y + baby.size.y / 2.0 - cam.y;
            draw_baby_sprite(bx, by, baby);
        }

        // ── Poops ──────────────────────────────────────────────────────
        for poop in &self.poops {
            if !poop.eaten {
                let sx = poop.pos.x - cam.x;
                let sy = poop.pos.y - cam.y;
                draw_poop_sprite(sx, sy);
            }
        }

        // ── Player ──────────────────────────────────────────────────────
        if !self.player.dead {
            let psx = self.player.pos.x + self.player.size.x / 2.0 - cam.x;
            let psy = self.player.pos.y + self.player.size.y / 2.0 - cam.y;
            draw_dog_sprite(psx, psy, &self.player);
        }

        // ── HUD ─────────────────────────────────────────────────────────
        let hud_text = format!("x: {:.0}  y: {:.0}  grounded: {}", self.player.pos.x, self.player.pos.y, self.player.grounded);
        draw_text(&hud_text, 12.0, 28.0, 20.0, Color::from_hex(0xaaaaaa));
        draw_text("Arrow keys / WASD to move, Space to jump  |  E to poop  |  R to reset", 12.0, screen_height() - 12.0, 16.0, Color::from_hex(0x666666));

        // ── Death overlay ───────────────────────────────────────────────
        if self.player.dead {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                           Color::from_rgba(0, 0, 0, 180));

            let title = "The baby pulled your tail";
            let title_size = measure_text(title, None, 48, 1.0);
            draw_text(title, screen_width() / 2.0 - title_size.width / 2.0, screen_height() / 2.0 - 20.0,
                      48.0, Color::from_hex(0xcc0000));

            let subtitle = "Press Space to respawn";
            let sub_size = measure_text(subtitle, None, 22, 1.0);
            draw_text(subtitle, screen_width() / 2.0 - sub_size.width / 2.0, screen_height() / 2.0 + 30.0,
                      22.0, Color::from_hex(0xaaaaaa));
        }
    }
}

// ── Dog sprite drawing ───────────────────────────────────────────────────────

fn draw_dog_sprite(cx: f32, cy: f32, p: &Player) {
    let flip = if p.facing_right { 1.0 } else { -1.0 };
    let t = p.walk_time;
    let s = DOG_SCALE;

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

    let tail_anchor_x = -15.0 * s;
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

// ── Baby sprite drawing ──────────────────────────────────────────────────────

/// Draw a crawling baby at the given centre position.
fn draw_baby_sprite(cx: f32, cy: f32, b: &Baby) {
    let flip = if b.facing_right { 1.0 } else { -1.0 };
    let t = b.crawl_time;

    // Crawl-cycle arm/leg rock
    let crawl = (t * 7.0).sin();

    let ox = |dx: f32| cx + dx * flip;

    // ── Body (torso — small, slightly tilted) ──────────────────────────
    let body_cy = cy + 1.0;
    draw_rectangle(ox(-5.0), body_cy - 4.0, 10.0, 8.0, BABY_SKIN);

    // ── Diaper (round bottom) ──────────────────────────────────────────
    draw_circle(ox(0.0), body_cy + 4.0, 6.0, BABY_DIAPER);
    draw_circle(ox(0.0), body_cy + 4.0, 5.0, BABY_DIAPER_DARK);

    // ── Back arm (left side, behind body) ──────────────────────────────
    let back_arm_x = ox(-5.0) + crawl * 2.0;
    draw_rectangle(back_arm_x - 1.5, body_cy + 2.0, 3.0, 6.0, BABY_SKIN_SHADOW);
    draw_circle(back_arm_x, body_cy + 8.0, 2.5, BABY_SKIN_SHADOW);

    // ── Back leg ────────────────────────────────────────────────────────
    let back_leg_x = ox(-3.0) - crawl * 1.5;
    draw_rectangle(back_leg_x - 1.5, body_cy + 4.0, 3.0, 5.0, BABY_SKIN_SHADOW);
    draw_circle(back_leg_x, body_cy + 9.0, 2.0, BABY_SKIN_SHADOW);

    // ── Front arm (visible side) ────────────────────────────────────────
    let front_arm_x = ox(5.0) - crawl * 2.0;
    draw_rectangle(front_arm_x - 1.5, body_cy + 2.0, 3.0, 6.0, BABY_SKIN);
    draw_circle(front_arm_x, body_cy + 8.0, 2.5, BABY_SKIN);

    // ── Front leg ───────────────────────────────────────────────────────
    let front_leg_x = ox(3.0) + crawl * 1.5;
    draw_rectangle(front_leg_x - 1.5, body_cy + 4.0, 3.0, 5.0, BABY_SKIN);
    draw_circle(front_leg_x, body_cy + 9.0, 2.0, BABY_SKIN);

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

// ── Entry point ──────────────────────────────────────────────────────────────

#[macroquad::main("Platformer")]
async fn main() {
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(0.05);

        // ── Update ──────────────────────────────────────────────────────
        if !game.player.dead {
            game.update_player(dt);
            for baby in &mut game.babies {
                baby.update(dt);
            }

            // Poop interaction: babies detect and react to poops
            for baby in &mut game.babies {
                if baby.flee_timer > 0.0 {
                    continue;
                }
                let baby_cx = baby.pos.x + baby.size.x / 2.0;
                for poop in &mut game.poops {
                    if poop.eaten {
                        continue;
                    }
                    let dist = (baby_cx - poop.pos.x).abs();
                    if dist < 120.0 {
                        if !mq_rand::rand().is_multiple_of(4) {
                            // 75%: flee away from the poop
                            let dir = if baby_cx < poop.pos.x { -1.0 } else { 1.0 };
                            baby.vel.x = dir * BABY_SPEED * 2.0;
                            baby.facing_right = dir > 0.0;
                            baby.flee_timer = 2.5;
                        } else {
                            // 25%: eat it
                            poop.eaten = true;
                        }
                        break;
                    }
                }
            }

            // Spawn a poop when pressing E
            if is_key_pressed(KeyCode::E) {
                let px = game.player.pos.x + game.player.size.x / 2.0;
                let py = game.player.pos.y + game.player.size.y;
                game.poops.push(Poop::new(px, py));
            }
        } else if is_key_pressed(KeyCode::Space) {
            game.reset();
        }

        if is_key_pressed(KeyCode::R) {
            game.reset();
        }

        // ── Draw ────────────────────────────────────────────────────────
        game.draw();

        next_frame().await
    }
}
