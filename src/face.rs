use crate::surface::{Surface, SurfacePatch, TrimmedSurface};
use crate::TriangleMesh;

/// A face can be representable by a triangle mesh.
pub trait Face {
    fn get_triangle_mesh(&self) -> TriangleMesh;
}

impl Face for TriangleMesh {
    fn get_triangle_mesh(&self) -> TriangleMesh {
        self.clone()
    }
}

impl<S: Surface> Face for SurfacePatch<S> {
    fn get_triangle_mesh(&self) -> TriangleMesh {
        self.get_points().into()
    }
}

impl<S: Surface> Face for TrimmedSurface<S> {
    fn get_triangle_mesh(&self) -> TriangleMesh {
        self.surface.trim(&self.edges)
    }
}
