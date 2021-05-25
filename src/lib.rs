pub type Float = f64;
pub type Vec3 = glam::DVec3;
pub type Vec4 = glam::DVec4;
pub type Point3 = Vec3;
pub type Point4 = Vec4;

mod basis;
pub mod curve;
mod mesh;
mod model;
pub mod surface;

pub use grid::{grid, Grid};
pub use mesh::TriangleMesh;
pub use model::Model;
