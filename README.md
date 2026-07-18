# HCR Web Twin Simulator

This project is a 3D digital twin simulator for hair-cutting robotics. It utilizes Rust and WebAssembly for high-performance physics computation via WebGPU, rendered in the browser using Babylon.js.

### Prerequisites
* Rust & Cargo
* wasm-pack

### Build Instructions
1. Navigate to the haircut_core directory:
    ```bash
    cd haircut_core
    ```

2. Build the project for the web:
    ```bash
    wasm-pack build --target web
    ```

3. Return to the root directory and serve the project:
    ```bash

    cd ..
    # Start local server from this root directory
    ```

## Important Note
The hotaru_mqtt folder is a required mock dependency to ensure the workspace builds correctly. Please do not remove it, as it is structured to interface with the system's internal broker.
