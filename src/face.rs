use crate::surface::{Surface, SurfacePatch};
use crate::TriangleMesh;

/// A face can be representable by a triangle mesh.
pub trait Face {
    fn get_triangle_count(&self) -> usize;
    fn get_triangle_mesh(&self) -> TriangleMesh;
}

impl<S: Surface> Face for SurfacePatch<S> {
    fn get_triangle_count(&self) -> usize {
        let (u_div, v_div) = self.parameter_division;
        u_div * v_div * 2
    }

    fn get_triangle_mesh(&self) -> TriangleMesh {
        self.get_points().into()
    }
}
