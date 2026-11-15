//! Iapetus Raytracing Shader Stub
use crate::math::Vec3;

pub struct IapetusShader;

impl IapetusShader {
    pub fn new() -> Self {
        Self
    }

    pub fn shade(&self, _local_norm: Vec3, _local_sun: Vec3, _local_view: Vec3) -> [u8; 3] {
        [170, 160, 150]
    }
}
