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

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}
impl Transform {
    pub fn new(pos: Vec3, rot: Vec3, scale: Vec3) -> Self {
        Self {
            position: pos,
            rotation: rot,
            scale,
        }
    }
}

pub fn model_mat_from(i: Transform) -> glm::Mat4 {
    use glm::ext::{rotate, scale, translate};
    let mut res = MAT4_ONE;
    res = translate(&res, i.position);
    res = rotate(&res, glm::radians(i.rotation.x), glm::vec3(1., 0., 0.));
    res = rotate(&res, glm::radians(i.rotation.y), glm::vec3(0., 1., 0.));
    res = rotate(&res, glm::radians(i.rotation.z), glm::vec3(0., 0., 1.));
    res = scale(&res, i.scale);
    res
}

pub struct ComputedMatrices {
    pub view: glm::Mat4,
    pub projection: glm::Mat4,
    pub right: glm::Vec3,
    pub front: glm::Vec3,
}
pub fn compute_matrices(
    position: glm::Vec3,
    rotation: glm::Vec2,
    fov: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> ComputedMatrices {
    let projection = glm::ext::perspective(fov, aspect_ratio, z_near, z_far);

    let (cx, sx) = (glm::cos(rotation.x), glm::sin(rotation.x));
    let (cy, sy) = (glm::cos(rotation.y), glm::sin(rotation.y));
    let direction = glm::vec3(cy * sx, sy, cy * cx);
    let right_angle = rotation.x - std::f32::consts::FRAC_PI_2;
    let right = glm::vec3(glm::sin(right_angle), 0.0, glm::cos(right_angle));
    let up = glm::cross(right, direction);
    let front = -glm::cross(right, glm::vec3(0.0, 1.0, 0.0));

    let view = glm::ext::look_at(position, position + direction, up);
    ComputedMatrices {
        view,
        projection,
        right,
        front,
    }
}
