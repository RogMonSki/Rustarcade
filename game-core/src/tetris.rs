use std::collections::VecDeque;
use wasm_bindgen::prelude::*;

const COLS: usize = 10;
const ROWS: usize = 20;

fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// Bounding-box size for a piece kind. 0=I 1=O 2=T 3=S 4=Z 5=J 6=L.
fn box_size(kind: u8) -> i8 {
    match kind {
        0 => 4,
        1 => 2,
        _ => 3,
    }
}

/// Spawn-orientation cells (row, col) within the piece's bounding box.
fn base_shape(kind: u8) -> [(i8, i8); 4] {
    match kind {
        0 => [(1, 0), (1, 1), (1, 2), (1, 3)], // I
        1 => [(0, 0), (0, 1), (1, 0), (1, 1)], // O
        2 => [(0, 1), (1, 0), (1, 1), (1, 2)], // T
        3 => [(0, 1), (0, 2), (1, 0), (1, 1)], // S
        4 => [(0, 0), (0, 1), (1, 1), (1, 2)], // Z
        5 => [(0, 0), (1, 0), (1, 1), (1, 2)], // J
        6 => [(0, 2), (1, 0), (1, 1), (1, 2)], // L
        _ => [(0, 0), (0, 0), (0, 0), (0, 0)],
    }
}

/// Rotate cells 90° clockwise within an n×n box, `times` times.
fn rotate_cells(cells: [(i8, i8); 4], n: i8, times: u8) -> [(i8, i8); 4] {
    let mut c = cells;
    for _ in 0..(times % 4) {
        c = c.map(|(r, col)| (col, n - 1 - r));
    }
    c
}

fn piece_cells(kind: u8, rotation: u8) -> [(i8, i8); 4] {
    rotate_cells(base_shape(kind), box_size(kind), rotation)
}

fn line_score(lines: u32) -> u32 {
    match lines {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    }
}

#[derive(Clone, Copy)]
struct Piece {
    kind: u8,
    rotation: u8,
    origin_row: i32,
    origin_col: i32,
}

impl Piece {
    fn spawn(kind: u8) -> Piece {
        let b = box_size(kind) as i32;
        Piece {
            kind,
            rotation: 0,
            origin_row: 0,
            origin_col: (COLS as i32 - b) / 2,
        }
    }
}

#[wasm_bindgen]
pub struct TetrisGame {
    grid: Vec<u8>,
    piece: Piece,
    bag: VecDeque<u8>,
    rng: u64,
    score: u32,
    lines: u32,
    level: u32,
    over: bool,
}

