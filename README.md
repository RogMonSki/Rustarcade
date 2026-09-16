# Rustarcade

A small arcade of classic games, with all game logic written in Rust and compiled to WASM, and a Svelte 5 frontend for rendering and input.

**Play it here: https://rogmonski.github.io/Rustarcade/**

## Games

- Snake
- Tetris
- Runner

## Project Structure

- `frontend/`: Svelte 5 + Vite + TailwindCSS landing page and game UIs
- `game-core/`: Rust crate compiled to WASM, containing the actual game logic

Each game lives entirely as a Rust struct in `game-core/src/<game>.rs`, exposed to JS via
`wasm-bindgen`. The frontend never implements game rules itself — it only calls into the compiled
WASM module and renders the state it returns.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and `wasm-pack`
- Node.js and npm

### Build the game-core WASM package

```bash
cd game-core && wasm-pack build --target bundler
```

### Run the frontend

```bash
cd frontend
npm install
npm run dev
```

### Build the frontend for production

```bash
cd frontend && npm run build
```

Note: after any change to `game-core`, rebuild the WASM package and run `npm install` in
`frontend` again to pick up the changes (Vite does not watch Rust source files).

## License

See [LICENSE](LICENSE).
