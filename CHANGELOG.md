# Changelog

All notable changes to **AstroEngine ASCII** will be documented in this file.

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
