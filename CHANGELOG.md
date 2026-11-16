# Changelog

All notable changes to **solar-system-ascii (AstroEngine)** will be documented in this file.

## [1.1.0] - 2026-11-16

### 🎮 UX & Navigation Overhaul & English Standardization

#### 🖱️ Full Mouse & Interactive Navigation
- **Terminal Mouse Capture**:
  - Left-click drag to orbit camera around the targeted planet or star with smooth 1:1 direct manipulation.
  - Mouse scroll wheel for continuous exponential zoom in and zoom out.
- **Polar Elevation Clamping**:
  - Orbital elevation strictly clamped to $[-1.55, +1.55]$ radians ($-88.8^\circ$ to $+88.8^\circ$), preventing camera inversion, gimbal lock, and disorienting 346° angular snap flips.
- **Continuous Mid-Flight Redirects**:
  - Retargeting during an active interplanetary transition now smoothly interpolates from the current dynamic position in space without jumping or corrupting the saved perspective views of origin planets.
- **Hierarchical Ascension (`[Backspace]`)**:
  - Intuitive one-key ascent up the celestial hierarchy: `Moon -> Parent Planet -> Solar System Overview`.
- **Sequential World Cycling (`[Tab]` / `[Shift+Tab]`)**:
  - Cycle sequentially forward and backward through all celestial worlds: `Sun -> Mercury -> Venus -> Earth -> Mars -> Ceres -> Vesta -> Jupiter -> Saturn -> Uranus -> Neptune -> Pluto -> Sun`.
- **Closed-Loop Moon Cycling (`[M]`)**:
  - Natural satellite navigation loops now return cleanly to their parent planets:
    - Earth: `Earth -> Moon -> Earth`
    - Mars: `Mars -> Phobos -> Deimos -> Mars`
    - Ceres / Vesta: `Ceres -> Vesta -> Ceres`
    - Jupiter: `Jupiter -> Io -> Europa -> Ganymede -> Callisto -> Jupiter`
    - Saturn: `Saturn -> Titan -> Enceladus -> Mimas -> Iapetus -> Rhea -> Saturn`
    - Uranus: `Uranus -> Miranda -> Ariel -> Umbriel -> Titania -> Oberon -> Uranus`
    - Neptune: `Neptune -> Triton -> Proteus -> Neptune`
    - Pluto: `Pluto -> Charon -> Pluto`
- **Planetary System Overview (`[N]`)**:
  - Smoothly transitions to curated system overview altitudes framing all satellite orbits for Mars, Jupiter, Saturn, Uranus, Neptune, and the Asteroid Belt.
- **Interactive Help Modal (`[?]` / `[F1]`)**:
  - Centered on-screen dialog overlay displaying comprehensive controls for mouse and keyboard navigation. Dismissible with `[?]` or `[Esc]`.
- **Time Warp Reset (`[\]` / `[|]`)**:
  - Immediate one-key reset back to default simulation rate ($10\text{ min/s}$, $600\times$).

#### 🌐 Complete English Standardization
- **HUD & Telemetry**:
  - Standardized all HUD headers, breadcrumbs, and status labels to English (`Solar System › Jupiter › Europa (Moon 2/4)`, `Transit: Earth -> Mars`, `min/s`, `hr/s`, `days/s`, `months/s`, `years/s`).
- **3D Celestial Labels**:
  - Standardized all 3D billboards in raycaster and HUD to English: `[Sun]`, `[Mercury]`, `[Venus]`, `[Earth]`, `[Moon]`, `[Mars]`, `[Jupiter]`, `[Saturn]`, `[Uranus]`, `[Neptune]`, `[Pluto]`, `[Callisto]`, `[Ganymede]`, etc.
- **CLI & Controls Bar**:
  - Updated CLI `--help` with `--target pluto/charon` and modern English navigation controls.

#### ⚡ Clean Build & Dependency Pruning
- Pruned unused `rand` dependency from `Cargo.toml`, eliminating compiler warnings and speeding up dependency resolution.

