use wasm_bindgen::prelude::*;

pub mod snake;
pub use snake::SnakeGame;

#[wasm_bindgen]
pub fn greet() -> String {
    String::from("Hello from Rust!")
}