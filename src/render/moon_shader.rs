//! Ultra-accurate Moon Shader:
//! - Complete catalog of Lunar Maria (Oceanus Procellarum, Imbrium, Serenitatis, Tranquillitatis, etc.)
//! - Catalog of 32+ prominent documented craters (Tycho with massive rays, Copernicus, Kepler, Plato, Clavius, Aristarchus)
//! - Ejection ray filaments, terraced elevated rims, dark basalt floors & central peaks
//! - Synchronous tidal locking alignment and physical solar phase lighting

use crate::math::Vec3;
use std::f64::consts::PI;

pub struct MoonShader;

struct LunarMare {
    lat: f64,
    lon: f64,
    r_lat: f64,
    r_lon: f64,
}

// Major basaltic maria on the Moon
const MARIA: &[LunarMare] = &[
    LunarMare { lat: 18.0, lon: -57.0, r_lat: 28.0, r_lon: 35.0 }, // Oceanus Procellarum
    LunarMare { lat: 33.0, lon: -16.0, r_lat: 20.0, r_lon: 24.0 }, // Mare Imbrium
    LunarMare { lat: 28.0, lon: 18.0, r_lat: 16.0, r_lon: 18.0 },  // Mare Serenitatis
    LunarMare { lat: 8.5, lon: 31.0, r_lat: 15.0, r_lon: 18.0 },   // Mare Tranquillitatis
    LunarMare { lat: -3.0, lon: -23.0, r_lat: 14.0, r_lon: 16.0 }, // Mare Nubium
    LunarMare { lat: -15.0, lon: 35.0, r_lat: 12.0, r_lon: 14.0 }, // Mare Nectaris
    LunarMare { lat: 17.0, lon: 59.0, r_lat: 11.0, r_lon: 13.0 },  // Mare Crisium
    LunarMare { lat: -8.0, lon: 51.0, r_lat: 16.0, r_lon: 16.0 },  // Mare Fecunditatis
    LunarMare { lat: -24.0, lon: -39.0, r_lat: 11.0, r_lon: 12.0 },// Mare Humorum
    LunarMare { lat: 56.0, lon: 0.0, r_lat: 7.0, r_lon: 38.0 },    // Mare Frigoris
    LunarMare { lat: 13.0, lon: 4.0, r_lat: 8.0, r_lon: 9.0 },     // Mare Vaporum
    LunarMare { lat: 45.0, lon: -32.0, r_lat: 7.0, r_lon: 9.0 },   // Sinus Iridum
    LunarMare { lat: 27.3, lon: 147.9, r_lat: 8.0, r_lon: 10.0 },  // Mare Moscoviense (Far-side basalt sea)
];

struct LunarCrater {
    lat: f64,
    lon: f64,
    radius_deg: f64,
    has_rays: bool,
    ray_extent_deg: f64,
    is_dark_floor: bool,
    has_central_peak: bool,
    brightness: f64,
}

