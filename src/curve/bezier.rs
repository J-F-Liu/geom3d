use super::Curve;
use crate::basis::bernstein;
use crate::{utils, Float, Point3, Point4};

#[derive(Debug)]
pub struct BezierCurve<P> {
    pub control_points: Vec<P>,
}

impl<P> BezierCurve<P> {
    pub fn degree(&self) -> usize {
        self.control_points.len() - 1
    }
}

impl<P: std::ops::Sub<Output = P> + std::ops::Mul<Float, Output = P> + Copy> BezierCurve<P> {
    // The derivative of an nth-degree Bezier curve is an (n - 1)th-degree Bezier curve
    pub fn derivative(&self) -> BezierCurve<P> {
        let n = self.degree() as Float;
        let control_points = self
            .control_points
            .windows(2)
            .map(|pair| (pair[1] - pair[0]) * n)
            .collect::<Vec<P>>();
        BezierCurve { control_points }
    }
}

/// bezier curve in 3D space
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
        if self.degree() == 1 {
            return crate::curve::Polyline::new(self.control_points.clone()).project(point);
        }
        let der1 = self.derivative();
        let der2 = der1.derivative();
        let parameters = utils::uniform_divide((0.0, 1.0), self.degree() * 4);
        utils::find_nearest_parameter(self, &der1, &der2, point, &parameters, 10)
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

    fn project(&self, _point: Point3) -> Float {
        unimplemented!()
    }
}
