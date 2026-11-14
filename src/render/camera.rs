//! 6-DOF Astronomical Camera with continuous logarithmic/exponential zoom.

use crate::math::{Ray, Vec3};
use crate::physics::body::BodyId;
use crate::render::buffer::RenderMode;
use std::f64::consts::PI;

pub struct Camera {
    pub target_body: BodyId,
    pub target_pos: Vec3,
    pub azimuth: f64,   // Radians
    pub elevation: f64, // Radians (-PI/2 .. PI/2)
    pub distance_km: f64,
    pub fov_y_rad: f64,
    pub saved_distances: [f64; 3], // [Sun, Earth, Moon]

    // Free flight mode
    pub free_flight: bool,
    pub free_pos: Vec3,
    pub free_yaw: f64,
    pub free_pitch: f64,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            target_body: BodyId::Earth,
            target_pos: Vec3::ZERO,
            azimuth: 2.30,        // Framing both day lit hemisphere and night city lights
            elevation: 0.28,
            distance_km: 21_000.0, // Initial balanced view of Earth
            fov_y_rad: 45.0 * PI / 180.0,
            saved_distances: [3_500_000.0, 21_000.0, 8_000.0],
            free_flight: false,
            free_pos: Vec3::ZERO,
            free_yaw: 0.0,
            free_pitch: 0.0,
        }
    }

    pub fn zoom_in(&mut self) {
        // Logarithmic zoom: decrease distance by 12%
        let min_dist = match self.target_body {
            BodyId::Earth => 6_800.0, // ~430 km above surface (ISS orbit)
            BodyId::Moon => 1_900.0,
            BodyId::Sun => 750_000.0,
        };
        self.distance_km = (self.distance_km * 0.88).max(min_dist);
        self.saved_distances[self.target_body.index()] = self.distance_km;
    }

    pub fn zoom_out(&mut self) {
        // Logarithmic zoom: increase distance by 12% up to deep space
        self.distance_km = (self.distance_km * 1.14).min(5e9);
        self.saved_distances[self.target_body.index()] = self.distance_km;
    }

    pub fn orbit(&mut self, d_az: f64, d_el: f64) {
        self.azimuth = (self.azimuth + d_az).rem_euclid(2.0 * PI);
        self.elevation = (self.elevation + d_el).rem_euclid(2.0 * PI);
    }

    pub fn recenter(&mut self) {
        self.azimuth = 2.30;
        self.elevation = 0.28;
        self.distance_km = self.saved_distances[self.target_body.index()];
    }

    pub fn set_target(&mut self, body: BodyId, body_pos: Vec3) {
        // Save current distance for the body being left
        self.saved_distances[self.target_body.index()] = self.distance_km;

        self.target_body = body;
        self.target_pos = body_pos;

        // Restore target's dedicated viewing distance
        self.distance_km = self.saved_distances[body.index()];
    }

    /// Compute camera position in world space.
    pub fn position(&self) -> Vec3 {
        if self.free_flight {
            return self.free_pos;
        }
        let (sin_az, cos_az) = self.azimuth.sin_cos();
        let (sin_el, cos_el) = self.elevation.sin_cos();

        let offset = Vec3::new(
            self.distance_km * cos_el * sin_az,
            self.distance_km * sin_el,
            self.distance_km * cos_el * cos_az,
        );
        self.target_pos + offset
    }

    /// Generate ray for terminal pixel (u, v) in a buffer of size (cols, v_height).
    #[inline(always)]
    pub fn generate_ray(
        &self,
        u: usize,
        v: usize,
        cols: usize,
        v_height: usize,
        mode: RenderMode,
    ) -> Ray {
        let cam_pos = self.position();
        let (forward, right, up) = if self.free_flight {
            let (sy, cy) = self.free_yaw.sin_cos();
            let (sp, cp) = self.free_pitch.sin_cos();
            let f = Vec3::new(cp * sy, sp, cp * cy).normalize();
            let r = f.cross(Vec3::Y).normalize();
            let u = r.cross(f).normalize();
            (f, r, u)
        } else {
            let (sin_az, cos_az) = self.azimuth.sin_cos();
            let (sin_el, cos_el) = self.elevation.sin_cos();
            let f = Vec3::new(-cos_el * sin_az, -sin_el, -cos_el * cos_az);
            let r = Vec3::new(cos_az, 0.0, -sin_az);
            let u = Vec3::new(-sin_el * sin_az, cos_el, -sin_el * cos_az);
            (f, r, u)
        };

        // Terminal pixel aspect ratio correction:
        // In ASCII, 1 char is ~2x taller than wide.
        // In HalfBlock, 1 pixel is ~square (1.0).
        let pixel_aspect = match mode {
            RenderMode::Ascii => 2.15,
            RenderMode::HalfBlock => 1.05,
        };

        let screen_aspect = (cols as f64) / (v_height as f64) / pixel_aspect;

        let tan_half_fov_y = (self.fov_y_rad * 0.5).tan();
        let tan_half_fov_x = tan_half_fov_y * screen_aspect;

        let ndc_x = (2.0 * (u as f64 + 0.5) / (cols as f64) - 1.0) * tan_half_fov_x;
        let ndc_y = (1.0 - 2.0 * (v as f64 + 0.5) / (v_height as f64)) * tan_half_fov_y;

        let ray_dir = (forward + right * ndc_x + up * ndc_y).normalize();
        Ray::new(cam_pos, ray_dir)
    }
}
