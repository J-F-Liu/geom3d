use crate::{utils, Float, Point3};
use downcast_rs::{impl_downcast, Downcast};

/// Parametric curve
pub trait Curve: std::fmt::Debug + Downcast {
    /// Get a point on the curve with parameter `u`
    fn get_point(&self, u: Float) -> Point3;

    /// Get parameter of nearest point on the curve to the given point
    fn project(&self, point: Point3) -> Float;

    /// Refine parameter range according to whether same sense or not
    fn refine_parameter_range(&self, range: (Float, Float), _same_sense: bool) -> (Float, Float) {
        range
    }
}

impl_downcast!(Curve);

impl Curve for Box<dyn Curve> {
    fn get_point(&self, u: Float) -> Point3 {
        self.as_ref().get_point(u)
    }
    fn project(&self, point: Point3) -> Float {
        self.as_ref().project(point)
    }
    fn refine_parameter_range(&self, range: (Float, Float), same_sense: bool) -> (Float, Float) {
        self.as_ref().refine_parameter_range(range, same_sense)
    }
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
        let parameters = utils::uniform_divide(self.parameter_range, self.parameter_division);
        parameters
            .into_iter()
            .map(|u| self.curve.get_point(u))
            .collect()
    }
}

/// A continuous curve made up of a series of segments.
pub struct Polycurve {
    pub segments: Vec<CurveSegment<Box<dyn Curve>>>,
}

mod bezier;
mod bspline;
mod circle;
mod line;
mod polyline;
pub use bezier::*;
pub use bspline::*;
pub use circle::*;
pub use line::*;
pub use polyline::*;
