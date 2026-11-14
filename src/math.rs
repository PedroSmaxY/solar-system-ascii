//! High-precision 64-bit vector and matrix mathematics for astronomical scales.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const X: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    #[inline(always)]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline(always)]
    pub fn dot(self, rhs: Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline(always)]
    pub fn cross(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[inline(always)]
    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    #[inline(always)]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    #[inline(always)]
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len > 1e-15 {
            self * (1.0 / len)
        } else {
            Vec3::ZERO
        }
    }

    #[inline(always)]
    pub fn lerp(self, rhs: Vec3, t: f64) -> Vec3 {
        self + (rhs - self) * t
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl AddAssign for Vec3 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl SubAssign for Vec3 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Vec3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, rhs: f64) -> Vec3 {
        Vec3 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: rhs.x * self, y: rhs.y * self, z: rhs.z * self }
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn div(self, rhs: f64) -> Vec3 {
        let inv = 1.0 / rhs;
        Vec3 { x: self.x * inv, y: self.y * inv, z: self.z * inv }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn neg(self) -> Vec3 {
        Vec3 { x: -self.x, y: -self.y, z: -self.z }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mat3 {
    pub m: [f64; 9],
}

impl Mat3 {
    pub const IDENTITY: Mat3 = Mat3 {
        m: [
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        ],
    };

    pub fn from_cols(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Mat3 {
            m: [
                c0.x, c1.x, c2.x,
                c0.y, c1.y, c2.y,
                c0.z, c1.z, c2.z,
            ],
        }
    }

    pub fn rotation_x(rad: f64) -> Self {
        let (s, c) = rad.sin_cos();
        Mat3 {
            m: [
                1.0, 0.0, 0.0,
                0.0,   c,  -s,
                0.0,   s,   c,
            ],
        }
    }

    pub fn rotation_y(rad: f64) -> Self {
        let (s, c) = rad.sin_cos();
        Mat3 {
            m: [
                  c, 0.0,   s,
                0.0, 1.0, 0.0,
                 -s, 0.0,   c,
            ],
        }
    }

    pub fn rotation_z(rad: f64) -> Self {
        let (s, c) = rad.sin_cos();
        Mat3 {
            m: [
                  c,  -s, 0.0,
                  s,   c, 0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline(always)]
    pub fn mul_vec(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.m[0] * v.x + self.m[1] * v.y + self.m[2] * v.z,
            y: self.m[3] * v.x + self.m[4] * v.y + self.m[5] * v.z,
            z: self.m[6] * v.x + self.m[7] * v.y + self.m[8] * v.z,
        }
    }

    pub fn mul_mat(self, rhs: Mat3) -> Mat3 {
        let mut out = [0.0; 9];
        for r in 0..3 {
            for c in 0..3 {
                out[r * 3 + c] = self.m[r * 3 + 0] * rhs.m[0 * 3 + c]
                               + self.m[r * 3 + 1] * rhs.m[1 * 3 + c]
                               + self.m[r * 3 + 2] * rhs.m[2 * 3 + c];
            }
        }
        Mat3 { m: out }
    }

    pub fn transpose(self) -> Mat3 {
        Mat3 {
            m: [
                self.m[0], self.m[3], self.m[6],
                self.m[1], self.m[4], self.m[7],
                self.m[2], self.m[5], self.m[8],
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    #[inline(always)]
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self {
            origin,
            dir: dir.normalize(),
        }
    }

    #[inline(always)]
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + self.dir * t
    }
}
