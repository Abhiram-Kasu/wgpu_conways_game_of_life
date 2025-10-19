# Conway's Game of Life - WGPU Implementation

A high-performance implementation of Conway's Game of Life using WGPU (WebGPU) and Rust, with GPU compute shaders for simulation.

## Screenshot

<img width="2550" height="1426" alt="image" src="https://github.com/user-attachments/assets/47a361d8-eb8a-4a50-a173-b2133315043a" />


## Features

- **GPU-Accelerated Simulation**: Uses WGPU compute shaders to run Conway's Game of Life entirely on the GPU
- **Real-time Rendering**: Fast and efficient rendering using modern graphics APIs
- **Interactive**: Uses Winit for window management and event handling
- **Compute Shaders**: Written in WGSL (WebGPU Shading Language) for maximum performance

## Prerequisites

- Rust (latest stable version)
- Cargo
- A GPU that supports WGPU

## Building

To build the project, run:

```bash
cargo build --release
```

## Running

To run the simulation:

```bash
cargo run --release
```

The simulation will open in a 1280x720 window and start running automatically.

## Dependencies

- `wgpu` - WebGPU implementation for Rust
- `winit` - Window creation and event handling
- `pollster` - Async runtime for WGPU initialization
- `bytemuck` - Safe casting between plain data types
- `rand` - Random number generation for initial state
- `env_logger` / `log` - Logging utilities

## How It Works

The simulation uses a double-buffering technique with two textures:
1. One texture stores the current state of the grid
2. A compute shader reads from the current state and writes to the next state
3. The textures are swapped each frame
4. A render pipeline displays the current state

The compute shader implements the classic Conway's Game of Life rules:
- Any live cell with 2 or 3 live neighbors survives
- Any dead cell with exactly 3 live neighbors becomes alive
- All other cells die or remain dead
