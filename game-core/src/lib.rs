use wasm_bindgen::prelude::*;

pub mod snake;
pub use snake::SnakeGame;

pub mod tetris;
pub use tetris::TetrisGame;

pub mod runner;
pub use runner::RunnerGame;

#[wasm_bindgen]
pub fn greet() -> String {
    String::from("Hello from Rust!")
}