use crate::curve::{Curve, CurveSegment};
use crate::{utils, Float, Grid, Point3, TriangleMesh};

/// Parametric surface
pub trait Surface: std::fmt::Debug {
    /// Get a point on the surface with parameters `(u,v)`
    fn get_point(&self, u: Float, v: Float) -> Point3;

    /// Trim the surface with an edge loop
    fn trim(&self, _edges: &[CurveSegment<Box<dyn Curve>>]) -> TriangleMesh {
        TriangleMesh::new()
    }
}

impl Surface for Box<dyn Surface> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        self.as_ref().get_point(u, v)
    }

    fn trim(&self, edges: &[CurveSegment<Box<dyn Curve>>]) -> TriangleMesh {
        self.as_ref().trim(edges)
    }
}

/// A piece of surface with natural boundaries defined by parameter ranges.
#[derive(Debug)]
pub struct SurfacePatch<S: Surface> {
    pub surface: S,
    /// (u_range, v_range)
    pub parameter_range: ((Float, Float), (Float, Float)),
    /// (u_division, v_division)
    pub parameter_division: (usize, usize),
}

impl<S: Surface> SurfacePatch<S> {
    /// Get sample points on the surface patch
    pub fn get_points(&self) -> Grid<Point3> {
        let (u_range, v_range) = self.parameter_range;
        let (u_div, v_div) = self.parameter_division;
        let u_parameters = utils::uniform_divide(u_range, u_div);
        let v_parameters = utils::uniform_divide(v_range, v_div);
        let points = v_parameters
            .into_iter()
            .map(|v| {
                u_parameters
                    .iter()
                    .map(move |&u| self.surface.get_point(u, v))
            })
            .flatten()
            .collect::<Vec<Point3>>();
        Grid::from_vec(points, u_div + 1)
    }
}

pub struct TrimmedSurface<S> {
    pub surface: S,

    /// The edges should form a closed loop.
    pub edges: Vec<CurveSegment<Box<dyn Curve>>>,
    // To be implemented
    // pub holes: Vec<CurveSegment<C>>,
}

mod bezier;
mod bspline;
mod cylinder;
mod plane;
mod spin;
mod sweep;

pub use bezier::*;
pub use bspline::*;
pub use cylinder::*;
pub use plane::*;
pub use spin::*;
pub use sweep::*;
