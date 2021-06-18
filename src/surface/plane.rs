use super::Surface;
use crate::curve::{Curve, CurveSegment};
use crate::utils;
use crate::{Float, Point2, Point3, TriangleMesh, Vec3};

#[derive(Debug)]
pub struct Plane {
    pub origin: Point3,
    pub normal: Vec3,
    pub u_axis: Vec3,
    pub v_axis: Vec3,
}

impl Plane {
    pub fn project(&self, point: Point3) -> Point2 {
        let vector = point - self.origin;
        Point2::new(vector.dot(self.u_axis), vector.dot(self.v_axis))
    }
}

impl Surface for Plane {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        self.origin + self.u_axis * u + self.v_axis * v
    }

    fn trim(&self, edges: &[CurveSegment<Box<dyn Curve>>]) -> TriangleMesh {
        let mut vertices = Vec::new();
        for edge in edges {
            vertices.extend(edge.get_points());
        }

        let points: Vec<Point2> = vertices.iter().map(|v| self.project(*v)).collect();

        let triangles = utils::trianglate_polygon(&points);

        TriangleMesh {
            vertices,
            triangles,
        }
    }
}
