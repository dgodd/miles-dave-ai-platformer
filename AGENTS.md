# AGENTS.md — Platformer

## Project overview

A 2D side-scrolling platformer written in Rust using the `macroquad` game framework. The player controls a dog character who must navigate platforms, avoid spike pits, and evade crawling baby enemies.

## Tech

| Aspect | Choice |
|---|---|
| Language | Rust (edition 2024) |
| Framework | macroquad 0.4 |
| Rendering | Immediate-mode shapes (no assets, no texture files) |
| Sprites | Drawn with macroquad primitives (rectangles, circles, triangles, lines) |
| Physics | Simple Euler integration, AABB collision |

## Code structure (`src/main.rs`)

The entire game is a single file. The major sections are:

### Constants
- **Physics constants**: `GRAVITY`, `JUMP_VELOCITY`, `MOVE_SPEED`, `PLAYER_WIDTH`, `PLAYER_HEIGHT`
- **Gameplay constants**: `COYOTE_FRAMES`, `JUMP_BUFFER_FRAMES`, `BABY_SPEED`
- **Visual constants**: `DOG_SCALE`, and colour constants prefixed by entity (`FUR_*`, `BABY_*`, etc.)

### Structs
- **`Player`** — position, velocity, animation state (`walk_time`, `dead`, `grounded`), coyote + jump buffer counters
- **`Platform`** — positioned rectangle with `rect()` helper
- **`Spike`** — positioned rectangle with `rect()` helper and `draw()` method
- **`Baby`** — position, velocity, patrol bounds (`min_x`, `max_x`), `crawl_time` for animation

### Game state (`struct Game`)
- Holds `player`, `platforms: Vec<Platform>`, `spikes: Vec<Spike>`, `babies: Vec<Baby>`
- **`new()`** — constructs the level layout (platform positions, spike pits, baby patrol zones)
- **`reset()`** — recreates the entire game state
- **`update_player(dt)`** — input handling, physics integration, platform collision, spike/baby collision
- **`camera_offset()`** — returns a `Vec2` camera offset centred on the player
- **`draw()`** — renders everything (background, platforms, spikes, babies, player, HUD, death overlay)

### Sprite drawing functions
- **`draw_dog_sprite(cx, cy, &Player)`** — draws the animated dog at a screen position
- **`draw_front_leg(x, y, grounded, s)`** — dog front leg primitive
- **`draw_back_leg(x, y, grounded, s)`** — dog back leg primitive
- **`draw_baby_sprite(cx, cy, &Baby)`** — draws a crawling baby at a screen position

### Entry point
- `#[macroquad::main("Platformer")]` async main loop calling `update_player`, `baby.update()`, and `draw()` each frame

## Conventions

### Coordinate system
- **`bx, by`** = absolute screen coordinates of the dog's body centre
- **`ox(dx)`** = `bx + dx * flip * DOG_SCALE` — converts a local offset to absolute screen x, mirroring when the dog faces left
- **`hx_off(dx)`** = `hx + dx * flip * DOG_SCALE` — same but relative to the head centre
- `flip` is `1.0` when facing right, `-1.0` when facing left
- **Never** nest flip-aware helpers: `ox()` and `hx_off()` expect *local* offsets, not absolute positions

### DOG_SCALE
- All positional offsets, radii, sizes, and line thicknesses in the dog drawing code are multiplied by `DOG_SCALE` (currently 1.3). The `s` parameter (passed to leg functions) is this same scale factor.

### Animation
- **Dog walking**: `walk_time` accumulated when grounded + moving, drives leg phase (8x speed), body bob (10x), tail wag (12x), ear flop (8x)
- **Dog idle**: slow breathing bob (2.5x), slow tail wag (3x), occasional tongue
- **Dog jumping**: legs tucked (PI phase), tail straight back (-0.8), ears back (-0.15), tongue out
- **Baby crawl**: `crawl_time` drives arm/leg rock via `(t * 7.0).sin()`

### Level layout
- Platforms and spikes are defined in `Game::new()` with absolute world coordinates
- `floor_y = screen_height() - 40.0` is the ground level
- Babies are placed with centre x, floor y, and patrol range (total width they walk)

## Adding new entities

1. Define the struct with position, size, and any state fields
2. Add a `rect()` method for collision
3. Add an `update(dt)` method for movement/AI
4. Add a `draw_baby_sprite`-style drawing function
5. Add `Vec<NewThing>` to `Game`
6. Create instances in `Game::new()`
7. Call `update()` in the main loop (inside the `!player.dead` block)
8. Call draw in `Game::draw()`
9. Add collision checks in `Game::update_player()`

## Git

- Commit after every task. Descriptive commit messages explaining what changed and why.
