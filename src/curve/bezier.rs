use super::Curve;
use crate::basis::bernstein;
use crate::{Float, Point3, Point4};

pub struct BezierCurve<P> {
    pub control_points: Vec<P>,
}

impl Curve for BezierCurve<Point3> {
    fn get_point(&self, u: Float) -> Point3 {
        let mut point = Point3::ZERO;
        let basis = bernstein(self.control_points.len(), u);
        for (b, &p) in basis.into_iter().zip(self.control_points.iter()) {
            point += b * p;
        }
        point
    }
}

impl Curve for BezierCurve<Point4> {
    fn get_point(&self, u: Float) -> Point3 {
        let mut point = Point4::ZERO;
        let basis = bernstein(self.control_points.len(), u);
        for (b, &p) in basis.into_iter().zip(self.control_points.iter()) {
            point += b * p;
        }
        (1.0 / point.w) * point.truncate()
    }
}
