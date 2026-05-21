use macroquad::prelude::*;

// ── Constants ────────────────────────────────────────────────────────────────

const GRAVITY: f32 = 980.0;       // pixels/sec^2
const JUMP_VELOCITY: f32 = -520.0; // pixels/sec (upward)
const MOVE_SPEED: f32 = 280.0;     // pixels/sec
const PLAYER_WIDTH: f32 = 28.0;
const PLAYER_HEIGHT: f32 = 36.0;

/// The "coyote time" window — frames after leaving a ledge where jump is still
/// allowed. Makes the controls feel much more responsive.
const COYOTE_FRAMES: i32 = 6;

/// Frames during which a jump is buffered so pressing the button slightly
/// before landing still triggers an immediate jump.
const JUMP_BUFFER_FRAMES: i32 = 5;

const SPIKE_HEIGHT: f32 = 24.0;
const SPIKE_TOOTH_WIDTH: f32 = 20.0;

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

/// A spike pit that kills the player on contact.
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

    /// Draw the spike pit as a row of triangular teeth.
    fn draw(&self, offset: Vec2) {
        let sx = self.pos.x - offset.x;
        let sy = self.pos.y - offset.y;

        let tooth_count = (self.width / SPIKE_TOOTH_WIDTH).ceil() as usize;
        let spacing = self.width / tooth_count as f32;

        // Pit background
        draw_rectangle(sx, sy, self.width, self.height, Color::from_hex(0x0d0d1a));

        for i in 0..tooth_count {
            let cx = sx + i as f32 * spacing + spacing / 2.0;
            let left = cx - spacing / 2.0;
            let right = cx + spacing / 2.0;
            let bottom = sy + self.height;
            let tip_y = sy + 2.0; // point up

            // Outline
            draw_triangle(
                vec2(left, bottom),
                vec2(right, bottom),
                vec2(cx, tip_y),
                Color::from_hex(0x8b0000),
            );
            // Inner highlight
            draw_triangle(
                vec2(left + 2.0, bottom - 1.0),
                vec2(right - 2.0, bottom - 1.0),
                vec2(cx, tip_y + 3.0),
                Color::from_hex(0xcc0000),
            );
        }
    }
}

// ── Game state ───────────────────────────────────────────────────────────────

struct Game {
    player: Player,
    platforms: Vec<Platform>,
    spikes: Vec<Spike>,
}

impl Game {
    fn new() -> Self {
        let floor_y = screen_height() - 40.0;

        let platforms = vec![
            // Ground
            Platform { pos: vec2(0.0, floor_y), size: vec2(800.0, 40.0) },
            // Floating platforms going up and to the right
            Platform { pos: vec2(220.0, screen_height() - 130.0), size: vec2(140.0, 20.0) },
            Platform { pos: vec2(420.0, screen_height() - 240.0), size: vec2(140.0, 20.0) },
            Platform { pos: vec2(620.0, screen_height() - 350.0), size: vec2(140.0, 20.0) },
            // Higher challenge platforms
            Platform { pos: vec2(850.0, screen_height() - 180.0), size: vec2(130.0, 20.0) },
            Platform { pos: vec2(1050.0, screen_height() - 280.0), size: vec2(130.0, 20.0) },
            // A long high platform
            Platform { pos: vec2(1250.0, screen_height() - 400.0), size: vec2(250.0, 20.0) },
            // A small tricky jump
            Platform { pos: vec2(1600.0, screen_height() - 300.0), size: vec2(100.0, 20.0) },
            // Final ground stretch
            Platform { pos: vec2(1800.0, floor_y), size: vec2(400.0, 40.0) },
        ];

        // Spike pits fill every gap between ground-level platforms, plus a
        // long stretch below the elevated platforms.
        let spike_y = floor_y + 40.0 - SPIKE_HEIGHT; // flush against the ground
        let spikes = vec![
            // Big pit between ground segment 1 (ends x=800) and ground segment 2 (starts x=1800)
            Spike { pos: vec2(800.0, spike_y), width: 1000.0, height: SPIKE_HEIGHT },
            // Pit after the final ground segment (ends x=2200)
            Spike { pos: vec2(2200.0, spike_y), width: 200.0, height: SPIKE_HEIGHT },
        ];

        let start_x = 80.0;
        let start_y = floor_y - PLAYER_HEIGHT;

        Self {
            player: Player::new(start_x, start_y),
            platforms,
            spikes,
        }
    }

    /// Move the player and resolve collisions with every platform.
    fn update_player(&mut self, dt: f32) {
        // ── Input ────────────────────────────────────────────────────────
        let mut move_x = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            move_x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            move_x += 1.0;
        }

        // Track facing direction for visual feedback
        if move_x > 0.0 {
            self.player.facing_right = true;
        } else if move_x < 0.0 {
            self.player.facing_right = false;
        }