#[wasm_bindgen]
impl TetrisGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TetrisGame {
        let mut game = TetrisGame {
            grid: vec![0u8; COLS * ROWS],
            piece: Piece::spawn(0),
            bag: VecDeque::new(),
            rng: 98765432101,
            score: 0,
            lines: 0,
            level: 1,
            over: false,
        };
        game.ensure_bag();
        let kind = game.bag.pop_front().unwrap();
        game.ensure_bag();
        game.piece = Piece::spawn(kind);
        game
    }

    /// Advance one gravity tick. Returns false when the game is over.
    pub fn step(&mut self) -> bool {
        if self.over {
            return false;
        }
        if !self.try_move_down() {
            self.lock_piece();
        }
        !self.over
    }

    pub fn move_left(&mut self) {
        if self.over {
            return;
        }
        let (row, col) = (self.piece.origin_row, self.piece.origin_col - 1);
        if !self.collision(row, col, self.piece.rotation) {
            self.piece.origin_col = col;
        }
    }

    pub fn move_right(&mut self) {
        if self.over {
            return;
        }
        let (row, col) = (self.piece.origin_row, self.piece.origin_col + 1);
        if !self.collision(row, col, self.piece.rotation) {
            self.piece.origin_col = col;
        }
    }

    pub fn rotate(&mut self) {
        if self.over {
            return;
        }
        let new_rotation = (self.piece.rotation + 1) % 4;
        // Try the rotated position in place, then a small set of wall-kick offsets.
        const KICKS: [(i32, i32); 6] = [(0, 0), (-1, 0), (1, 0), (0, -1), (-2, 0), (2, 0)];
        for (dc, dr) in KICKS {
            let row = self.piece.origin_row + dr;
            let col = self.piece.origin_col + dc;
            if !self.collision(row, col, new_rotation) {
                self.piece.origin_row = row;
                self.piece.origin_col = col;
                self.piece.rotation = new_rotation;
                return;
            }
        }
    }

    /// Move the active piece down one row. +1 score if it moves; locks
    /// immediately (like a gravity tick) if it can't.
    pub fn soft_drop(&mut self) {
        if self.over {
            return;
        }
        if self.try_move_down() {
            self.score += 1;
        } else {
            self.lock_piece();
        }
    }

    /// Drop the active piece straight to the floor and lock it.
    pub fn hard_drop(&mut self) {
        if self.over {
            return;
        }
        let mut rows = 0u32;
        while self.try_move_down() {
            rows += 1;
        }
        self.score += rows * 2;
        self.lock_piece();
    }

    /// Flat grid as bytes, `row * 10 + col`. 0=empty, 1..=7=locked/active
    /// mino (colored by piece kind), 8..=14=ghost outline for that kind
    /// (piece kind's code + 7).
    pub fn cells(&self) -> Vec<u8> {
        let mut grid = self.grid.clone();

        let ghost_row = self.ghost_origin_row();
        for (r, c) in piece_cells(self.piece.kind, self.piece.rotation) {
            let row = ghost_row + r as i32;
            let col = self.piece.origin_col + c as i32;
            if row >= 0 && (row as usize) < ROWS && col >= 0 && (col as usize) < COLS {
                grid[row as usize * COLS + col as usize] = self.piece.kind + 1 + 7;
            }
        }

        for (r, c) in piece_cells(self.piece.kind, self.piece.rotation) {
            let row = self.piece.origin_row + r as i32;
            let col = self.piece.origin_col + c as i32;
            if row >= 0 && (row as usize) < ROWS && col >= 0 && (col as usize) < COLS {
                grid[row as usize * COLS + col as usize] = self.piece.kind + 1;
            }
        }

        grid
    }

    /// Flat 4x4 preview grid for the upcoming piece, same color coding as `cells()`.
    pub fn next_cells(&self) -> Vec<u8> {
        let mut grid = vec![0u8; 16];
        let kind = *self.bag.front().unwrap();
        let b = box_size(kind);
        let off = (4 - b) / 2;
        for (r, c) in piece_cells(kind, 0) {
            let rr = (r + off) as usize;
            let cc = (c + off) as usize;
            grid[rr * 4 + cc] = kind + 1;
        }
        grid
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn lines(&self) -> u32 {
        self.lines
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn is_over(&self) -> bool {
        self.over
    }

    /// Frames between automatic gravity ticks at the current level; the
    /// frontend drives `step()` at this cadence.
    pub fn gravity_frames(&self) -> u32 {
        (30i32 - (self.level as i32 - 1) * 2).max(2) as u32
    }
}

impl TetrisGame {
    fn ensure_bag(&mut self) {
        while self.bag.len() < 2 {
            let mut ids: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
            for i in (1..7).rev() {
                let j = (xorshift64(&mut self.rng) % (i as u64 + 1)) as usize;
                ids.swap(i, j);
            }
            self.bag.extend(ids);
        }
    }

    fn collision(&self, origin_row: i32, origin_col: i32, rotation: u8) -> bool {
        for (r, c) in piece_cells(self.piece.kind, rotation) {
            let row = origin_row + r as i32;
            let col = origin_col + c as i32;
            if col < 0 || col >= COLS as i32 || row >= ROWS as i32 {
                return true;
            }
            if row < 0 {
                continue;
            }
            if self.grid[row as usize * COLS + col as usize] != 0 {
                return true;
            }
        }
        false
    }

    fn try_move_down(&mut self) -> bool {
        let row = self.piece.origin_row + 1;
        if self.collision(row, self.piece.origin_col, self.piece.rotation) {
            false
        } else {
            self.piece.origin_row = row;
            true
        }
    }

    fn ghost_origin_row(&self) -> i32 {
        let mut row = self.piece.origin_row;
        while !self.collision(row + 1, self.piece.origin_col, self.piece.rotation) {
            row += 1;
        }
        row
    }

    fn lock_piece(&mut self) {
        for (r, c) in piece_cells(self.piece.kind, self.piece.rotation) {
            let row = self.piece.origin_row + r as i32;
            let col = self.piece.origin_col + c as i32;
            if row >= 0 && (row as usize) < ROWS && col >= 0 && (col as usize) < COLS {
                self.grid[row as usize * COLS + col as usize] = self.piece.kind + 1;
            }
        }

        self.clear_lines();
        self.spawn_piece();
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        for r in 0..ROWS {
            let row = &self.grid[r * COLS..(r + 1) * COLS];
            if row.iter().all(|&v| v != 0) {
                cleared += 1;
            }
        }

        if cleared > 0 {
            let mut kept = Vec::with_capacity((ROWS - cleared as usize) * COLS);
            for r in 0..ROWS {
                let row = &self.grid[r * COLS..(r + 1) * COLS];
                if !row.iter().all(|&v| v != 0) {
                    kept.extend_from_slice(row);
                }
            }
            let mut g = vec![0u8; COLS * cleared as usize];
            g.extend_from_slice(&kept);
            self.grid = g;

            self.score += line_score(cleared) * self.level;
            self.lines += cleared;
            self.level = self.lines / 10 + 1;
        }
    }

    fn spawn_piece(&mut self) {
        let kind = self.bag.pop_front().unwrap();
        self.ensure_bag();
        let piece = Piece::spawn(kind);
        if self.collision(piece.origin_row, piece.origin_col, piece.rotation) {
            self.over = true;
        }
        self.piece = piece;
    }
}
