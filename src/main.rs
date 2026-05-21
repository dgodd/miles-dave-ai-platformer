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

// ── Game state ───────────────────────────────────────────────────────────────

struct Game {
    player: Player,
    platforms: Vec<Platform>,
}

impl Game {
    fn new() -> Self {
        // Build a little tutorial-style level
        let platforms = vec![
            // Ground
            Platform { pos: vec2(0.0, screen_height() - 40.0), size: vec2(800.0, 40.0) },
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
            Platform { pos: vec2(1800.0, screen_height() - 40.0), size: vec2(400.0, 40.0) },
        ];

        let start_x = 80.0;
        let start_y = screen_height() - 40.0 - PLAYER_HEIGHT;

        Self {
            player: Player::new(start_x, start_y),
            platforms,
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

        // ── Collision resolution ────────────────────────────────────────
        // Only check the current platform set (small, so O(n) is fine)
        self.player.grounded = false;

        for plat in &self.platforms {
            if let Some(collision) = self.player.rect().intersect(plat.rect()) {
                // Determine the dominant overlap axis
                let overlap_x = collision.w;
                let overlap_y = collision.h;

                // Prevent "sticking" to walls — only resolve the smallest
                // overlap direction when it's clear-cut, and prefer vertical
                // landing when velocity is downward.
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

        // ── Clamp to screen boundaries horizontally ─────────────────────
        // (Don't let the player walk off the left edge of the world)
        if self.player.pos.x < 0.0 {
            self.player.pos.x = 0.0;
            self.player.vel.x = 0.0;
        }
    }

    /// Compute the camera offset so the player is roughly centred.
    fn camera_offset(&self) -> Vec2 {
        let target_x = self.player.pos.x + self.player.size.x / 2.0 - screen_width() / 2.0;
        let target_y = self.player.pos.y + self.player.size.y / 2.0 - screen_height() / 2.0;

        // Clamp so we don't show empty space to the left of the level
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
            // Shadow / border
            draw_rectangle(sx, sy, plat.size.x, plat.size.y, Color::from_hex(0x16213e));
            // Inner fill
            draw_rectangle(sx + 2.0, sy + 2.0, plat.size.x - 4.0, plat.size.y - 4.0, Color::from_hex(0x0f3460));
            // Top highlight
            draw_line(sx + 4.0, sy + 1.0, sx + plat.size.x - 4.0, sy + 1.0, 2.0, Color::from_hex(0x533483));
        }

        // ── Player ──────────────────────────────────────────────────────
        let psx = self.player.pos.x - cam.x;
        let psy = self.player.pos.y - cam.y;

        // Body
        let body_color = if self.player.grounded {
            Color::from_hex(0xe94560)
        } else {
            Color::from_hex(0xf5a623)
        };
        draw_rectangle(psx, psy, self.player.size.x, self.player.size.y, body_color);
        // A lighter inner rectangle for depth
        draw_rectangle(psx + 3.0, psy + 3.0, self.player.size.x - 6.0, self.player.size.y - 6.0,
                       Color::from_rgba(255, 255, 255, 30));

        // Eyes (indicate facing direction)
        let (eye_x, pupil_x) = if self.player.facing_right {
            (psx + self.player.size.x * 0.55, psx + self.player.size.x * 0.65)
        } else {
            (psx + self.player.size.x * 0.25, psx + self.player.size.x * 0.15)
        };
        let eye_y = psy + self.player.size.y * 0.25;

        // White of eye
        draw_circle(eye_x, eye_y, 4.0, WHITE);
        // Pupil
        draw_circle(pupil_x, eye_y, 2.0, BLACK);

        // ── HUD ─────────────────────────────────────────────────────────
        let hud_text = format!("x: {:.0}  y: {:.0}  grounded: {}", self.player.pos.x, self.player.pos.y, self.player.grounded);
        draw_text(&hud_text, 12.0, 28.0, 20.0, Color::from_hex(0xaaaaaa));
        draw_text("Arrow keys / WASD to move, Space to jump  |  R to reset", 12.0, screen_height() - 12.0, 16.0, Color::from_hex(0x666666));
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[macroquad::main("Platformer")]
async fn main() {
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(0.05); // cap to avoid physics blowups

        // ── Update ──────────────────────────────────────────────────────
        game.update_player(dt);

        // Reset with R
        if is_key_pressed(KeyCode::R) {
            game = Game::new();
        }

        // ── Draw ────────────────────────────────────────────────────────
        game.draw();

        next_frame().await
    }
}