// Official IAU / NASA documented lunar craters (Near Side & Far Side)
const CRATERS: &[LunarCrater] = &[
    // --- Near Side: Massive Ray Craters ---
    LunarCrater { lat: -43.3, lon: -11.2, radius_deg: 2.8, has_rays: true, ray_extent_deg: 55.0, is_dark_floor: false, has_central_peak: true, brightness: 1.50 }, // Tycho
    LunarCrater { lat: 9.6, lon: -20.1, radius_deg: 3.1, has_rays: true, ray_extent_deg: 25.0, is_dark_floor: false, has_central_peak: true, brightness: 1.40 },  // Copernicus
    LunarCrater { lat: 8.1, lon: -38.0, radius_deg: 1.6, has_rays: true, ray_extent_deg: 16.0, is_dark_floor: false, has_central_peak: true, brightness: 1.35 },  // Kepler
    LunarCrater { lat: 23.7, lon: -47.4, radius_deg: 1.8, has_rays: true, ray_extent_deg: 12.0, is_dark_floor: false, has_central_peak: true, brightness: 1.65 }, // Aristarchus
    LunarCrater { lat: 16.1, lon: 46.8, radius_deg: 1.4, has_rays: true, ray_extent_deg: 15.0, is_dark_floor: false, has_central_peak: true, brightness: 1.50 },  // Proclus
    LunarCrater { lat: -8.9, lon: 61.1, radius_deg: 3.8, has_rays: true, ray_extent_deg: 14.0, is_dark_floor: false, has_central_peak: true, brightness: 1.35 },  // Langrenus
    LunarCrater { lat: -32.5, lon: 54.2, radius_deg: 2.4, has_rays: true, ray_extent_deg: 12.0, is_dark_floor: false, has_central_peak: true, brightness: 1.40 }, // Stevinus
    LunarCrater { lat: -1.9, lon: 47.6, radius_deg: 1.0, has_rays: true, ray_extent_deg: 12.0, is_dark_floor: false, has_central_peak: false, brightness: 1.40 }, // Messier

    // --- Near Side: Prominent Walled Plains & Dark-Floored Craters ---
    LunarCrater { lat: 51.6, lon: -9.3, radius_deg: 3.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.45 },   // Plato
    LunarCrater { lat: -5.2, lon: -68.6, radius_deg: 5.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.40 },  // Grimaldi
    LunarCrater { lat: 29.7, lon: -4.0, radius_deg: 2.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.60 },   // Archimedes
    LunarCrater { lat: -9.2, lon: -1.8, radius_deg: 4.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.65 },   // Ptolemaeus
    LunarCrater { lat: -44.4, lon: -54.6, radius_deg: 6.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.55 }, // Schickard
    LunarCrater { lat: 31.8, lon: 29.9, radius_deg: 3.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.65 },   // Posidonius

    // --- Near Side: Major Highland Craters ---
    LunarCrater { lat: -58.4, lon: -14.4, radius_deg: 6.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.15 },// Clavius
    LunarCrater { lat: 14.5, lon: -11.3, radius_deg: 2.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.25 }, // Eratosthenes
    LunarCrater { lat: -13.4, lon: -2.8, radius_deg: 3.4, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.10 }, // Alphonsus
    LunarCrater { lat: -18.2, lon: -1.9, radius_deg: 2.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.20 }, // Arzachel
    LunarCrater { lat: -11.4, lon: 26.4, radius_deg: 3.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.30 }, // Theophilus
    LunarCrater { lat: -13.2, lon: 24.0, radius_deg: 2.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.05 }, // Cyrillus
    LunarCrater { lat: -18.0, lon: 23.6, radius_deg: 3.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.00 },// Catharina
    LunarCrater { lat: -25.3, lon: 60.4, radius_deg: 4.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.15 }, // Petavius
    LunarCrater { lat: -17.5, lon: -39.9, radius_deg: 3.4, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.15 },// Gassendi
    LunarCrater { lat: -20.7, lon: -22.3, radius_deg: 2.1, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.25 },// Bullialdus
    LunarCrater { lat: 14.5, lon: 9.1, radius_deg: 1.6, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.35 },  // Manilius
    LunarCrater { lat: 16.3, lon: 16.0, radius_deg: 1.4, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.35 },  // Menelaus
    LunarCrater { lat: 21.8, lon: 17.9, radius_deg: 1.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.30 }, // Bessel
    LunarCrater { lat: -70.6, lon: -5.5, radius_deg: 3.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.25 }, // Moretus
    LunarCrater { lat: 27.7, lon: 55.5, radius_deg: 3.6, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: true, brightness: 0.65 },   // Cleomedes
    LunarCrater { lat: -21.2, lon: 33.0, radius_deg: 3.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.60 },  // Fracastorius
    LunarCrater { lat: -41.8, lon: 14.0, radius_deg: 3.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.15 }, // Maurolycus
    LunarCrater { lat: 63.5, lon: -63.0, radius_deg: 4.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.20 }, // Pythagoras

    // --- Far Side: Monumental Basins, Ray Systems & Documented Craters ---
    // South Pole-Aitken Basin (Largest impact basin on Moon, 2500 km across)
    LunarCrater { lat: -53.0, lon: 169.0, radius_deg: 35.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.65 },
    // Tsiolkovskiy (Dark basalt lake & gleaming white central peak)
    LunarCrater { lat: -20.4, lon: 129.1, radius_deg: 5.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: true, brightness: 0.35 },
    // Jackson (King of Far-Side Rays)
    LunarCrater { lat: 22.4, lon: -163.1, radius_deg: 2.1, has_rays: true, ray_extent_deg: 35.0, is_dark_floor: false, has_central_peak: true, brightness: 1.60 },
    // Giordano Bruno (Youngest crater on the Moon, ultra-bright ray system)
    LunarCrater { lat: 35.9, lon: 102.8, radius_deg: 0.8, has_rays: true, ray_extent_deg: 25.0, is_dark_floor: false, has_central_peak: true, brightness: 1.70 },
    // Ohm (Prominent far-side ray crater)
    LunarCrater { lat: 18.4, lon: -113.5, radius_deg: 1.8, has_rays: true, ray_extent_deg: 20.0, is_dark_floor: false, has_central_peak: true, brightness: 1.55 },
    // Hertzsprung (Giant 536 km multi-ring basin)
    LunarCrater { lat: 1.4, lon: -128.7, radius_deg: 15.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.15 },
    // Korolev (437 km double-ring basin)
    LunarCrater { lat: -4.2, lon: -157.4, radius_deg: 12.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.15 },
    // Apollo Basin (538 km multi-ring basin inside SPA)
    LunarCrater { lat: -36.1, lon: -151.8, radius_deg: 15.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.70 },
    // Gagarin (265 km walled plain)
    LunarCrater { lat: -19.7, lon: 149.3, radius_deg: 7.6, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.10 },
    // Mendeleev (313 km basin)
    LunarCrater { lat: 5.7, lon: 140.9, radius_deg: 9.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.75 },
    // Van de Graaff (Figure-8 crater with magnetic anomaly)
    LunarCrater { lat: -27.4, lon: 172.2, radius_deg: 6.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.10 },
    // Daedalus (Exact center of Far Side)
    LunarCrater { lat: -5.9, lon: 179.4, radius_deg: 2.8, has_rays: true, ray_extent_deg: 12.0, is_dark_floor: false, has_central_peak: true, brightness: 1.35 },
    // Von Kármán (180 km crater inside SPA, Chang'e 4 landing site)
    LunarCrater { lat: -44.5, lon: 175.8, radius_deg: 5.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: true, brightness: 0.55 },
    // Jules Verne (Dark lava-filled floor)
    LunarCrater { lat: -34.9, lon: 147.3, radius_deg: 4.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.50 },
    // Antoniadi (Deepest point on Moon inside SPA)
    LunarCrater { lat: -69.7, lon: -172.0, radius_deg: 4.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.25 },
    // Oppenheimer (208 km basin rim)
    LunarCrater { lat: -35.2, lon: -166.3, radius_deg: 6.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.10 },
    // Birkhoff (345 km northern basin)
    LunarCrater { lat: 58.7, lon: -146.1, radius_deg: 10.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.10 },
    // Milne (272 km walled plain)
    LunarCrater { lat: -31.0, lon: 112.8, radius_deg: 7.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.05 },
    // Poincaré (319 km basin)
    LunarCrater { lat: -56.7, lon: 163.6, radius_deg: 9.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.65 },
    // Zeeman (190 km crater with 4000m cliffs)
    LunarCrater { lat: -75.2, lon: -133.6, radius_deg: 5.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.20 },
    // Icarus (Huge central mountain peak)
    LunarCrater { lat: -5.5, lon: -173.2, radius_deg: 2.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.30 },
    // Pasteur (227 km walled plain)
    LunarCrater { lat: -11.9, lon: 104.6, radius_deg: 6.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.05 },
    // Keeler (170 km prominent crater)
    LunarCrater { lat: -8.9, lon: 161.9, radius_deg: 4.8, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.20 },
    // Heaviside (165 km crater)
    LunarCrater { lat: -10.4, lon: 167.1, radius_deg: 4.7, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.10 },
    // Chaplygin (123 km circular crater)
    LunarCrater { lat: -5.7, lon: 150.2, radius_deg: 3.6, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.15 },
    // Fizeau (169 km prominent crater)
    LunarCrater { lat: -58.2, lon: -133.9, radius_deg: 4.9, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.20 },
    // Leibnitz (245 km dark-floored crater)
    LunarCrater { lat: -38.3, lon: 179.2, radius_deg: 7.1, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: true, brightness: 0.60 },
    // Pavlov (148 km crater)
    LunarCrater { lat: -28.3, lon: 142.5, radius_deg: 4.3, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: true, brightness: 1.15 },
    // Schrödinger (312 km impact basin near South Pole)
    LunarCrater { lat: -75.0, lon: 132.4, radius_deg: 9.0, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: true, brightness: 0.60 },
    // Planck (314 km basin)
    LunarCrater { lat: -57.5, lon: 135.3, radius_deg: 9.1, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.05 },
    // Minkowski (113 km basalt floor)
    LunarCrater { lat: -56.5, lon: -146.0, radius_deg: 3.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.55 },
    // Landau (214 km basin)
    LunarCrater { lat: 41.6, lon: -119.4, radius_deg: 6.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.10 },
    // D'Alembert (248 km northern plain)
    LunarCrater { lat: 50.8, lon: 164.9, radius_deg: 7.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.05 },
    // Campbell (219 km dark floor)
    LunarCrater { lat: 45.3, lon: 165.3, radius_deg: 6.5, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: true, has_central_peak: false, brightness: 0.65 },
    // Fowler (146 km crater)
    LunarCrater { lat: 42.3, lon: -145.0, radius_deg: 4.2, has_rays: false, ray_extent_deg: 0.0, is_dark_floor: false, has_central_peak: false, brightness: 1.15 },
];

