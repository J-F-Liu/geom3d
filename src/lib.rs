pub type Float = f64;
pub type Vec3 = glam::DVec3;
pub type Vec4 = glam::DVec4;
pub type Point3 = Vec3;
pub type Point4 = Vec4;
pub type Quat = glam::DQuat;
pub use std::f64::consts;

mod basis;
pub mod curve;
mod knot;
mod mesh;
pub mod model;
pub mod surface;

pub use grid::{grid, Grid};
pub use knot::KnotVector;
pub use mesh::TriangleMesh;
pub use model::Model;
