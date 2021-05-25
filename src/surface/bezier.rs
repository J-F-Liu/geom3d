use super::Surface;
use crate::basis::bernstein;
use crate::{Float, Point3, Point4};
use grid::Grid;

#[derive(Debug)]
pub struct BezierSurface<P> {
    pub control_points: Grid<P>,
}

impl<P: Clone> BezierSurface<P> {
    pub fn new(control_points: Grid<P>) -> Self {
        assert!(control_points.rows() > 1 && control_points.cols() > 1);
        Self { control_points }
    }
}

impl Surface for BezierSurface<Point3> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        let (n, m) = self.control_points.size();
        let basis_u = bernstein(m, u); // m cols
        let basis_v = bernstein(n, v); // n rows
        let mut point = Point3::ZERO;
        for i in 0..n {
            for j in 0..m {
                let p = self.control_points[i][j];
                point += basis_u[j] * basis_v[i] * p;
            }
        }
        point
    }
}

/// Rational bezier surface, point (x,y,z) with weight w is (wx,wy,wz,w)
impl Surface for BezierSurface<Point4> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        let (n, m) = self.control_points.size();
        let basis_u = bernstein(m, u); // m cols
        let basis_v = bernstein(n, v); // n rows
        let mut point = Point4::ZERO;
        for i in 0..n {
            for j in 0..m {
                let p = self.control_points[i][j];
                point += basis_u[j] * basis_v[i] * p;
            }
        }
        (1.0 / point.w) * point.truncate()
    }
}
