use crate::surface::{EdgeLoop, Surface};
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

    fn trim(&self, bounds: &[EdgeLoop]) -> TriangleMesh {
        let mut polygons = Vec::with_capacity(bounds.len() + 1);
        polygons.push(0);
        let mut end = 0;
        let mut vertices = bounds
            .iter()
            .map(|bound| {
                let polygon = bound.generate_polygon();
                end += polygon.len();
                polygons.push(end);
                polygon
            })
            .flatten()
            .collect::<Vec<_>>();
        let points: Vec<Point2> = vertices.iter().map(|v| self.project(*v)).collect();

        if bounds.len() == 1 {
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
        } else {
            let mut vertex_indices = utils::merge_polygons(&points, &polygons);
            let concave_points = utils::find_concave_vertices(&points, &mut vertex_indices);
            let triangles = utils::trianglate_polygon(&points, vertex_indices, concave_points);

            TriangleMesh {
                vertices,
                triangles,
            }
        }
    }
}
