use super::Curve;
use crate::{utils, Float, KnotVector, Point3, Point4};

#[derive(Debug)]
pub struct BSplineCurve<P> {
    pub control_points: Vec<P>,
    pub knots: KnotVector,
    pub degree: u8,
}

impl<P> BSplineCurve<P> {
    #[inline]
    pub fn degree(&self) -> usize {
        self.degree as usize
    }
}

impl<P: std::ops::Sub<Output = P> + std::ops::Mul<Float, Output = P> + Copy> BSplineCurve<P> {
    // The derivative of a pth-degree B-Spline curve is a (p - 1)th-degree B-Spline curve
    pub fn derivative(&self) -> BSplineCurve<P> {
        let p = self.degree as Float;
        let knots = self.knots.shrink();
        let control_points = self
            .control_points
            .windows(2)
            .zip(knots.spans(self.degree()).into_iter())
            .map(|(pair, span)| (pair[1] - pair[0]) * p * utils::inv_or_zero(span))
            .collect::<Vec<P>>();
        BSplineCurve {
            control_points,
            knots,
            degree: self.degree - 1,
        }
    }
}

/// 3D BSpline curve
impl Curve for BSplineCurve<Point3> {
    fn get_point(&self, u: Float) -> Point3 {
        let mut point = Point3::ZERO;
        let basis = self.knots.bspline_basis(self.degree(), u);
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
        let parameters = utils::uniform_divide((0.0, 1.0), self.control_points.len() * 4);
        utils::find_nearest_parameter(self, &der1, &der2, point, &parameters, 10)
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

    fn project(&self, _point: Point3) -> Float {
        unimplemented!()
    }
}
