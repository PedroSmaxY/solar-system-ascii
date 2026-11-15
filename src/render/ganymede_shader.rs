//! Photorealistic Ganymede Surface Shader:
//! - High-resolution NASA Galileo/Voyager photographic albedo and topographic relief.
//! - Strong geomorphological dichotomy:
//!   * Dark ancient polygonal terrain (Galileo, Marius, Nicholson Regio, albedo ~0.35): ~[95, 88, 80]
//!   * Bright grooved tectonic terrain (Uruk Sulcus, Tiamat Sulcus, albedo ~0.55): ~[190, 185, 180]
//!   * Rayed impact craters with fresh ice ejecta (Osiris crater): bright ice highlights ~[225, 230, 240]
//! - Topographic bump mapping via elevation finite differences highlighting tectonic sulci and crater rims.
//! - Non-Lambertian Lommel-Seeliger scattering with opposition surge and pitch-black nightside.

use crate::math::Vec3;
use crate::render::jupiter_shader::{JUPITER_SYSTEM_DATA, PIXELS, TEX_HEIGHT, TEX_WIDTH};
use std::f64::consts::PI;

pub const GANYMEDE_ALBEDO_OFFSET: usize = 16 + 8 * PIXELS; // 4,194,320
pub const GANYMEDE_ELEVATION_OFFSET: usize = 16 + 9 * PIXELS; // 4,718,608

pub struct GanymedeShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl GanymedeShader {
    pub fn new() -> Self {
        assert_eq!(
            &JUPITER_SYSTEM_DATA[0..8],
            b"ASTRO_JP",
            "Invalid jupiter_system_data.bin header format"
        );
        let albedo_map =
            &JUPITER_SYSTEM_DATA[GANYMEDE_ALBEDO_OFFSET..GANYMEDE_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &JUPITER_SYSTEM_DATA[GANYMEDE_ELEVATION_OFFSET..GANYMEDE_ELEVATION_OFFSET + PIXELS];

        Self {
            albedo_map,
            elevation_map,
        }
    }

    /// Access the underlying zero-copy albedo map slice.
    #[inline(always)]
    pub fn albedo_map(&self) -> &'static [u8] {
        self.albedo_map
    }

