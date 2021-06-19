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
        let (vertex_indices, concave_points) = utils::compute_vertex_convexity(&points);

        if vertex_indices.len() == concave_points.len() {
            // clockwise polygon
            // dbg!(concave_points.len());
            return TriangleMesh::new();
        }

        let triangles = if concave_points.is_empty() {
            // convex polygon
            let n = vertices.len();
            let center = vertices.iter().sum::<Point3>() / (n as Float);
            vertices.push(center);
            let mut triangles = Vec::with_capacity(n * 3);
            for i in 0..n - 1 {
                triangles.push(n as u32);
                triangles.push(i as u32);
                triangles.push(i as u32 + 1);
            }
            triangles.push(n as u32);
            triangles.push(n as u32 - 1);
            triangles.push(0);
            triangles
        } else {
            // polygon has both convex and concave vertices
            utils::trianglate_polygon(&points, vertex_indices, concave_points)
        };

        TriangleMesh {
            vertices,
            triangles,
        }
    }
}
