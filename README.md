# AstroEngine ASCII 🌌

> **Real-time 3D Solar System simulator in TrueColor ANSI Half-Block raymarching for any terminal (Rust edition)**

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Terminal](https://img.shields.io/badge/terminal-TrueColor%2024--bit-green.svg)]()

---

## 🌟 Highlights

- **The Complete Solar System**:
  - **Sun & Inner Planetes**: Dynamic solar plasma turbulence, Mercury's cratered Caloris Basin, Venus with thick sulfuric clouds and Magellan synthetic radar mode (`[V]`), Earth with continental geography, night lights, biomes & cloud shadows, Luna with synchronized tidal locking, and Mars with Valles Marineris, Olympus Mons, and Phobos & Deimos.
  - **Main Asteroid Belt**: 3,500 Keplerian particles aligned on the ecliptic plane with dwarf planets **Ceres** (Occator crater bright spots) and **Vesta** (Rheasilvia impact basin).
  - **Jupiter System**: Dynamic cloud bands, Great Red Spot, four Galilean moons (**Io**, **Europa**, **Ganymede**, **Callisto**) plus the Jovian irregular moon cloud.
  - **Saturn & Rings**: Photorealistic ring system with Cassini & Encke divisions, shadows cast on the gas giant, plus major moons (**Titan**, **Enceladus**, **Mimas**, **Iapetus**, **Rhea**, **Hyperion**, **Phoebe**).
  - **Ice Giants & Dwarf Worlds**: Uranus with extreme $97.8^\circ$ axial tilt and 5 major moons; Neptune with azure storms, Great Dark Spot, retrograde **Triton** and **Proteus**; **Pluto** with Tombaugh Regio heart, Sputnik Planitia, and binary companion **Charon**.
  - **Deep Space Probes**: **Voyager 1** and **Voyager 2** navigating beyond the heliopause into interstellar space.
- **Realistic Deep Sky Background**:
  - Accurate positions and spectral colors for real stars (Sirius, Canopus, Alpha Centauri, Arcturus, Vega, Betelgeuse, Rigel, etc.).
  - Realistic constellations (Orion, Ursa Major, Scorpius, Cassiopeia, Crux, etc.) and nebulae (Orion Nebula, Carina, Magellanic Clouds).
- **Dual Perspective & Smooth Transitions**:
  - **Top-Down Orrery Overview**: Starts directly in an intuitive top-down view of the entire solar system with planetary orbit paths (`[L]`).
  - **Smooth Cinematic Camera Transitions**: Exponential zoom-out from any planet automatically transitions into the solar system overview.
  - **Curated Beauty Angles**: One-key camera reset (`[R]`) or direct selection (`[0-9]`) perfectly frames every planet at its most iconic perspective.
- **Dual-Mode Rendering Engine**:
  - **Half-Block Mode (`▀`)**: Utilizes independent ANSI 24-bit TrueColor foreground and background escape codes, doubling vertical pixel density for smooth, razor-sharp planetary globes.
  - **Classic Retro ASCII Mode**: Dynamic luminance ramp with TrueColor foreground text.
  - Toggle between modes on the fly with key `B`.
- **High-Performance Multi-Threading**:
  - Parallel scanline raycasting powered by `Rayon`, running at **600+ FPS** in release mode.

---

## 🎮 Controls

| Key | Action |
| :--- | :--- |
| `0` / `Tab` / `Z` | **Top-Down Overview** $\leftrightarrow$ **Zoom in/out of Sun** |
| `1` – `8` | Jump to **Mercury**, **Venus**, **Earth**, **Mars**, **Jupiter**, **Saturn**, **Uranus**, **Neptune** |
| `9` | Jump to **Pluto** |
| `M` | Cycle through **Moons** of the current focused body |
| `V` | Toggle **Magellan Radar / Atmosphere** (on Venus) |
| `T` | Quick-cycle **Time Warp Presets** (10 min/s, 1 day/s, 1 month/s, 1 year/s, 5 years/s, 1x) |
| `[` / `]` | Fine time warp speed adjustment (up to **10 years / second**) |
| `L` | Toggle **Orbital Path Lines** |
| `P` | Toggle **Asteroid Belt** particles |
| `R` | Reset camera to **Optimal Beauty Angle** |
| `+` / `=` | **Zoom In** (continuous exponential zoom) |
| `-` / `_` | **Zoom Out** (continuous exponential zoom) |
| `W` / `S` | Orbit camera pitch (up / down) |
| `A` / `D` | Orbit camera yaw (left / right) |
| `B` | Toggle Render Mode: **Half-Block (`▀`)** $\leftrightarrow$ **Classic ASCII** |
| `Space` | **Pause / Resume** simulation and orbital motion |
| `Q` / `Esc` | Quit gracefully and restore terminal |

---

## 🚀 Installation & Pre-built Binaries

Pre-compiled standalone binaries with zero dependencies are automatically built and released for:
- **Windows x64** (`.zip`)
- **Linux x86_64** (GNU & Static musl `.tar.gz`)
- **Linux ARM64** (`.tar.gz`)
- **macOS Apple Silicon (M1/M2/M3/M4)** (`.tar.gz`)
- **macOS Intel x86_64** (`.tar.gz`)

Download the latest binary from the **[Releases](https://github.com/PedroSmaxY/solar-system-ascii/releases)** tab.

### Build from Source

#### Prerequisites
- [Rust](https://www.rust-lang.org) (edition 2021, rustc 1.80+)
- A modern terminal supporting 24-bit TrueColor ANSI (Apple Terminal, iTerm2, Alacritty, Kitty, Ghostty, WezTerm, Windows Terminal).

```bash
# Clone the repository
git clone https://github.com/PedroSmaxY/solar-system-ascii.git
cd solar-system-ascii

# Run in high-performance Release mode
cargo run --release

# Launch directly with High-Definition Half-Block mode
cargo run --release -- --hd
```

---

## 📐 Architecture & Physics

- **Coordinate System**: Dual-scale 64-bit IEEE 754 floating-point (`f64`) world coordinates in kilometers.
- **Ephemeris**: Keplerian two-body mechanics solved via Newton-Raphson iteration on Kepler's transcendental equation $M = E - e \sin E$.
- **Analytical Raycasting**: Solves quadratic ray-sphere and ray-ring plane intersections in $O(1)$ per character, guaranteeing infinite geometric resolution without polygon discretization or point-cloud sparsity.

---

## 📜 License

MIT License. Crafted with precision by PedroSmaxY.
