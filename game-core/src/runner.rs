use wasm_bindgen::prelude::*;

const WORLD_W: f32 = 640.0;
const WORLD_H: f32 = 200.0;
const GROUND_Y: f32 = 165.0; // y of the ground line; things rest with bottom here

const PLAYER_X: f32 = 60.0;
const PLAYER_W: f32 = 26.0;
const PLAYER_H: f32 = 46.0;
const DUCK_H: f32 = 26.0; // ducking halves height; width stays

const GRAVITY: f32 = 0.62;
const JUMP_VELOCITY: f32 = -11.6; // ~37 frames airborne, ~108px peak
const FAST_FALL_GRAVITY: f32 = 1.5; // ducking mid-air drops you fast

const BASE_SPEED: f32 = 5.0;
const MAX_SPEED: f32 = 12.5;
const SPEED_RAMP: f32 = 0.0009; // px/frame gained per px travelled

const BIRD_MIN_SCORE: u32 = 200;

// Belly height (above ground) for each bird kind. The low bird must sit
// strictly between DUCK_H and PLAYER_H: a standing player's head (up to
// PLAYER_H) reaches into it, but a ducking player's head (only up to
// DUCK_H) passes underneath — that's what makes ducking mandatory rather
// than optional. The high bird sits above PLAYER_H so it's always clear,
// standing or ducking, and needs no input.
const LOW_BIRD_BELLY_H: f32 = 34.0;
const HIGH_BIRD_BELLY_H: f32 = 80.0;

fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// Frames during which the top of an obstacle of height `h` is below the
/// player's lowest point during a jump arc (i.e. the player's feet clear it),
/// derived from the jump parabola `y(t) = v0*t + 0.5*g*t^2`.
fn frames_above(h: f32) -> f32 {
    // Player clears the obstacle while airborne AND below the height needed
    // to clear it. Solve for the two roots of the parabola crossing
    // `-h` (measuring up from the ground) and take the spread between them.
    let v0 = -JUMP_VELOCITY; // positive "launch" speed
    let disc = v0 * v0 - 2.0 * GRAVITY * h;
    if disc <= 0.0 {
        // Obstacle taller than jump apex: never clearable (shouldn't happen
        // given our obstacle table, but guard against div weirdness).
        return 0.0;
    }
    let sqrt_disc = disc.sqrt();
    let t1 = (v0 - sqrt_disc) / GRAVITY;
    let t2 = (v0 + sqrt_disc) / GRAVITY;
    t2 - t1
}

/// Obstacle kind codes, exposed to the frontend for choosing a sprite.
/// 0=small cactus 1=cactus cluster 2=tall cactus 3=low bird 4=high bird
#[derive(Clone, Copy)]
struct Obstacle {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    kind: u8,
}

#[wasm_bindgen]
pub struct RunnerGame {
    distance: f32,
    speed: f32,
    player_y: f32,
    velocity: f32,
    ducking: bool,
    airborne: bool,
    obstacles: Vec<Obstacle>,
    rng: u64,
    score: u32,
    over: bool,
}

