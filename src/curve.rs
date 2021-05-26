use crate::basis::create_parameters;
use crate::{Float, Point3};

mod bezier;
mod bspline;
pub use bezier::*;
pub use bspline::*;

pub trait Curve {
    /// Get a point on the curve with parameter `u`
    fn get_point(&self, u: Float) -> Point3;
}

#[derive(Debug)]
pub struct CurveSegment<C: Curve> {
    pub curve: C,
    pub parameter_range: (Float, Float),
    pub parameter_division: usize,
}

impl<C: Curve> CurveSegment<C> {
    /// Get sample points on the curve segment
    pub fn get_points(&self) -> Vec<Point3> {
        let parameters = create_parameters(self.parameter_range, self.parameter_division);
        parameters
            .into_iter()
            .map(|u| self.curve.get_point(u))
            .collect()
    }
}