        // Jump input buffering
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            self.player.jump_buffer_counter = JUMP_BUFFER_FRAMES;
        } else {
            self.player.jump_buffer_counter = self.player.jump_buffer_counter.saturating_sub(1);
        }

        // ── Apply horizontal movement ───────────────────────────────────
        self.player.vel.x = move_x * MOVE_SPEED;

        // ── Apply gravity ───────────────────────────────────────────────
        self.player.vel.y += GRAVITY * dt;

        // Cap fall speed so we don't clip through thin platforms
        self.player.vel.y = self.player.vel.y.clamp(-1200.0, 1200.0);

        // ── Coyote time ─────────────────────────────────────────────────
        if self.player.grounded {
            self.player.coyote_counter = COYOTE_FRAMES;
        } else {
            self.player.coyote_counter = self.player.coyote_counter.saturating_sub(1);
        }

        // ── Jump ────────────────────────────────────────────────────────
        if self.player.jump_buffer_counter > 0 && self.player.coyote_counter > 0 {
            self.player.vel.y = JUMP_VELOCITY;
            self.player.grounded = false;
            self.player.coyote_counter = 0;
            self.player.jump_buffer_counter = 0;
        }

        // Variable jump height: if the player releases jump while rising,
        // cut the upward velocity in half.
        let jump_released = is_key_released(KeyCode::Space)
            || is_key_released(KeyCode::W)
            || is_key_released(KeyCode::Up);
        if self.player.vel.y < 0.0 && jump_released {
            self.player.vel.y *= 0.5;
        }

        // ── Integrate position ──────────────────────────────────────────
        self.player.pos += self.player.vel * dt;

        // ── Platform collisions ─────────────────────────────────────────
        self.player.grounded = false;

        for plat in &self.platforms {
            if let Some(collision) = self.player.rect().intersect(plat.rect()) {
                let overlap_x = collision.w;
                let overlap_y = collision.h;

                if overlap_x < overlap_y {
                    // Horizontal push-out
                    if self.player.vel.x > 0.0 {
                        self.player.pos.x -= overlap_x;
                    } else if self.player.vel.x < 0.0 {
                        self.player.pos.x += overlap_x;
                    }
                    self.player.vel.x = 0.0;
                } else {
                    // Vertical push-out
                    if self.player.vel.y > 0.0 {
                        // Landing on top
                        self.player.pos.y -= overlap_y;
                        self.player.vel.y = 0.0;
                        self.player.grounded = true;
                    } else if self.player.vel.y < 0.0 {
                        // Hitting head on bottom
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

        // ── Clamp to screen boundaries horizontally ─────────────────────
        if self.player.pos.x < 0.0 {
            self.player.pos.x = 0.0;
            self.player.vel.x = 0.0;
        }
    }

    /// Compute the camera offset so the player is roughly centred.
    fn camera_offset(&self) -> Vec2 {
        let target_x = self.player.pos.x + self.player.size.x / 2.0 - screen_width() / 2.0;
        let target_y = self.player.pos.y + self.player.size.y / 2.0 - screen_height() / 2.0;

        let target_x = target_x.max(0.0);

        vec2(target_x, target_y.max(0.0))
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

        // ── Player ──────────────────────────────────────────────────────
        if !self.player.dead {
            let psx = self.player.pos.x - cam.x;
            let psy = self.player.pos.y - cam.y;

            let body_color = if self.player.grounded {
                Color::from_hex(0xe94560)
            } else {
                Color::from_hex(0xf5a623)
            };
            draw_rectangle(psx, psy, self.player.size.x, self.player.size.y, body_color);
            draw_rectangle(psx + 3.0, psy + 3.0, self.player.size.x - 6.0, self.player.size.y - 6.0,
                           Color::from_rgba(255, 255, 255, 30));

            let (eye_x, pupil_x) = if self.player.facing_right {
                (psx + self.player.size.x * 0.55, psx + self.player.size.x * 0.65)
            } else {
                (psx + self.player.size.x * 0.25, psx + self.player.size.x * 0.15)
            };
            let eye_y = psy + self.player.size.y * 0.25;

            draw_circle(eye_x, eye_y, 4.0, WHITE);
            draw_circle(pupil_x, eye_y, 2.0, BLACK);
        }

        // ── HUD ─────────────────────────────────────────────────────────
        let hud_text = format!("x: {:.0}  y: {:.0}  grounded: {}", self.player.pos.x, self.player.pos.y, self.player.grounded);
        draw_text(&hud_text, 12.0, 28.0, 20.0, Color::from_hex(0xaaaaaa));
        draw_text("Arrow keys / WASD to move, Space to jump  |  R to reset", 12.0, screen_height() - 12.0, 16.0, Color::from_hex(0x666666));

        // ── Death overlay ───────────────────────────────────────────────
        if self.player.dead {
            // Semi-transparent overlay
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                           Color::from_rgba(0, 0, 0, 180));

            let title = "YOU IMPALED YOURSELF";
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

// ── Entry point ──────────────────────────────────────────────────────────────

#[macroquad::main("Platformer")]
async fn main() {
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(0.05);

        // ── Update ──────────────────────────────────────────────────────
        if !game.player.dead {
            game.update_player(dt);
        } else if is_key_pressed(KeyCode::Space) {
            game = Game::new();
        }

        // Manual reset
        if is_key_pressed(KeyCode::R) {
            game = Game::new();
        }

        // ── Draw ────────────────────────────────────────────────────────
        game.draw();

        next_frame().await
    }
}
