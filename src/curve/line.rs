use super::Curve;
use crate::{Float, Point3, Vec3};

#[derive(Debug)]
pub struct Line {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Curve for Line {
    fn get_point(&self, u: Float) -> Point3 {
        self.origin + self.direction * u
    }
}
