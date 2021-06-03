use crate::basis::create_parameters;
use crate::curve::{Curve, CurveSegment};
use crate::{Float, Grid, Point3, TriangleMesh};

/// Parametric surface
pub trait Surface: std::fmt::Debug {
    /// Get a point on the surface with parameters `(u,v)`
    fn get_point(&self, u: Float, v: Float) -> Point3;
}

impl Surface for Box<dyn Surface> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        self.as_ref().get_point(u, v)
    }
}

/// A surface patch is representable by a triangle mesh.
pub trait SurfacePatch {
    fn get_triangle_count(&self) -> usize;
    fn get_triangle_mesh(&self) -> TriangleMesh;
}

/// A piece of surface with natural boundaries defined by parameter ranges.
#[derive(Debug)]
pub struct BoundedSurface<S: Surface> {
    pub surface: S,
    /// (u_range, v_range)
    pub parameter_range: ((Float, Float), (Float, Float)),
    /// (u_division, v_division)
    pub parameter_division: (usize, usize),
}

impl<S: Surface> BoundedSurface<S> {
    /// Get sample points on the surface patch
    pub fn get_points(&self) -> Grid<Point3> {
        let (u_range, v_range) = self.parameter_range;
        let (u_div, v_div) = self.parameter_division;
        let u_parameters = create_parameters(u_range, u_div);
        let v_parameters = create_parameters(v_range, v_div);
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

impl<S: Surface> SurfacePatch for BoundedSurface<S> {
    fn get_triangle_count(&self) -> usize {
        let (u_div, v_div) = self.parameter_division;
        u_div * v_div * 2
    }

    fn get_triangle_mesh(&self) -> TriangleMesh {
        self.get_points().into()
    }
}

pub struct TrimmedSurface<S, C: Curve> {
    pub surface: S,

    /// The edges should form a closed loop.
    pub edges: Vec<CurveSegment<C>>,
    // To be implemented
    // pub holes: Vec<CurveSegment<C>>,
}

impl<S: SurfacePatch, C: Curve> SurfacePatch for TrimmedSurface<S, C> {
    fn get_triangle_count(&self) -> usize {
        unimplemented!()
    }

    fn get_triangle_mesh(&self) -> TriangleMesh {
        unimplemented!()
    }
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
