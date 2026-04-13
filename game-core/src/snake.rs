use std::collections::VecDeque;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    fn opposite(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }

    /// Returns (col_delta, row_delta)
    fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Right => (1, 0),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
        }
    }
}

fn dir_from_u8(v: u8) -> Option<Dir> {
    match v {
        0 => Some(Dir::Up),
        1 => Some(Dir::Right),
        2 => Some(Dir::Down),
        3 => Some(Dir::Left),
        _ => None,
    }
}

fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[wasm_bindgen]
pub struct SnakeGame {
    cols: u32,
    rows: u32,
    /// Front = head, back = tail
    body: VecDeque<(u32, u32)>,
    dir: Dir,
    queued_dir: Dir,
    food: (u32, u32),
    score: u32,
    over: bool,
    rng: u64,
}

#[wasm_bindgen]
impl SnakeGame {
    #[wasm_bindgen(constructor)]
    pub fn new(cols: u32, rows: u32) -> SnakeGame {
        let mid_col = cols / 2;
        let mid_row = rows / 2;

        // Start with a 3-cell snake facing right: head at center, body going left
        let mut body = VecDeque::new();
        body.push_back((mid_col, mid_row));
        body.push_back((mid_col - 1, mid_row));
        body.push_back((mid_col - 2, mid_row));

        let mut rng: u64 = 12345678901;
        let food = place_food(&body, cols, rows, &mut rng);

        SnakeGame {
            cols,
            rows,
            body,
            dir: Dir::Right,
            queued_dir: Dir::Right,
            food,
            score: 0,
            over: false,
            rng,
        }
    }

    /// Advance one game tick. Returns false when the game is over.
    pub fn step(&mut self) -> bool {
        if self.over {
            return false;
        }

        // Apply queued direction (ignore 180° reversals)
        if self.queued_dir != self.dir.opposite() {
            self.dir = self.queued_dir;
        }

        let (dc, dr) = self.dir.delta();
        let (head_col, head_row) = *self.body.front().unwrap();

        let new_col = head_col as i32 + dc;
        let new_row = head_row as i32 + dr;

        // Bounds check
        if new_col < 0 || new_col >= self.cols as i32 || new_row < 0 || new_row >= self.rows as i32
        {
            self.over = true;
            return false;
        }

        let new_head = (new_col as u32, new_row as u32);

        // Self-collision check (exclude tail since it will move away, unless growing)
        let eating = new_head == self.food;
        let tail = if !eating { self.body.back().copied() } else { None };

        for &seg in &self.body {
            if seg == new_head && Some(seg) != tail {
                self.over = true;
                return false;
            }
        }

        // Move
        self.body.push_front(new_head);
        if eating {
            self.score += 1;
            self.food = place_food(&self.body, self.cols, self.rows, &mut self.rng);
        } else {
            self.body.pop_back();
        }

        true
    }

    /// Queue a direction change. 0=Up 1=Right 2=Down 3=Left.
    /// Ignored if invalid or if it would immediately reverse the snake.
    pub fn set_direction(&mut self, dir: u8) {
        if let Some(d) = dir_from_u8(dir) {
            self.queued_dir = d;
        }
    }

    /// Returns a flat grid as bytes: 0=empty, 1=body, 2=head, 3=food.
    /// Length = cols * rows, indexed as [row * cols + col].
    pub fn cells(&self) -> Vec<u8> {
        let mut grid = vec![0u8; (self.cols * self.rows) as usize];

        grid[(self.food.1 * self.cols + self.food.0) as usize] = 3;

        let mut iter = self.body.iter();
        if let Some(&(hc, hr)) = iter.next() {
            // Body segments (skip head)
            for &(c, r) in iter {
                grid[(r * self.cols + c) as usize] = 1;
            }
            // Head written last so it takes priority
            grid[(hr * self.cols + hc) as usize] = 2;
        }

        grid
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn is_over(&self) -> bool {
        self.over
    }
}

fn place_food(body: &VecDeque<(u32, u32)>, cols: u32, rows: u32, rng: &mut u64) -> (u32, u32) {
    let total = cols * rows;
    loop {
        let idx = (xorshift64(rng) % total as u64) as u32;
        let candidate = (idx % cols, idx / cols);
        if !body.contains(&candidate) {
            return candidate;
        }
    }
}
