use crate::basis::create_parameters;
use crate::{Float, Point3};

mod bezier;
pub use bezier::*;

pub trait Curve {
    /// Get a point on the curve with parameter `u`
    fn get_point(&self, u: Float) -> Point3;
}

#[derive(Debug)]
pub struct CurveSegment<C: Curve> {
    pub curve: C,
    pub parameter_range: (Float, Float),
    pub sample_count: usize,
}

impl<C: Curve> CurveSegment<C> {
    /// Get sample points on the curve segment
    pub fn get_points(&self) -> Vec<Point3> {
        let parameters = create_parameters(self.parameter_range, self.sample_count);
        parameters
            .into_iter()
            .map(|u| self.curve.get_point(u))
            .collect()
    }
}
