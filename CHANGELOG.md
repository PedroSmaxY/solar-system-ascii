# Changelog

All notable changes to **solar-system-ascii (AstroEngine)** will be documented in this file.

## [1.5.0] - 2026-11-16

### 🌍 "Momento Zero", Carl Sagan's "Pale Blue Dot" Experience & 100% Reactive HUD

#### 🎬 Cinematic "Momento Zero" Startup Experience
- **Interactive Orbital Approach to Earth**:
  - Launching the simulator interactively initiates an automated, non-blocking 2.4-second orbital glide from high orbit ($58{,}000\text{ km}$, $\text{azimuth}=1.45$, $\text{elevation}=0.48$) into Earth's curated beauty angle ($21{,}000\text{ km}$, $\text{azimuth}=2.30$, $\text{elevation}=0.28$).
  - Instant visual 3D gratification from Frame 0: immediate full globe rendering with Rayleigh atmospheric twilight scattering, daylit continents, and night city lights.
  - 100% responsive: any mouse drag, zoom scroll, or keypress instantly takes direct control without blocking.
- **On-Screen Welcome Guide (HUD Line 1)**:
  - Non-intrusive golden welcome banner displayed during the first 6 seconds of simulation:
    `✦ Welcome to AstroEngine 3D · Press [0] Solar System Orrery  [1-9] Planets  [M] Moons  [Y] Voyagers  [?] Help ✦`

#### 📐 Geometrically Calibrated Default Distances & Oblique Angles
- **Solar System Orrery (`[0]`)**:
  - Default overview distance expanded from $11\times 10^9\text{ km}$ to **$18\times 10^9\text{ km}$ ($120.3\text{ AU}$)**, perfectly framing all 8 planets, the Asteroid Belt, and Pluto's complete elliptical orbit with ~22% margin.
  - Solar system maximum zoom-out expanded to **$1.5\times 10^{11}\text{ km}$ (~1,000 AU)**.
