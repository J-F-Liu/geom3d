use crate::basis::create_parameters;
use crate::{Float, Point3};
use grid::Grid;

mod bezier;
pub use bezier::*;

pub trait Surface {
    /// Get a point on the surface with parameters `(u,v)`
    fn get_point(&self, u: Float, v: Float) -> Point3;
}

#[derive(Debug)]
pub struct SurfacePatch<S: Surface> {
    pub surface: S,
    pub parameter_range: ((Float, Float), (Float, Float)),
    pub parameter_division: (usize, usize),
}

impl<S: Surface> SurfacePatch<S> {
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
        Grid::from_vec(points, v_div + 1)
    }
}
