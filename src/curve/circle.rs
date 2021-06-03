use super::Curve;
use crate::{Float, Point3, Quat, Vec3};

#[derive(Debug)]
pub struct Circle {
    pub center: Point3,
    pub radius: Float,
    pub axis: Vec3,
    pub ref_dir: Vec3,
}

impl Curve for Circle {
    fn get_point(&self, angle: Float) -> Point3 {
        let rotation = Quat::from_axis_angle(self.axis, angle);
        self.center + rotation * self.ref_dir * self.radius
    }
}
