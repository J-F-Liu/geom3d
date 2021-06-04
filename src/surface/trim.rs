use crate::curve::{Curve, CurveSegment};
// use crate::surface::{Surface, SurfacePatch};
use crate::{Face, TriangleMesh};

pub struct TrimmedSurface<S, C: Curve> {
    pub surface: S,

    /// The edges should form a closed loop.
    pub edges: Vec<CurveSegment<C>>,
    // To be implemented
    // pub holes: Vec<CurveSegment<C>>,
}

impl<S: Face, C: Curve> Face for TrimmedSurface<S, C> {
    fn get_triangle_count(&self) -> usize {
        unimplemented!()
    }

    fn get_triangle_mesh(&self) -> TriangleMesh {
        self.surface.get_triangle_mesh()
    }
}
