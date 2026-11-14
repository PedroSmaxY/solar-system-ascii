# AstroEngine ASCII 🌌

> **Real-time 3D Solar System & Earth Simulator in TrueColor ANSI for any terminal (Rust edition)**

[![Rust](https://img.shields.io/badge/rust-1.99%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Terminal](https://img.shields.io/badge/terminal-TrueColor%2024--bit-green.svg)]()

---

## 🌟 Highlights

- **Ultra-Detailed 3D Earth (Terra)**:
  - **Hybrid Topography**: Real continental geography (Natural Earth 110m) combined with multi-frequency spherical fractal noise for infinite coastline detail.
  - **Dynamic Biomes**: Dense equatorial rainforests (Amazon, Congo), golden arid deserts (Sahara, Australian Outback, Atacama), temperate woodlands, high mountain ranges (Andes, Himalayas, Rockies, Alps), and reflective polar ice sheets.
  - **Living Human Civilization (Night Lights)**: As dusk sweeps past the terminator, over 25 global metropolitan hubs (Tokyo, São Paulo, New York, London, Paris, Cairo, Delhi, Shanghai, etc.) ignite with warm golden-amber glow (`#ffd75f`), featuring connected highway corridors and organic micro-twinkling traffic pulses.
  - **Rayleigh Atmospheric Scattering**: Radiant cyan/blue atmospheric limb glow (`[65, 165, 255]`) softly rimming the planet's curvature against deep space.
  - **Ocean Sun Glint**: Physically grounded specular highlight reflecting the Sun across ocean waters.
  - **Drifting Cloud Layer**: Procedural atmospheric cloud layer rotating with independent differential velocity and casting subtle ground shadows.
- **The Moon (Luna)**:
  - **Tidal Locking**: Geometrically synchronized rotation preserving the near-side orientation facing Earth.
  - **Lunar Geography**: Dark basaltic maria (*Oceanus Procellarum*, *Mare Imbrium*, *Mare Tranquillitatis*, etc.) vs. bright anorthositic highlands with procedural impact craters.
  - **Astronomical Phase Illumination**: Exact solar phase angle shading (New Moon, Crescent, Quarter, Full, Gibbous).
- **Dual-Mode Rendering Engine**:
  - **Half-Block Mode (`▀`)**: Utilizes independent ANSI 24-bit TrueColor foreground and background escape codes, doubling vertical pixel density for smooth, sharp planetary globes.
  - **Classic Retro ASCII Mode**: Dynamic luminance ramp (` .:-=+*x%#@`) with TrueColor foreground text.
  - Toggle between modes on the fly with key `B`.
- **Astronomical Scale & Continuous Zoom**:
  - Continuous exponential zoom from Low Earth Orbit ($7,000\text{ km}$) through cislunar space ($384,400\text{ km}$) out to planetary distances.
- **High-Performance Multi-Threading**:
  - Parallel scanline raycasting powered by `Rayon`, running at **600+ FPS** in release mode.

---

## 🎮 Controls

| Key | Action |
| :--- | :--- |
| `+` / `=` | **Zoom In** (continuous exponential zoom) |
| `-` / `_` | **Zoom Out** (continuous exponential zoom) |
| `W` / `Up` | Orbit camera upward |
| `S` / `Down` | Orbit camera downward |
| `A` / `Left` | Orbit camera left |
| `D` / `Right` | Orbit camera right |
| `1` | Focus camera on **Earth** |
| `2` | Focus camera on **Moon** |
| `B` | Toggle Render Mode: **Half-Block (`▀`)** $\leftrightarrow$ **Classic ASCII** |
| `Space` | **Pause / Resume** simulation and orbital motion |
| `[` | Decrease time warp speed |
| `]` | Increase time warp speed ($1\times, 60\times, 3600\times, 86400\times$) |
| `Q` / `Esc` | Quit gracefully and restore terminal |

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org) (edition 2021, rustc 1.80+)
- A modern terminal supporting 24-bit TrueColor ANSI (Apple Terminal, iTerm2, Alacritty, Kitty, Ghostty, WezTerm, VS Code Terminal, Windows Terminal).

### Build & Run

```bash
# Clone the repository
git clone https://github.com/PedroSmaxY/solar-system-ascii.git
cd solar-system-ascii

# Run in high-performance Release mode (starts in default ASCII mode)
cargo run --release

# Or launch directly into High-Definition Half-Block mode
cargo run --release -- --hd

# Benchmark mode (runs N frames and outputs frame telemetry)
cargo run --release -- --frames 120
```

---

## 📐 Architecture & Physics

- **Coordinate System**: Dual-scale 64-bit IEEE 754 floating-point (`f64`) world coordinates in kilometers.
- **Ephemeris**: Keplerian two-body mechanics solved via Newton-Raphson iteration on Kepler's transcendental equation $M = E - e \sin E$.
- **Analytical Raycasting**: Solves quadratic ray-sphere intersections in $O(1)$ per character, guaranteeing infinite geometric resolution without polygon discretization or point-cloud sparsity.

---

## 📜 License

MIT License. Crafted with precision by PedroSmaxY.