#[wasm_bindgen]
impl RunnerGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RunnerGame {
        let mut game = RunnerGame {
            distance: 0.0,
            speed: BASE_SPEED,
            player_y: GROUND_Y - PLAYER_H,
            velocity: 0.0,
            ducking: false,
            airborne: false,
            obstacles: Vec::new(),
            rng: 55512340987,
            score: 0,
            over: false,
        };
        game.spawn_obstacle();
        game
    }

    /// Advance one frame. Returns false when the game is over.
    pub fn step(&mut self) -> bool {
        if self.over {
            return false;
        }

        self.distance += self.speed;
        self.score = (self.distance / 10.0) as u32;
        self.speed = (BASE_SPEED + self.distance * SPEED_RAMP).min(MAX_SPEED);

        // Integrate player vertical motion.
        if self.airborne {
            let gravity = if self.ducking {
                FAST_FALL_GRAVITY
            } else {
                GRAVITY
            };
            self.velocity += gravity;
            self.player_y += self.velocity;
            let ground_top = GROUND_Y - self.player_height();
            if self.player_y >= ground_top {
                self.player_y = ground_top;
                self.velocity = 0.0;
                self.airborne = false;
            }
        } else {
            self.player_y = GROUND_Y - self.player_height();
        }

        // Scroll obstacles and prune those that have left the world.
        for o in self.obstacles.iter_mut() {
            o.x -= self.speed;
        }
        self.prune_offscreen();

        // Spawn more if the gap to the rightmost obstacle has opened up.
        self.maybe_spawn();

        // Collision check.
        let (px, py, pw, ph) = self.player_rect();
        for o in &self.obstacles {
            if px < o.x + o.w && px + pw > o.x && py < o.y + o.h && py + ph > o.y {
                self.over = true;
                return false;
            }
        }

        true
    }

    /// Start a jump; no-op if already airborne or the game is over.
    pub fn jump(&mut self) {
        if self.over || self.airborne {
            return;
        }
        self.airborne = true;
        self.velocity = JUMP_VELOCITY;
    }

    /// Set the held-duck state. On the ground this shrinks the hitbox; in
    /// the air it fast-falls.
    pub fn set_ducking(&mut self, on: bool) {
        self.ducking = on;
    }

    /// Flat obstacle list, 5 floats each: [x, y, w, h, kind, ...].
    pub fn obstacles(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.obstacles.len() * 5);
        for o in &self.obstacles {
            out.push(o.x);
            out.push(o.y);
            out.push(o.w);
            out.push(o.h);
            out.push(o.kind as f32);
        }
        out
    }

    pub fn player_x(&self) -> f32 {
        PLAYER_X
    }

    pub fn player_y(&self) -> f32 {
        self.player_y
    }

    pub fn player_w(&self) -> f32 {
        PLAYER_W
    }

    pub fn player_h(&self) -> f32 {
        self.player_height()
    }

    pub fn is_ducking(&self) -> bool {
        self.ducking
    }

    pub fn is_airborne(&self) -> bool {
        self.airborne
    }

    pub fn ground_y(&self) -> f32 {
        GROUND_Y
    }

    pub fn world_w(&self) -> f32 {
        WORLD_W
    }

    pub fn world_h(&self) -> f32 {
        WORLD_H
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn is_over(&self) -> bool {
        self.over
    }
}

impl RunnerGame {
    fn player_height(&self) -> f32 {
        if self.ducking && !self.airborne {
            DUCK_H
        } else {
            PLAYER_H
        }
    }

    fn player_rect(&self) -> (f32, f32, f32, f32) {
        (PLAYER_X, self.player_y, PLAYER_W, self.player_height())
    }

    /// Minimum gap (px) between obstacles at the current speed so a jump
    /// fully lands before the next obstacle arrives.
    fn min_gap(&self) -> f32 {
        self.speed * 42.0
    }

    fn rightmost_edge(&self) -> f32 {
        self.obstacles
            .iter()
            .map(|o| o.x + o.w)
            .fold(f32::MIN, f32::max)
    }

    fn maybe_spawn(&mut self) {
        // An empty list means nothing is in flight to block a spawn at all —
        // spawn immediately rather than measuring a gap against a phantom
        // obstacle. (Without this, once the last obstacle scrolls offscreen
        // faster than the next spawn's gap threshold is reached — which
        // becomes likely as speed increases — obstacles would stop spawning
        // permanently: the gap would be measured as `WORLD_W - WORLD_W == 0`,
        // which can never satisfy the always-positive required gap.)
        if self.obstacles.is_empty() {
            self.spawn_obstacle();
            return;
        }
        let rightmost = self.rightmost_edge();
        let jitter = (xorshift64(&mut self.rng) % 1000) as f32 / 1000.0 * self.speed * 45.0;
        let required_gap = self.min_gap() + jitter;
        if WORLD_W - rightmost >= required_gap {
            self.spawn_obstacle();
        }
    }

    fn prune_offscreen(&mut self) {
        self.obstacles.retain(|o| o.x + o.w >= 0.0);
    }

