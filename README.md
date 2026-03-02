# Schwarzschild Raytracer TUI

A **real-time black hole raytracer** running entirely in your terminal. It simulates gravitational lensing around a non-rotating (Schwarzschild) black hole by numerically integrating null geodesics through curved spacetime — no GPU required.

Light doesn't travel straight near a black hole. This engine traces every ray backward from the camera using **Runge-Kutta 4th order** (RK4) integration of the exact Schwarzschild geodesic equations, producing physically accurate effects:

- **Gravitational lensing** — the background starfield warps and bends around the event horizon
- **Photon sphere** — the unstable orbit at r = 1.5 r_s where light circles the black hole
- **Accretion disk** — a glowing ring of matter with temperature-based coloring (white-hot inner edge → deep red outer edge)
- **Einstein ring** — the bright ring of light visible when looking directly at the black hole

All of this is rendered using **Unicode half-block pixels** (`▀`) with 24-bit RGB colors for double vertical resolution, and parallelized across all CPU cores with [rayon](https://github.com/rayon-rs/rayon) for real-time frame rates.

See demo here:
[](https://raw.githubusercontent.com/ck90works/schwarzschild-raytracer-tui-public/refs/heads/ck90works-patch-1/demo/demo.gif)
---

## Features

- **Physically accurate** — Exact Schwarzschild geodesic integration in spherical coordinates, not flat-space approximations
- **Real-time rendering** — Targets 60 FPS on modern multi-core CPUs
- **Zero-allocation hot path** — The ray integration loop performs no heap allocations
- **Adaptive step sizing** — Small steps near the event horizon for precision, large steps in flat spacetime for speed
- **Interactive camera** — Orbit, zoom, and explore the black hole from any angle
- **Live HUD telemetry** — Observer distance, polar angle, FPS, frame time, and ray count
- **Procedural background** — Celestial grid with scattered stars to clearly show the lensing effect

---

## Prerequisites

- **Rust** ≥ 1.85.0 (Edition 2024) (I used Rust 1.93.1 to compile this project)
- A terminal emulator with **true-color (24-bit RGB)** support
  - Windows Terminal, iTerm2, Alacritty, kitty, WezTerm, etc.
  - The classic `cmd.exe` does **not** support true color

---

## Building & Running

Clone the repository and build in release mode for best performance:

```bash
git clone https://github.com/ck90works/schwarzschild-raytracer-tui-public.git
cd schwarzschild-raytracer-tui-public

# Debug build (faster compile, slower runtime)
cargo run

# Release build (recommended — enables LTO and max optimizations)
cargo run --release
```

The release profile is configured with `opt-level = 3`, link-time optimization (`lto = true`), and single codegen unit for maximum performance.

---

## Controls

| Input | Action |
|---|---|
| **Arrow keys** | Orbit the camera around the black hole |
| **Mouse drag** (left button) | Orbit the camera (click and drag) |
| **`+`** / **`=`** | Zoom in (decrease distance) |
| **`-`** / **`_`** | Zoom out (increase distance) |
| **Scroll wheel** | Zoom in/out |
| **`q`** / **Ctrl+C** | Quit |

---

## HUD Telemetry

A heads-up display in the top-left corner shows real-time information:

| Field | Description |
|---|---|
| **r** | Observer distance from the singularity (in units of Schwarzschild radius r_s) |
| **θ** | Observer polar angle in degrees |
| **FPS** | Frames per second |
| **Frame** | Render time for the last frame in milliseconds |
| **Rays** | Total RK4 integration steps computed in the last frame |

---

## Project Structure

```
src/
├── main.rs              # Entry point: terminal setup & main event loop
├── constants.rs         # All physical constants, tuning params, and style tokens
├── camera/
│   └── camera.rs        # Orbital camera (spherical coordinates, zoom, orbit)
├── physics/
│   ├── metric.rs        # Schwarzschild metric & geodesic equation coefficients
│   └── ray.rs           # RK4 integrator for null geodesics
├── render/
│   ├── framebuffer.rs   # Pre-allocated frame buffer with half-block cell encoding
│   ├── renderer.rs      # Parallel raymarcher (rayon-powered)
│   └── scene.rs         # Scene intersection: accretion disk, celestial sphere
└── tui/
    ├── app.rs           # Application state & crossterm event handling
    └── ui.rs            # ratatui widget rendering & HUD overlay
```

---

## The Physics

The Schwarzschild metric describes spacetime around a non-rotating, uncharged black hole:

```
ds² = -(1 - r_s/r) c² dt² + (1 - r_s/r)⁻¹ dr² + r² (dθ² + sin²θ dφ²)
```

The engine traces light rays backward from the camera by integrating the **null geodesic equation** derived from this metric. Each ray's path is computed in Cartesian coordinates using a 4th-order Runge-Kutta integrator with adaptive step sizing. The GR acceleration `a = -3/2 · r_s · h² / r⁵ · pos` (from the Binet orbit equation) curves each photon's trajectory. A ray terminates when it either:

- **Escapes** — reaches r > 50 r_s → shaded from the celestial background
- **Falls in** — crosses the event horizon (r ≤ r_s) → rendered as void
- **Hits the disk** — intersects the accretion disk plane between the ISCO (3 r_s) and the outer edge (12 r_s) → colored by temperature gradient

---

## Dependencies

| Crate | Purpose |
|---|---|
| [ratatui](https://ratatui.rs/) `0.30` | Terminal UI framework — widgets, layout, and rendering |
| [crossterm](https://github.com/crossterm-rs/crossterm) `0.29` | Terminal backend — raw mode, input events, alternate screen |
| [rayon](https://github.com/rayon-rs/rayon) `1.10` | Data-parallelism — splits raymarching across all CPU cores |
| [glam](https://github.com/bitshifter/glam-rs) `0.29` | SIMD-backed 3D math — vectors, dot products, cross products |

---

## Codebase Size

| Metric | Count |
|---|---|
| **Functional code** | ~583 lines |
| Test code | ~304 lines |

---

## License

This project is licensed under the **MIT License**. See [Cargo.toml](Cargo.toml) for details.
