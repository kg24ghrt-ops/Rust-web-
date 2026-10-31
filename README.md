# Rust-web- — B5 School Notebook Paper (wgpu + Tokio)

A realistic ruled school notebook paper rendered on the GPU.

## What it draws

* **B5 sheet** — 176 × 250 mm, aspect locked to the window.
* **Blue ruling lines** — 34 horizontal lines across the page.
* **Red left margin** and **red header band** — classic notebook look.
* **White paper** on a light gray desk background.

## Stack

* **wgpu** — GPU rendering (Vulkan / Metal / DX12 / OpenGL / WebGPU).
* **winit** — windowing and input.
* **Tokio** — async runtime driving device creation and a background task.
* **bytemuck** — vertex buffer casting.

## Build & run

```bash
# Native
cargo run

# Release
cargo run --release

# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets -- -D warnings

# Test
cargo test
```

## CI

GitHub Actions builds to `wasm32-unknown-unknown` (WebGPU) on every push to
`main` and on pull requests. The CI run is the authoritative proof the code
compiles — see `.github/workflows/rust-wasm.yml`.

## Controls

* Close the window to quit.
* Resize the window — the B5 sheet resizes preserving its aspect ratio.
