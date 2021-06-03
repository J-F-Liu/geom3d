use super::Curve;
use crate::{Float, KnotVector, Point3, Point4};

#[derive(Debug)]
pub struct BSplineCurve<P> {
    pub control_points: Vec<P>,
    pub knots: KnotVector,
    pub degree: u8,
}

/// 3D BSpline curve
impl Curve for BSplineCurve<Point3> {
    fn get_point(&self, u: Float) -> Point3 {
        let mut point = Point3::ZERO;
        let basis = self.knots.bspline_basis(self.degree as usize, u);
        for (b, &p) in basis.into_iter().zip(self.control_points.iter()) {
            point += b * p;
        }
        point
    }
}

/// Rational BSpline curve, point (x,y,z) with weight w is (wx,wy,wz,w)
impl Curve for BSplineCurve<Point4> {
    fn get_point(&self, u: Float) -> Point3 {
        let mut point = Point4::ZERO;
        let basis = self.knots.bspline_basis(self.degree as usize, u);
        for (b, &p) in basis.into_iter().zip(self.control_points.iter()) {
            point += b * p;
        }
        (1.0 / point.w) * point.truncate()
    }
}
