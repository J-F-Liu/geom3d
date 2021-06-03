use super::Curve;
use crate::basis::bernstein;
use crate::{Float, Point3, Point4};

#[derive(Debug)]
pub struct BezierCurve<P> {
    pub control_points: Vec<P>,
}

/// 3D bezier curve
impl Curve for BezierCurve<Point3> {
    fn get_point(&self, u: Float) -> Point3 {
        let mut point = Point3::ZERO;
        let basis = bernstein(self.control_points.len(), u);
        for (b, &p) in basis.into_iter().zip(self.control_points.iter()) {
            point += b * p;
        }
        point
    }

    fn project(&self, point: Point3) -> Float {
        unimplemented!()
    }
}

/// Rational bezier curve, point (x,y,z) with weight w is (wx,wy,wz,w)
impl Curve for BezierCurve<Point4> {
    fn get_point(&self, u: Float) -> Point3 {
        let mut point = Point4::ZERO;
        let basis = bernstein(self.control_points.len(), u);
        for (b, &p) in basis.into_iter().zip(self.control_points.iter()) {
            point += b * p;
        }
        (1.0 / point.w) * point.truncate()
    }

    fn project(&self, point: Point3) -> Float {
        unimplemented!()
    }
}