impl MoonShader {
    fn procedural_noise(x: f64, y: f64, z: f64) -> f64 {
        let c1 = (x * 12.0 + y * 9.0).sin() * (z * 11.5).cos();
        let c2 = (x * 28.0 - z * 24.0).sin() * (y * 27.0).cos() * 0.5;
        let c3 = (x * 65.0 + z * 60.0).sin() * (y * 62.0).cos() * 0.25;
        (c1 + c2 + c3) / 1.75
    }

    pub fn shade(local_hit: Vec3, world_norm: Vec3, sun_dir: Vec3) -> [u8; 3] {
        let lat_rad = local_hit.y.clamp(-1.0, 1.0).asin();
        let lat_deg = lat_rad * 180.0 / PI;
        let lon_rad = local_hit.z.atan2(local_hit.x);
        let lon_deg = lon_rad * 180.0 / PI;

        // 1. Lunar Maria (Dark Basalt Plains)
        let mut mare_weight = 0.0f64;
        for mare in MARIA {
            let dy = (lat_deg - mare.lat) / mare.r_lat;
            let dx = (lon_deg - mare.lon).abs().min(360.0 - (lon_deg - mare.lon).abs()) / mare.r_lon;
            let dist2 = dx * dx + dy * dy;
            if dist2 < 1.0 {
                let w = (1.0 - dist2).powf(1.6);
                mare_weight = mare_weight.max(w);
            }
        }

        // 2. High-Frequency Crater Field
        let base_crater_noise = Self::procedural_noise(local_hit.x, local_hit.y, local_hit.z);

        // Base albedo: Highlands are bright anorthosite (200), Maria are dark basalt (70)
        let highland_albedo = 198.0 + base_crater_noise * 22.0;
        let mare_albedo = 68.0 + base_crater_noise * 14.0;
        let mut albedo = highland_albedo * (1.0 - mare_weight) + mare_albedo * mare_weight;

        // 3. Documented Craters & Ray Systems
        let mut crater_rim_glow = 0.0f64;
        let mut ray_ejecta = 0.0f64;
        let mut floor_darkening = 1.0f64;

        for crater in CRATERS {
            let d_lat = lat_deg - crater.lat;
            let d_lon_raw = (lon_deg - crater.lon).abs().min(360.0 - (lon_deg - crater.lon).abs());
            // Account for spherical meridian convergence
            let cos_c_lat = (crater.lat * PI / 180.0).cos();
            let d_lon = d_lon_raw * cos_c_lat;
            let dist = (d_lat * d_lat + d_lon * d_lon).sqrt();

            // Inside crater structure
            if dist <= crater.radius_deg {
                let norm_d = dist / crater.radius_deg;
                if norm_d > 0.78 {
                    // Elevated reflective crater rim
                    let rim_intensity = ((norm_d - 0.78) / 0.22) * (1.0 - ((norm_d - 1.0) / 0.2).abs()).max(0.0);
                    crater_rim_glow = crater_rim_glow.max(rim_intensity * crater.brightness);
                } else if norm_d < 0.18 && crater.has_central_peak {
                    // Central peak (e.g. Tycho, Copernicus, Jackson, Tsiolkovskiy)
                    let peak = (1.0 - norm_d / 0.18).powf(2.0);
                    let peak_bright = if crater.is_dark_floor { 1.6 } else { crater.brightness };
                    crater_rim_glow = crater_rim_glow.max(peak * 0.9 * peak_bright);
                } else if crater.is_dark_floor {
                    // Dark basalt filled floor (e.g. Plato, Grimaldi, Tsiolkovskiy basalt sea)
                    let floor_factor = if crater.has_central_peak {
                        // Keep floor dark around central peak
                        ((norm_d - 0.16) / 0.62).clamp(0.0, 1.0) * (1.0 - ((norm_d - 0.78) / 0.1).max(0.0))
                    } else {
                        (1.0 - norm_d / 0.78).clamp(0.0, 1.0)
                    };
                    floor_darkening = floor_darkening.min(1.0 - floor_factor * (1.0 - crater.brightness));
                }
            }

            // Radial Ejecta Ray Systems (Tycho, Copernicus, Kepler, Proclus, Jackson, Giordano Bruno)
            if crater.has_rays && dist <= crater.ray_extent_deg && dist > crater.radius_deg * 0.7 {
                let angle = d_lat.atan2(d_lon);
                // 12-lobed ray filament structure
                let filament = (angle * 12.0).sin().powi(4);
                let dist_falloff = (1.0 - dist / crater.ray_extent_deg).powf(1.6);
                let ray_strength = filament * dist_falloff * 0.55 * crater.brightness;
                ray_ejecta = ray_ejecta.max(ray_strength);
            }
        }

        // Apply crater geology to albedo
        albedo *= floor_darkening;
        albedo += crater_rim_glow * 65.0;
        albedo += ray_ejecta * 75.0;
        albedo = albedo.clamp(35.0, 255.0);

        // 4. Photometric Illumination & Subtle Lunar Night Starlight
        let diffuse = world_norm.dot(sun_dir).max(0.0);
        let n_dot_l = world_norm.dot(sun_dir);

        // Smooth lunar terminator
        let sun_term = ((n_dot_l + 0.10) / 0.20).clamp(0.0, 1.0);
        let lit_factor = sun_term * (diffuse * 0.92 + 0.08) + (1.0 - sun_term) * 0.12;

        let final_lum = albedo * lit_factor;

        let r = final_lum.clamp(0.0, 255.0) as u8;
        let g = (final_lum * 0.98).clamp(0.0, 255.0) as u8;
        let b = (final_lum * 0.95).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}
