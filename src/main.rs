use macroquad::prelude::*;

// ── Constants ────────────────────────────────────────────────────────────────

const GRAVITY: f32 = 980.0;
const JUMP_VELOCITY: f32 = -520.0;
const MOVE_SPEED: f32 = 280.0;
const PLAYER_WIDTH: f32 = 28.0;
const PLAYER_HEIGHT: f32 = 36.0;

/// Drawn sprite size (can be larger than the collision box).
const SPRITE_WIDTH: f32 = 48.0;
const SPRITE_HEIGHT: f32 = 48.0;

const COYOTE_FRAMES: i32 = 6;
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
    /// Accumulated time for walk-cycle animation.
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
            return; // off-screen culling
        }

        let tooth_count = (self.width / SPIKE_TOOTH_WIDTH).ceil() as usize;
        let spacing = self.width / tooth_count as f32;

        draw_rectangle(sx, sy, self.width, self.height, Color::from_hex(0x0d0d1a));

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

// ── Game state ───────────────────────────────────────────────────────────────

struct Game {
    player: Player,
    platforms: Vec<Platform>,
    spikes: Vec<Spike>,
    dog_texture: Texture2D,
}

impl Game {
    fn new(dog_texture: Texture2D) -> Self {
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

        let start_x = 80.0;
        let start_y = floor_y - PLAYER_HEIGHT;

        Self {
            player: Player::new(start_x, start_y),
            platforms,
            spikes,
            dog_texture,
        }
    }

    /// Reset the game world while keeping the loaded texture.
    fn reset(&mut self) {
        *self = Self::new(self.dog_texture.clone());
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

        // Walk animation timer
        if self.player.grounded && move_x != 0.0 {
            self.player.walk_time += dt;
        } else if self.player.grounded {
            self.player.walk_time = 0.0;
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

        // ── Clamp to screen boundaries ──────────────────────────────────
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

        // ── Player (animated dog sprite) ────────────────────────────────
        if !self.player.dead {
            self.draw_dog_sprite(cam);
        }

        // ── HUD ─────────────────────────────────────────────────────────
        let hud_text = format!("x: {:.0}  y: {:.0}  grounded: {}", self.player.pos.x, self.player.pos.y, self.player.grounded);
        draw_text(&hud_text, 12.0, 28.0, 20.0, Color::from_hex(0xaaaaaa));
        draw_text("Arrow keys / WASD to move, Space to jump  |  R to reset", 12.0, screen_height() - 12.0, 16.0, Color::from_hex(0x666666));

        // ── Death overlay ───────────────────────────────────────────────
        if self.player.dead {
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

    /// Draw the dog with walk-cycle bobbing and jump squash/stretch.
    fn draw_dog_sprite(&self, cam: Vec2) {
        let p = &self.player;

        // Centre the sprite on the collision box
        let draw_x = p.pos.x + (p.size.x - SPRITE_WIDTH) / 2.0 - cam.x;
        let draw_y = p.pos.y + (p.size.y - SPRITE_HEIGHT) / 2.0 - cam.y;

        // ── Compute animation transforms ────────────────────────────────
        let (scale_x, scale_y, offset_y);

        if p.grounded {
            if p.vel.x != 0.0 {
                // Walking: vertical bob with gentle squish/stretch
                let cycle = (p.walk_time * 10.0).sin();
                // Bob up/down by 2px
                offset_y = cycle * 2.0;
                // Subtle squash/stretch (0.92 – 1.08)
                scale_x = 1.0 + cycle * 0.06;
                scale_y = 1.0 - cycle * 0.06;
            } else {
                // Idle: very subtle breathing
                let breath = (p.walk_time * 3.0).sin() * 0.02;
                offset_y = 0.0;
                scale_x = 1.0 + breath;
                scale_y = 1.0 - breath;
            }
        } else {
            // In air: stretch vertically, squish horizontally
            let t = (p.vel.y / 800.0).clamp(-0.15, 0.15);
            offset_y = 0.0;
            scale_x = 1.0 + t;   // widen when falling fast
            scale_y = 1.0 - t;   // lengthen when falling fast
        }

        // ── Flip based on facing direction ──────────────────────────────
        let flip = if p.facing_right { 1.0 } else { -1.0 };

        // ── Draw the dog ────────────────────────────────────────────────
        let half_w = SPRITE_WIDTH / 2.0;
        let half_h = SPRITE_HEIGHT / 2.0;
        let cx = draw_x + half_w;
        let cy = draw_y + half_h + offset_y;

        draw_texture_ex(
            &self.dog_texture,
            cx - half_w * scale_x,
            cy - half_h * scale_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SPRITE_WIDTH * scale_x, SPRITE_HEIGHT * scale_y)),
                flip_x: flip < 0.0,
                ..Default::default()
            },
        );
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[macroquad::main("Platformer")]
async fn main() {
    let dog_texture = load_texture("assets/Dog.png").await.unwrap();
    dog_texture.set_filter(FilterMode::Nearest);

    let mut game = Game::new(dog_texture);

    loop {
        let dt = get_frame_time().min(0.05);

        // ── Update ──────────────────────────────────────────────────────
        if !game.player.dead {
            game.update_player(dt);
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
