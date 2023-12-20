use crate::curve::{Curve, CurveSegment};
use crate::{utils, utils::Tolerance, Float, Grid, Point2, Point3, TriangleMesh, Vec3};
use downcast_rs::{impl_downcast, Downcast};

/// Parametric surface
pub trait Surface: std::fmt::Debug + Downcast {
    /// Get a point on the surface with parameters `(u,v)`
    fn get_point(&self, u: Float, v: Float) -> Point3;

    fn get_normals(&self, _params: &[Point2]) -> Vec<Vec3> {
        Vec::new()
    }

    /// Trim the surface with an edge loop
    fn trim(&self, _bounds: &[EdgeLoop]) -> TriangleMesh {
        TriangleMesh::new()
    }
}

impl_downcast!(Surface);

impl Surface for Box<dyn Surface> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        self.as_ref().get_point(u, v)
    }

    fn trim(&self, bounds: &[EdgeLoop]) -> TriangleMesh {
        self.as_ref().trim(bounds)
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
    pub fn get_points(&self, params: &[Point2]) -> Vec<Point3> {
        params
            .iter()
            .map(|p| self.surface.get_point(p.x, p.y))
            .collect::<Vec<Point3>>()
    }

    pub fn get_point_grid(&self) -> Grid<Point3> {
        let (u_range, v_range) = self.parameter_range;
        let (u_div, v_div) = self.parameter_division;
        let u_parameters = utils::uniform_divide(u_range, u_div);
        let v_parameters = utils::uniform_divide(v_range, v_div);
        let points = u_parameters
            .into_iter()
            .map(|u| {
                v_parameters
                    .iter()
                    .map(move |&v| self.surface.get_point(u, v))
            })
            .flatten()
            .collect::<Vec<Point3>>();
        Grid::from_vec(points, v_div + 1)
    }
}

pub struct EdgeLoop {
    /// The edges should form a closed loop.
    pub edges: Vec<CurveSegment<Box<dyn Curve>>>,
}

impl EdgeLoop {
    /// Approximate the edge loop with a polygon
    pub fn to_polygon(&self) -> Vec<Point3> {
        let mut vertices = Vec::new();
        for edge in &self.edges {
            vertices.extend(edge.get_points());
        }
        vertices.dedup_by(|a, b| a.distance_squared(*b).near(0.0));
        if vertices[0]
            .distance_squared(vertices[vertices.len() - 1])
            .near(0.0)
        {
            vertices.pop();
        }
        vertices
    }
}

pub struct TrimmedSurface<S> {
    pub surface: S,

    /// One counter-clockwise edge loop is the outer bound, other clockwise loops are inner holes.
    pub bounds: Vec<EdgeLoop>,
}

mod bezier;
mod bspline;
mod cylinder;
mod plane;
mod spin;
mod sweep;
mod umbrella;

pub use bezier::*;
pub use bspline::*;
pub use cylinder::*;
pub use plane::*;
pub use spin::*;
pub use sweep::*;
pub use umbrella::*;
