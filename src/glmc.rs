#![allow(dead_code)]
use glm::{Mat4, Vec3, Vec4};
#[macro_export]
macro_rules! vec3 {
    ($x: expr, $y: expr, $z: expr) => {
        Vec3 {
            x: $x,
            y: $y,
            z: $z,
        }
    };
}
#[macro_export]
macro_rules! vec4 {
    ($x: expr, $y: expr, $z: expr, $w: expr) => {
        Vec4 {
            x: $x,
            y: $y,
            z: $z,
            w: $w,
        }
    };
}
#[macro_export]
macro_rules! mat4 {
    ($c0: expr, $c1: expr, $c2: expr, $c3: expr) => {
        Mat4 {
            c0: $c0,
            c1: $c1,
            c2: $c2,
            c3: $c3,
        }
    };
}
pub const VEC3_UP: Vec3 = vec3!(0., 1., 0.);
pub const VEC4_ZERO: Vec4 = vec4!(0., 0., 0., 0.);
pub const MAT4_ZERO: Mat4 = mat4!(VEC4_ZERO, VEC4_ZERO, VEC4_ZERO, VEC4_ZERO);
pub const MAT4_ONE: Mat4 = mat4!(
    vec4!(1., 0., 0., 0.),
    vec4!(0., 1., 0., 0.),
    vec4!(0., 0., 1., 0.),
    vec4!(0., 0., 0., 1.)
);
