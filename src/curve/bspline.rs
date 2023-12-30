use super::Curve;
use crate::{utils, utils::Tolerance, Float, KnotVector, Point3, Point4};

#[derive(Debug, Clone)]
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

impl BSplineCurve<Point3> {
    pub fn add_knot(&mut self, knot: f64) {
        let p = self.degree();
        let n = self.control_points.len();

        let index = self.knots.add_knot(knot);
        if index == 0 {
            self.control_points.insert(0, Point3::ZERO);
        } else {
            let start = if index > p { index - p } else { 0 };
            let end = if index > n {
                self.control_points.push(Point3::ZERO);
                n + 1
            } else {
                self.control_points
                    .insert(index - 1, self.control_points[index - 1].clone());
                index
            };
            for i in (start..end).rev() {
                let delta = self.knots[i + p + 1] - self.knots[i];
                let a = (knot - self.knots[i]) * utils::inv_or_zero(delta);
                self.control_points[i] =
                    self.control_points[i - 1] * (1.0 - a) + self.control_points[i] * a;
            }
        }
    }

    pub fn split(&mut self, mut t: f64) -> BSplineCurve<Point3> {
        let p = self.degree();
        let index = self.knots.span_index(t);
        let s = if t.near(self.knots[index]) {
            t = self.knots[index];
            self.knots.multiplicity(index)
        } else {
            0
        };

        for _ in s..=p {
            self.add_knot(t);
        }

        let k = self.knots.span_index(t);
        let m = self.knots.len();
        let n = self.control_points.len();
        let knots0 = self.knots.sub_vec(0..=k);
        let knots1 = self.knots.sub_vec((k - p)..m);
        let control_points0 = Vec::from(&self.control_points[0..(k - p)]);
        let control_points1 = Vec::from(&self.control_points[(k - p)..n]);
        *self = BSplineCurve {
            knots: knots0,
            control_points: control_points0,
            degree: self.degree,
        };
        BSplineCurve {
            knots: knots1,
            control_points: control_points1,
            degree: self.degree,
        }
    }

    pub fn clamp(&mut self) {
        let degree = self.degree();

        let s = self.knots.multiplicity(0);
        for _ in s..=degree {
            self.add_knot(self.knots[0]);
        }

        let n = self.knots.len();
        let s = self.knots.multiplicity(n - 1);
        for _ in s..=degree {
            self.add_knot(self.knots[n - 1]);
        }
    }

    pub fn to_piecewise_bezier(&self) -> Vec<super::BezierCurve<Point3>> {
        let mut bspline = self.clone();
        bspline.clamp();

        let mut knots = self.knots.0.clone();
        knots.dedup_by(|a, b| a.near(*b));
        let n = knots.len();

        let mut result = Vec::with_capacity(n - 1);
        for i in 2..n {
            let piece = bspline.split(knots[n - i]);
            result.push(super::BezierCurve {
                control_points: piece.control_points,
            });
        }
        result.push(super::BezierCurve {
            control_points: bspline.control_points,
        });
        result.reverse();
        result
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
            let ratio = crate::curve::Polyline::new(self.control_points.clone()).project(point);
            return utils::range_at(self.knots.range(), ratio);
        }
        let der1 = self.derivative();
        let der2 = der1.derivative();
        utils::find_nearest_parameter(
            self,
            &der1,
            &der2,
            point,
            self.knots.range(),
            self.control_points.len() * 4,
            10,
        )
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

#[test]
fn test_bspline_curve() {
    let bspline = BSplineCurve {
        control_points: vec![
            Point3::new(17.14110848, -28.75946383, 55.24049254),
            Point3::new(17.71764285, -29.48891243, 54.73687694),
        ],
        knots: KnotVector::from_values_and_multiplicities(
            vec![2.802464183, 3.859874137],
            vec![2, 2],
        ),
        degree: 1,
    };
    let a = bspline.project(Point3::new(17.71764285, -29.48891243, 54.73687694));
    let b = bspline.project(Point3::new(17.14110848, -28.75946383, 55.24049254));
    dbg!(a, b);
    let segment = crate::curve::CurveSegment {
        curve: bspline,
        parameter_range: (a, b),
        tolerance: 0.001,
        parameter_division: 16,
    };
    dbg!(segment.get_points());
}
