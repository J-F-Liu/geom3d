use super::Surface;
use crate::{Float, Point3, Vec3};

#[derive(Debug)]
pub struct Plane {
    pub origin: Point3,
    pub normal: Vec3,
    pub u_axis: Vec3,
    pub v_axis: Vec3,
}

impl Surface for Plane {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        self.origin + self.u_axis * u + self.v_axis * v
    }
}