---

## [1.0.0] - 2026-11-15

### 🚀 Initial Public Release: Complete Solar System Simulator

#### 🌌 Solar System & Planetary Dynamics
- **Complete Planetary System**: Full Keplerian 3D ephemeris for the Sun, Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune, and dwarf planet Pluto.
- **Natural Moons**:
  - Earth: Luna with tidal locking, basaltic maria and impact craters.
  - Mars: Phobos and Deimos with realistic orbits and cratered bodies.
  - Jupiter: Galilean moons (Io, Europa, Ganymede, Callisto) + Jovian outer moon cloud.
  - Saturn: Major moons (Titan, Enceladus, Mimas, Iapetus, Rhea, Hyperion, Phoebe) + photorealistic A/B/C rings with Cassini & Encke divisions.
  - Uranus: Uranian ring system + Miranda, Ariel, Umbriel, Titania, Oberon with 97.8° axial tilt.
  - Neptune: Azure storm cloud bands, retrograde Triton (cryogeysers & cantaloupe terrain) and Proteus.
  - Pluto & Charon: Tombaugh Regio nitrogen heart, Sputnik Planitia, and binary barycentric companion Charon.
- **Main Asteroid Belt**: 3,500 particles accurately aligned on the ecliptic plane with dwarf planets **Ceres** (Occator crater bright spots) and **Vesta** (Rheasilvia impact basin).
- **Interstellar Spacecraft**: Deep-space trajectories for **Voyager 1** and **Voyager 2** beyond the heliopause.

#### 🔭 Celestial Sky & Navigation
- **Authentic Starfield**: Hipparcos astronomical catalogue stars rendered with true celestial coordinates, apparent magnitudes, and spectral class colors (Sirius, Canopus, Vega, Betelgeuse, Rigel, Alpha Centauri, etc.).
- **Constellations & Deep Sky Objects**: Orion, Ursa Major, Scorpius, Cassiopeia, Crux, Orion Nebula, Carina Nebula, and the Magellanic Clouds.
- **Orrery & Camera Navigation**:
  - Top-down Orrery Overview (`[Tab]`) as the default entry point with elliptical orbit traces (`[L]`).
  - Seamless cinematic zoom transition between planetary close-ups and solar system scale.
  - Curated beauty viewing angles (`[R]`) for every celestial body, including Saturn's iconic oblique ring shadow perspective.
  - Contextual HUD displaying Magellan synthetic aperture radar mode (`[V]`) on Venus and moon navigation (`[M]`).
  - Seamless moon cycling on Mars: `[M]` switches between Phobos and Deimos (`Phobos <-> Deimos`).
  - Sun Zoom Toggle: `[0]` and `[Z]` seamlessly zoom into the close-up Sun (2.8M km, coronal loops & granulation) and zoom back out to the full Solar System Orrery (1.1e10 km).
  - Extended Astronomical Time Warp: acceleration scales up to **10 years / second** (`315,360,000x`) with key `[T]` to cycle major astronomical presets (10 min/s, 1 day/s, 1 month/s, 1 year/s, 5 years/s, 1x).

#### 🎨 Application Packaging & Icons
- **Native Executable Icons**:
  - Embedded Windows PE icon resource (`assets/icon.ico`) with multi-resolution 256x256 down to 16x16 PNG layers via `winres`.
  - macOS multi-resolution icon bundle (`assets/icon.icns`) via `iconutil`.
  - High-resolution 512x512 PNG icon (`assets/icon.png`).

#### ⚡ Engine & Performance
- **Dual Rendering Modes**: 24-bit TrueColor Half-Block mode (`▀`) for doubled vertical pixel density and Classic ASCII retro mode.
- **Analytical Raycasting**: Pure Rust ray-sphere and ray-ring intersection algorithms running at 600+ FPS via Rayon parallel execution. Zero external graphics dependencies.
