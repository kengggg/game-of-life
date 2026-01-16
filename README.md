# Game of Life with Color Blending

A Conway's Game of Life implementation with color blending for 25 company members, built with Rust WebAssembly and HTML5 Canvas.

## Features

- Classic Conway's Game of Life rules
- 25 unique colors (one per company member)
- RGB color blending when cells are born from different colored neighbors
- High-performance WebAssembly implementation
- Fullscreen browser display
- Smooth 60fps animation

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (v14 or later)
- npm (comes with Node.js)

## Build Instructions

### 1. Build the Rust WebAssembly module

```bash
wasm-pack build --target web
```

### 2. Install web dependencies

```bash
cd www
npm install
```

### 3. Run the development server

```bash
npm start
```

The application will be available at `http://localhost:8080`

## Production Build

To create a production build:

```bash
# Build WASM with optimizations
wasm-pack build --target web --release

# Build web assets
cd www
npm run build
```

The production files will be in `www/dist/`

## How It Works

- **25 Colors**: Each company member is assigned a unique color from the HSL color space
- **Initial Seeds**: Random cells are scattered across the grid for each person
- **Game of Life Rules**:
  - Birth: Dead cell with exactly 3 live neighbors becomes alive
  - Survival: Live cell with 2-3 live neighbors stays alive
  - Death: All other cases result in cell death
- **Color Blending**: When a new cell is born, its color is the average RGB of its 3 parent cells
- **Toroidal Grid**: Edges wrap around for an infinite feel

## Project Structure

```
game-of-life/
├── Cargo.toml          # Rust project configuration
├── src/
│   └── lib.rs          # Game of Life WASM implementation
└── www/
    ├── package.json    # NPM dependencies
    ├── webpack.config.js
    ├── index.html      # Main HTML page
    ├── index.js        # Canvas rendering and WASM integration
    └── bootstrap.js    # WASM module loader
```

## Performance

The implementation uses WebAssembly for game logic computation and HTML5 Canvas for rendering, achieving smooth 60fps animation on modern browsers.
