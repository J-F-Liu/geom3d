pub type Float = f64;
pub type Vec2 = glam::DVec2;
pub type Vec3 = glam::DVec3;
pub type Vec4 = glam::DVec4;
pub type Point2 = Vec2;
pub type Point3 = Vec3;
pub type Point4 = Vec4;
pub type Quat = glam::DQuat;

mod basis;
pub mod curve;
mod face;
mod knot;
mod mesh;
pub mod model;
pub mod surface;
pub mod utils;

pub use curve::Polycurve;
pub use face::Face;
pub use grid::{grid, Grid};
pub use knot::KnotVector;
pub use mesh::TriangleMesh;
pub use model::Model;

pub mod consts {
    use crate::Float;
    pub use std::f64::consts::*;
    pub const HALF_SQRT_3: Float = 1.7320508075689 / 2.0;
}