- **Planetary System Views (`[N]`)**:
  - Calibrated distances to frame all outer natural satellites with generous margins:
    - **Saturn**: Expanded to **$11{,}000{,}000\text{ km}$** (framing Iapetus at $3{,}560{,}000\text{ km}$ 100% cleanly).
    - **Jupiter**: Expanded to **$6{,}000{,}000\text{ km}$** (framing Callisto at $1{,}882{,}700\text{ km}$).
    - **Earth**: Expanded to **$1{,}250{,}000\text{ km}$** (framing the Moon at $384{,}400\text{ km}$).
    - **Mars**: Expanded to **$78{,}000\text{ km}$** (framing Deimos at $23{,}460\text{ km}$).
    - **Uranus & Neptune**: Expanded to **$1{,}900{,}000\text{ km}$** and **$1{,}800{,}000\text{ km}$**.
    - **Pluto**: Expanded to **$65{,}000\text{ km}$** (framing Charon's binary orbit).
  - Entering System View automatically shifts to an oblique 3D angle ($\text{azimuth}=0.25$, $\text{elevation}=0.28$) so moon orbits open into wide concentric ellipses instead of looking flat edge-on.
  - Satellite orbit line rendering threshold (`max_cam_dist`) expanded 5x–10x across all planetary systems.

#### 🛰️ Carl Sagan's 1990 "Pale Blue Dot" Experience (`[F]` at Voyager)
- **Over-the-Shoulder Camera Framing**:
  - Pressing `[F]` at Voyager smoothly glides the camera into an over-the-shoulder perspective behind the high-gain antenna dish ($8.5\text{ m}$ distance), looking forward past the antenna rim.
- **True 3D Vector & Elevation Calculation**:
  - Fixed 3D solar/terrestrial alignment elevation to calculate true astronomical declination (accounting for Voyager 1's $+35^\circ$ and Voyager 2's $-48^\circ$ trajectories).
- **1990 Sagan Scattered Sunbeam & Cyan Sub-pixel Earth**:
  - Raycaster projects Earth coordinate from $> 24\text{ billion km}$ with a subtle scattered optical diffraction sunbeam ray and bright cyan sub-pixel point (`[135, 215, 255]`, `'*'`).
- **HUD Target Prioritization**:
  - Prioritizes `◉ [Earth · Pale Blue Dot (163.5 AU)]` at top of label placement queue, preventing Sun label collision at 160 AU.

#### 🎛️ 100% Reactive Contextual HUD
- **Dynamic Action Toggles in Control Bar**:
  - All contextual keys dynamically indicate the exact inverse action they will perform:
    - `[N]`: `[N] Planet Close-up` $\leftrightarrow$ `[N] System View`
    - `[0]`: `[0] Sun Close-up` $\leftrightarrow$ `[0] Solar Overview`
    - `[Z]`: `[Z] Full Craft (12m)` $\leftrightarrow$ `[Z] Golden Record (3m)`
    - `[F]`: `[F] Craft View` $\leftrightarrow$ `[F] Pale Blue Dot`
    - `[V]`: `[V] Optical Atmosphere` $\leftrightarrow$ `[V] Radar View`
    - `[L]`: `[L] Orbits: ON` $\leftrightarrow$ `[L] Orbits: OFF`
    - `[Space]`: `[Space] Resume` $\leftrightarrow$ `[Space] Pause`
- **Dynamic Header Breadcrumb**:
  - Header breadcrumb reactively reflects current mode (`"Solar System › Jupiter (System Overview)"`, `"Solar System › Venus (Magellan SAR Radar)"`, `"Solar System › Interstellar Space › Voyager 1 (Pale Blue Dot)"`).

### 🛰️ Voyager 1 & 2 Interstellar Spacecraft & "Click-to-Fly" Close-up Exploration

#### 📡 NASA Voyager 1 & 2 3D Spacecraft Modeling (`VoyagerShader`)
- **Analytical $O(1)$ CSG Raycaster**:
  - Pure mathematical ray-spacecraft intersection solver running in local probe coordinates with sub-millimeter precision.
  - **High-Gain Antenna (HGA)**: 3.66-meter diameter parabolic reflector ($y = c(x^2 + z^2)$) with reflective white fiberglass texture and concentric reinforcement ribs.
  - **Subreflector & Feed Horn**: Focal point subreflector mounted on titanium tripod struts.
  - **Decagonal Equipment Bus**: 10-sided chassis clad in shimmering gold Mylar multilayer insulation (MLI) blankets with micro-wrinkle specular sheen.
  - **The Golden Record ("The Sounds of Earth")**: 30-centimeter electroplated 24K gold disc on the $+X$ chassis face with concentric phonograph sound tracks and pulsar map etchings.
  - **RTG Power Boom**: 3 stacked radioisotope thermoelectric generator canisters extending along $-X$ with graphite beryllium casing and thermal infrared heat seams.
  - **Magnetometer Astromast**: 13-meter slender fiberglass lattice boom extending along $+Z$ with tip sensor sphere.

#### 🚀 Interactive Interstellar Flight & Controls
- **"Click-to-Fly" to Interstellar Probes**:
  - Clicking on `[Voyager 1 (163.5 AU)]` or `[Voyager 2 (136.8 AU)]` on the screen initiates an immediate, smooth flight across 24 billion kilometers out to the interstellar edge.
- **Dedicated Shortcut `[Y]`**:
  - Toggles focus directly between Voyager 1 ($163.5\text{ AU}$) and Voyager 2 ($136.8\text{ AU}$).
- **`[Z]` Golden Record Close-up**:
  - When targeting Voyager, pressing `[Z]` glides the camera into an intimate 3-meter close-up directly facing the Golden Record cover.
- **`[F]` "Pale Blue Dot" Alignment**:
  - Aligns the camera perspective directly back along the antenna boresight towards the Sun and Earth, recreating the iconic 1990 portrait.
- **`[Backspace]` Hierarchical Ascension**:
  - Smoothly ascends from the 12-meter close-up back to the top-down Solar System Orrery view.

#### 📈 Historical Telemetry & HUD Integration
- Contextual HUD bar displays interstellar status: `"Solar System › Interstellar Space › Voyager 1 (163.5 AU)"`.
- Sub-kilometer distance formatting in meters (`12.0 m`, `3.0 m`) during close-up examination.
- Contextual bottom control bar reflects Voyager modes (`[Z] Golden Record  [F] Pale Blue Dot  [Y] Voyager 2  [Bksp] Overview`).
- Integrated into the interactive help modal (`[?]`).

## [1.3.0] - 2026-11-16

### 🪐 Interactive "Click-to-Fly", Lunar Orbit Lines & Navigation Polish

#### 🖱️ Interactive "Click-to-Fly" Mouse Navigation
- **Screen-Space Raycasting & Selection**:
  - Left-clicking anywhere on a celestial body or its 3D text tag (`[Name]`) automatically triggers a smooth cinematic flight (`start_transition`) directly to that world.
  - Clicking on the bottom-right `[?] Help` indicator toggles the interactive help modal.

#### 🛰️ Planetary Moon Orbit Lines (`[L]`)
- **NASA Eyes-Style Lunar Trajectories**:
  - In addition to the 10 heliocentric planetary orbits, toggling `[L]` now renders the elliptical Keplerian trajectories for all major natural satellites around their host worlds:
    - Earth: Moon
    - Mars: Phobos, Deimos
    - Jupiter: Io, Europa, Ganymede, Callisto
    - Saturn: Titan, Enceladus, Mimas, Iapetus, Rhea
    - Uranus: Miranda, Ariel, Umbriel, Titania, Oberon
    - Neptune: Triton, Proteus
    - Pluto: Charon
  - Hierarchical LOD: Satellite orbits only activate within their host planet's proximity, avoiding visual noise at interplanetary distances.

#### 🎯 System View (`[N]`) Polish & Parabolic Overshoot Fix
- **Elimination of Parabolic Spike**:
  - In same-body transitions (`from_body == to_body`), `journey_dist_km` is set strictly to `0.0`, resulting in pure logarithmic distance zooming without false parabolic altitude spikes when returning to surface view.
- **Oblique System Overview Perspective**:
  - When toggling to system overview (`[N]`), camera pitch elevates to at least $0.38\text{ rad}$ ($22^\circ$), opening up the satellite orbital plane in elegant 3D perspective rather than an edge-on disc.
- **Contextual HUD Labeling**:
  - Displays `"Zoom: <World> Close-up"` and `"Zoom: <World> System Overview"` instead of `"Transit: <World> -> <World>"`.

#### 🚀 Camera Velocity & Inertia Smoothing
- **Transition Velocity Reset**:
  - `vel_azimuth` and `vel_elevation` are zeroed out upon initiating transitions, canceling transitions, or aligning to the Sun, preventing residual angular impulses from kicking the camera off-target upon arrival.
- **Clamped Orbit Impulse**:
  - Angular velocity increments are clamped to $[-0.5, 0.5]$ to prevent runaway spinning from rapid trackpad flicks.

#### 📊 Responsive HUD & Astronomical Telemetry
- **Guaranteed Visibility on 80x24 Terminals**:
  - Intelligent responsive header compacts the celestial breadcrumb string when terminal columns $\le 80$, ensuring `PAUSED`, simulation time warp, and FPS are never suppressed.
- **Simulation Epoch Indicator**:
  - Displays elapsed simulated mission time (`T+ 14.2d` / `T+ 1.45y`) in the top HUD telemetry header.
- **Single-Key Help Shortcut (`/`)**:
  - Added `/` as a zero-modifier shortcut to open help on international and ABNT2 keyboards.
  - Documented fine time warp `[,]` / `[.]` in the Help Modal.

---

## [1.2.0] - 2026-11-16

### 🚀 UX, Cross-Platform Robustness & Terminal Polishing

#### 🛡️ Terminal Safety & Cross-Platform Reliability
- **Panic Hook Safety (`std::panic::set_hook`)**:
  - Automatically resets terminal cursor visibility, exits alternate screen, disables mouse tracking, and disables raw mode if an unexpected panic occurs, preventing terminal bricking or frozen escape states.
- **Raw Mode `Ctrl+C` & `Ctrl+D` Interception**:
  - Captured POSIX/Windows control sequences directly within raw event loop to guarantee immediate, graceful terminal restoration and exit.
- **Teardown Sequence Fix**:
  - Inverted cleanup sequence to disable mouse capture *before* leaving the alternate screen buffer, eliminating stray mouse escape codes in Windows Terminal, tmux, and Linux consoles.
- **Keyboard Auto-Repeat**:
  - Added support for `KeyEventKind::Repeat` under Windows Terminal and Kitty keyboard protocols for uninterrupted keyboard orbiting.

#### 🎯 Navigation & Workflow Polish
- **P0 Fix: Sequential World Cycling (`[Tab]` / `[Shift+Tab]`)**:
  - Fixed infinite loop trap on Mercury/Pluto by prioritizing world array membership over generic parent resolution. Cycles smoothly through all 12 celestial worlds (`Sun -> Mercury -> ... -> Pluto -> Vesta -> Sun`).
- **P0 Fix: Bidirectional System Overview Toggle (`[N]`)**:
  - Implemented dynamic toggle between default surface beauty angle and full orbital overview altitude for Earth ($600,000\text{ km}$), Mars ($45,000\text{ km}$), Ceres ($650,000,000\text{ km}$), Jupiter ($2,800,000\text{ km}$), Saturn ($1,800,000\text{ km}$), Uranus ($1,200,000\text{ km}$), Neptune ($1,500,000\text{ km}$), and Pluto ($35,000\text{ km}$).
- **Mouse Drag Calibration**:
  - Recalibrated mouse drag sensitivity to $-0.022$ azimuth / $+0.035$ elevation for natural 1:1 orbiting feel across high-DPI trackpads and mice.
- **Mouse Click Modal Dismissal**:
  - Left-clicking anywhere on the screen now dismisses the interactive Help Modal.
- **Smooth Orbit Damping**:
  - Added subtle exponential orbital velocity damping for keyboard panning (`A`/`D`/`W`/`S`), providing cinematic smoothness.

#### 🎨 Visual Polish & Tear-Free Rendering
- **DEC Mode 2026 Synchronized Output**:
  - Enclosed ANSI output buffer with `\x1b[?2026h` ... `\x1b[?2026l` atomic sync markers, eliminating scanline tearing on modern terminals (Ghostty, Kitty, iTerm2, WezTerm, Windows Terminal).
- **Text Backdrop Dimming**:
  - Sub-pixel dimming on text row cells in Half-Block mode ensures HUD text and 3D labels remain razor-sharp and legible even when passing over the solar corona or bright planetary limbs.
- **HUD UTF-8 & Responsive Layout**:
  - Replaced byte length with `.chars().count()` for UTF-8 glyph calculation; redesigned compact controls bar to guarantee clean fit within standard $80 \times 24$ terminal windows.
- **Clean Visual Showcase Artifacts**:
  - Regenerated all gallery screenshots without raw-buffer HUD text artifacts (corner bars and colored text dashes).

---

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
