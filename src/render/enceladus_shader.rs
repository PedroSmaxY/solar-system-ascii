//! Enceladus Raytracing Shader Stub
use crate::math::Vec3;

pub struct EnceladusShader;

impl EnceladusShader {
    pub fn new() -> Self {
        Self
    }

    pub fn shade(&self, _local_norm: Vec3, _local_sun: Vec3, _local_view: Vec3) -> [u8; 3] {
        [245, 250, 255]
    }
}