    fn spawn_obstacle(&mut self) {
        let birds_unlocked = self.score >= BIRD_MIN_SCORE;
        let kind_count: u64 = if birds_unlocked { 5 } else { 3 };
        let kind = (xorshift64(&mut self.rng) % kind_count) as u8;

        let (base_w, h, y, kind) = match kind {
            0 => (17.0, 36.0, GROUND_Y - 36.0, 0u8),
            1 => (34.0, 36.0, GROUND_Y - 36.0, 1u8),
            2 => (20.0, 50.0, GROUND_Y - 50.0, 2u8),
            3 => (34.0, 24.0, GROUND_Y - LOW_BIRD_BELLY_H - 24.0, 3u8),
            _ => (34.0, 24.0, GROUND_Y - HIGH_BIRD_BELLY_H - 24.0, 4u8),
        };

        // Cluster width varies 2-3 cacti wide (34-51px); widen only within
        // the bound `frames_above` guarantees is still clearable at MAX_SPEED.
        let mut w = base_w;
        if kind == 1 {
            let extra = (xorshift64(&mut self.rng) % 2) as f32; // 0 or 1
            w = base_w + extra * 17.0;
        }

        // Clearability guard: shrink width until a jump can clear it even at
        // the current speed. This can only ever narrow ground obstacles;
        // birds (kind 3/4) aren't jumped over so they're exempt.
        if kind <= 2 {
            let max_w = (frames_above(h) * self.speed - PLAYER_W).max(base_w.min(17.0));
            if w > max_w {
                w = max_w;
            }
        }

        self.obstacles.push(Obstacle {
            x: WORLD_W,
            y,
            w,
            h,
            kind,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every ground obstacle width the spawner can produce must stay
    /// clearable by a jump at every speed from BASE_SPEED to MAX_SPEED.
    #[test]
    fn ground_obstacles_are_always_clearable() {
        let heights = [36.0f32, 50.0];
        let mut speed = BASE_SPEED;
        while speed <= MAX_SPEED {
            for &h in &heights {
                let max_w = frames_above(h) * speed - PLAYER_W;
                assert!(
                    max_w >= 17.0,
                    "at speed {speed}, height {h}: max clearable width {max_w} is below the smallest obstacle"
                );
            }
            speed += 0.1;
        }
    }

    /// Regression test: once every in-flight obstacle scrolls off (which
    /// becomes likely at high speed, since the time to leave the world
    /// shrinks while the spawn gap threshold in frames does not), the next
    /// `maybe_spawn` must refill the list rather than measuring a gap
    /// against a phantom obstacle and stalling forever.
    #[test]
    fn obstacles_never_stop_spawning_once_the_list_empties() {
        let mut game = RunnerGame::new();
        game.speed = MAX_SPEED;
        game.obstacles.clear();

        for _ in 0..500 {
            for o in game.obstacles.iter_mut() {
                o.x -= game.speed;
            }
            game.prune_offscreen();
            game.maybe_spawn();
            assert!(
                !game.obstacles.is_empty(),
                "obstacles permanently stopped spawning after the list emptied"
            );
        }
    }

    /// The low bird must fly low enough that a standing player's head
    /// reaches it (forcing a duck) but high enough that a ducking player's
    /// (shorter) head passes underneath. The high bird must fly above a
    /// standing player entirely, so it never requires any input.
    #[test]
    fn low_bird_requires_duck_but_high_bird_never_hits() {
        assert!(
            LOW_BIRD_BELLY_H > DUCK_H,
            "low bird flies at or below duck height and could never be hit ducking"
        );
        assert!(
            LOW_BIRD_BELLY_H < PLAYER_H,
            "low bird flies at or above standing height and would never force a duck"
        );
        assert!(
            HIGH_BIRD_BELLY_H > PLAYER_H,
            "high bird flies low enough to hit a standing player"
        );
    }

    #[test]
    fn game_runs_without_immediate_collision() {
        // The first obstacle spawns at the far edge of the world, so it
        // cannot possibly reach the player faster than (world width - player
        // right edge) / base speed frames. Stepping for fewer frames than
        // that with no input should never collide.
        let mut game = RunnerGame::new();
        for _ in 0..90 {
            if !game.step() {
                panic!("game ended within the first 90 frames with no player input");
            }
        }
    }
}
