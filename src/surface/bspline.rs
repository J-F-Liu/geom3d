use super::Surface;
use crate::{Float, KnotVector, Point3, Point4};
use grid::Grid;

pub struct BSplineSurface<P> {
    pub control_points: Grid<P>,
    pub knots: (KnotVector, KnotVector),
    pub degree: (u8, u8),
}

impl<P: Clone> BSplineSurface<P> {
    pub fn new(control_points: Grid<P>, degree: (usize, usize)) -> Self {
        let (u_deg, v_deg) = degree;
        assert!(control_points.rows() > v_deg && control_points.cols() > u_deg);

        let u_knots = KnotVector::uniform_knot(u_deg, control_points.cols() - u_deg);
        let v_knots = KnotVector::uniform_knot(v_deg, control_points.rows() - v_deg);
        Self {
            control_points,
            knots: (u_knots, v_knots),
            degree: (u_deg as u8, v_deg as u8),
        }
    }
}

/// 3D BSpline Surface
impl Surface for BSplineSurface<Point3> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        let (u_knots, v_knots) = &self.knots;
        let (p, q) = self.degree;
        let basis_u = u_knots.bspline_basis(p as usize, u);
        let basis_v = v_knots.bspline_basis(q as usize, v);
        let (n, m) = self.control_points.size();
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

/// Rational BSpline Surface, point (x,y,z) with weight w is (wx,wy,wz,w)
impl Surface for BSplineSurface<Point4> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        let (u_knots, v_knots) = &self.knots;
        let (p, q) = self.degree;
        let basis_u = u_knots.bspline_basis(p as usize, u);
        let basis_v = v_knots.bspline_basis(q as usize, v);
        let (n, m) = self.control_points.size();
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
