use super::Surface;
use crate::{Float, Point3, Quat, Vec3};

#[derive(Debug)]
pub struct Cylinder {
    pub origin: Point3,
    pub axis: Vec3,
    /// Normalized direction for angle start
    pub ref_dir: Vec3,
    pub radius: Float,
}

impl Surface for Cylinder {
    fn get_point(&self, length: Float, angle: Float) -> Point3 {
        let rotation = Quat::from_axis_angle(self.axis, angle);
        self.origin + self.axis * length + rotation * self.ref_dir * self.radius
    }
}