    /// Access the underlying zero-copy elevation map slice.
    #[inline(always)]
    pub fn elevation_map(&self) -> &'static [u8] {
        self.elevation_map
    }

    /// Convert 3D local normal (with y as north polar axis) to spherical coordinates (u, v).
    #[inline(always)]
    pub fn sphere_coords(n: Vec3) -> (f64, f64) {
        let v = (1.0 - (n.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = (n.z.atan2(n.x) + PI) / (2.0 * PI);
        (u, v)
    }

    /// 4-texel fractional bilinear interpolation on cylindrical projection.
    #[inline(always)]
    pub fn sample_bilinear(data: &[u8], u: f64, v: f64) -> f64 {
        let u_wrap = u.rem_euclid(1.0);
        let v_clamp = v.clamp(0.0, 1.0);

        let fx = u_wrap * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamp * (TEX_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % TEX_WIDTH;
        let y1 = (y0 + 1).min(TEX_HEIGHT - 1);

        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;

        let p00 = data[y0 * TEX_WIDTH + x0] as f64;
        let p10 = data[y0 * TEX_WIDTH + x1] as f64;
        let p01 = data[y1 * TEX_WIDTH + x0] as f64;
        let p11 = data[y1 * TEX_WIDTH + x1] as f64;

        let top = p00 * (1.0 - tx) + p10 * tx;
        let bot = p01 * (1.0 - tx) + p11 * tx;
        top * (1.0 - ty) + bot * ty
    }

    /// Sample raw Ganymede albedo byte with bilinear interpolation.
    #[inline(always)]
    pub fn sample_albedo_raw(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.albedo_map, u, v)
    }

    /// Sample raw Ganymede elevation byte with bilinear interpolation.
    #[inline(always)]
    pub fn sample_elevation_raw(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.elevation_map, u, v)
    }

    /// Calibrated RGB color palette for Ganymede geomorphology:
    /// - Dark ancient polygonal terrain (Galileo, Marius, Nicholson Regio, albedo ~0.35): ~[95, 88, 80]
    /// - Bright grooved tectonic terrain (Uruk Sulcus, Tiamat Sulcus, albedo ~0.55): ~[190, 185, 180]
    /// - Rayed impact craters with fresh ice ejecta (Osiris crater): bright ice highlights ~[225, 230, 240]
    #[inline(always)]
    pub fn albedo_to_color(raw: f64) -> [f64; 3] {
        if raw <= 135.0 {
            let t = ((raw - 100.0) / 35.0).clamp(0.0, 1.0);
            [
                75.0 + 20.0 * t,
                70.0 + 18.0 * t,
                64.0 + 16.0 * t,
            ]
        } else if raw <= 190.0 {
            let t = (raw - 135.0) / 55.0;
            [
                95.0 + 95.0 * t,
                88.0 + 97.0 * t,
                80.0 + 100.0 * t,
            ]
        } else {
            let t = ((raw - 190.0) / 50.0).clamp(0.0, 1.0);
            [
                190.0 + 35.0 * t,
                185.0 + 45.0 * t,
                180.0 + 60.0 * t,
            ]
        }
    }

    /// Topographic bump mapping:
    /// - Construct singularity-free orthonormal tangent & bitangent vectors.
    /// - Compute finite differences `du` and `dv` in elevation.
    /// - Perturb surface normal to highlight tectonic ridges (sulci) and impact crater rims.
    pub fn surface_normal(&self, local_norm: Vec3) -> Vec3 {
        let n = if local_norm.length_squared() > 1e-12 {
            local_norm.normalize()
        } else {
            return Vec3::new(0.0, 1.0, 0.0);
        };

        let (u, v) = Self::sphere_coords(n);

        // Orthonormal tangent & bitangent without polar singularities
        let xz_len = (n.x * n.x + n.z * n.z).sqrt();
        let tangent = if xz_len > 1e-6 {
            Vec3::new(-n.z / xz_len, 0.0, n.x / xz_len)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };
        let bitangent = tangent.cross(n).normalize();

        // 1-texel step finite differences
        let du_step = 1.0 / TEX_WIDTH as f64;
        let dv_step = 1.0 / TEX_HEIGHT as f64;

        let h_east = Self::sample_bilinear(self.elevation_map, u + du_step, v);
        let h_west = Self::sample_bilinear(self.elevation_map, u - du_step, v);
        let h_north = Self::sample_bilinear(self.elevation_map, u, (v - dv_step).max(0.0));
        let h_south = Self::sample_bilinear(self.elevation_map, u, (v + dv_step).min(1.0));

        let du = (h_east - h_west) * 0.5;
        let dv = (h_north - h_south) * 0.5;

        let bump_scale = 0.005;
        (n - tangent * (du * bump_scale) - bitangent * (dv * bump_scale)).normalize()
    }

    /// Primary photorealistic shader for Ganymede surface point:
    /// - Topographic normal perturbation for tectonic sulci and crater rims.
    /// - Lommel-Seeliger non-Lambertian scattering: `ls = mu_0 / (mu_0 + mu)`.
    /// - Pitch-black nightside when `mu_0 <= 0.0`.
    /// - Opposition surge retroreflection when phase angle `alpha ~ 0`.
    /// - Final illumination = `(ls * surge * 1.9).clamp(0.0, 1.5)`.
    pub fn shade(&self, local_norm: Vec3, sun_dir: Vec3, view_dir: Vec3) -> [u8; 3] {
        if local_norm.length_squared() < 1e-12
            || sun_dir.length_squared() < 1e-12
            || view_dir.length_squared() < 1e-12
        {
            return [0, 0, 0];
        }

        let sun_norm = sun_dir.normalize();
        let view_norm = view_dir.normalize();
        let perturbed_norm = self.surface_normal(local_norm);

        let mu_0 = perturbed_norm.dot(sun_norm);
        let mu = perturbed_norm.dot(view_norm).max(0.001);

        // PITCH-BLACK nightside, zero light leakage!
        if mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        let ls = mu_0 / (mu_0 + mu);
        let alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0).acos();
        let surge = 1.0 + 0.42 * (1.0 - (alpha / 0.16).min(1.0));
        let final_illumination = (ls * surge * 1.9).clamp(0.0, 1.5);

        let (u, v) = Self::sphere_coords(local_norm.normalize());
        let raw_albedo = self.sample_albedo_raw(u, v);
        let base_rgb = Self::albedo_to_color(raw_albedo);

        let r = (base_rgb[0] * final_illumination).round().clamp(0.0, 255.0) as u8;
        let g = (base_rgb[1] * final_illumination).round().clamp(0.0, 255.0) as u8;
        let b = (base_rgb[2] * final_illumination).round().clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_boundaries() {
        assert_eq!(TEX_WIDTH, 1024);
        assert_eq!(TEX_HEIGHT, 512);
        assert_eq!(PIXELS, 524_288);
        assert_eq!(GANYMEDE_ALBEDO_OFFSET, 4_194_320);
        assert_eq!(GANYMEDE_ELEVATION_OFFSET, 4_718_608);
        assert_eq!(GANYMEDE_ALBEDO_OFFSET, 16 + 8 * PIXELS);
        assert_eq!(GANYMEDE_ELEVATION_OFFSET, 16 + 9 * PIXELS);

        let shader = GanymedeShader::new();
        assert_eq!(shader.albedo_map().len(), PIXELS);
        assert_eq!(shader.elevation_map().len(), PIXELS);
        assert!(GANYMEDE_ELEVATION_OFFSET + PIXELS <= JUPITER_SYSTEM_DATA.len());
    }

    #[test]
    fn test_grooved_terrain_bump_gradients() {
        let shader = GanymedeShader::new();

        // North and South pole singularity checks
        let north_pole = Vec3::new(0.0, 1.0, 0.0);
        let south_pole = Vec3::new(0.0, -1.0, 0.0);
        let n_norm = shader.surface_normal(north_pole);
        let s_norm = shader.surface_normal(south_pole);

        assert!(!n_norm.x.is_nan() && !n_norm.y.is_nan() && !n_norm.z.is_nan());
        assert!(!s_norm.x.is_nan() && !s_norm.y.is_nan() && !s_norm.z.is_nan());
        assert!((n_norm.length() - 1.0).abs() < 1e-4);
        assert!((s_norm.length() - 1.0).abs() < 1e-4);

        // Find a location with grooved tectonic relief and verify perturbation
        let mut found_groove_perturbation = false;
        for lat_step in 1..8 {
            let lat = (lat_step as f64 / 8.0 - 0.5) * PI * 0.8;
            for lon_step in 0..16 {
                let lon = (lon_step as f64 / 16.0) * 2.0 * PI - PI;
                let base = Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
                let perturbed = shader.surface_normal(base);

                assert!((perturbed.length() - 1.0).abs() < 1e-4);
                let diff = (perturbed - base).length();
                if diff > 1e-5 {
                    found_groove_perturbation = true;
                    break;
                }
            }
            if found_groove_perturbation {
                break;
            }
        }
        assert!(
            found_groove_perturbation,
            "Bump mapping should perturb normals across Ganymede's grooved terrain"
        );
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = GanymedeShader::new();
        let hit = Vec3::new(1.0, 0.0, 0.0);

        // Pitch-black nightside
        let opposing_sun = Vec3::new(-1.0, 0.0, 0.0);
        let view = Vec3::new(1.0, 0.0, 0.0);
        let night_rgb = shader.shade(hit, opposing_sun, view);
        assert_eq!(
            night_rgb,
            [0, 0, 0],
            "Nightside must be pitch-black [0, 0, 0]"
        );

        // Subsolar opposition illumination
        let subsolar_sun = Vec3::new(1.0, 0.0, 0.0);
        let day_rgb = shader.shade(hit, subsolar_sun, view);
        assert!(
            day_rgb[0] > 0 && day_rgb[1] > 0 && day_rgb[2] > 0,
            "Subsolar dayside must be illuminated, got {:?}",
            day_rgb
        );

        // Opposition surge verification: alpha = 0 vs large alpha
        let sun_angled = Vec3::new(0.7071, 0.0, 0.7071);
        let angled_rgb = shader.shade(hit, sun_angled, view);
        // Illumination at opposition should be higher than at 45 degree phase
        assert!(
            day_rgb[0] > angled_rgb[0],
            "Opposition surge should result in brighter reflection at zero phase angle"
        );
    }

    #[test]
    fn test_polygonal_dark_terrain_albedo() {
        // Exact knot checks
        let dark_knot = GanymedeShader::albedo_to_color(135.0);
        assert_eq!(dark_knot, [95.0, 88.0, 80.0]);

        let grooved_knot = GanymedeShader::albedo_to_color(190.0);
        assert_eq!(grooved_knot, [190.0, 185.0, 180.0]);

        let ice_knot = GanymedeShader::albedo_to_color(240.0);
        assert_eq!(ice_knot, [225.0, 230.0, 240.0]);

        let shader = GanymedeShader::new();
        // Sample dark ancient polygonal terrain in Galileo Regio (approx Lon ~ 130W, Lat ~ 35N)
        // Lon in [-180, 180]: -130 deg -> u ~ 0.1388, Lat 35N -> v ~ 0.305
        let galileo_u = 0.1388;
        let galileo_v = 0.3055;
        let raw = shader.sample_albedo_raw(galileo_u, galileo_v);
        assert!(
            raw < 165.0,
            "Galileo Regio raw albedo should be dark ancient terrain (< 165), got {}",
            raw
        );

        let galileo_color = GanymedeShader::albedo_to_color(raw);
        assert!(
            galileo_color[0] <= 140.0 && galileo_color[1] <= 135.0 && galileo_color[2] <= 130.0,
            "Galileo Regio color should be dark ancient polygonal tone: got {:?}",
            galileo_color
        );
    }
}
