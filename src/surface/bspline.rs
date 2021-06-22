use crate::surface::{EdgeLoop, Surface};
use crate::{Float, Grid, KnotVector, Point3, Point4, TriangleMesh};

#[derive(Debug, Clone)]
pub struct BSplineSurface<P> {
    pub control_points: Grid<P>,
    pub knots: (KnotVector, KnotVector),
    pub degree: (u8, u8),
}

impl<P: Clone> BSplineSurface<P> {
    pub fn new(
        control_points: Grid<P>,
        knots: (KnotVector, KnotVector),
        degree: (usize, usize),
    ) -> Self {
        let (u_deg, v_deg) = degree;
        assert_eq!(control_points.rows() + u_deg + 1, knots.0.len());
        assert_eq!(control_points.cols() + v_deg + 1, knots.1.len());
        Self {
            control_points,
            knots,
            degree: (u_deg as u8, v_deg as u8),
        }
    }

    pub fn uniform_clamped(control_points: Grid<P>, degree: (usize, usize)) -> Self {
        let (u_deg, v_deg) = degree;
        assert!(control_points.rows() > u_deg && control_points.cols() > v_deg);

        let u_knots = KnotVector::uniform_knot(u_deg, control_points.rows() - u_deg);
        let v_knots = KnotVector::uniform_knot(v_deg, control_points.cols() - v_deg);
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
                point += basis_u[i] * basis_v[j] * p;
            }
        }
        point
    }

    fn trim(&self, _bounds: &[EdgeLoop]) -> TriangleMesh {
        let patch = crate::surface::SurfacePatch {
            surface: self.clone(),
            parameter_range: (self.knots.0.range(), self.knots.1.range()),
            parameter_division: (16, 16),
        };

        patch.get_points().into()
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
                point += basis_u[i] * basis_v[j] * p;
            }
        }
        (1.0 / point.w) * point.truncate()
    }
}
