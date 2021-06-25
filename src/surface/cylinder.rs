use crate::surface::{EdgeLoop, Surface};
use crate::utils;
use crate::{Float, Point2, Point3, Quat, TriangleMesh, Vec3};

#[derive(Debug)]
pub struct Cylinder {
    pub origin: Point3,
    pub axis: Vec3,
    /// Normalized direction for angle start
    pub ref_dir: Vec3,
    pub radius: Float,
}

impl Cylinder {
    pub fn project(&self, point: Point3) -> Point2 {
        let vector = point - self.origin;
        let length = vector.dot(self.axis);
        let x = vector.dot(self.ref_dir);
        let y = vector.dot(self.axis.cross(self.ref_dir));
        let angle = y.atan2(x); // -π﹤angle ⩽ π
        let angle = if angle >= 0.0 {
            angle
        } else {
            angle + crate::consts::TAU
        };
        Point2::new(length, angle)
    }
}

impl Surface for Cylinder {
    fn get_point(&self, length: Float, angle: Float) -> Point3 {
        let rotation = Quat::from_axis_angle(self.axis, angle);
        self.origin + self.axis * length + rotation * self.ref_dir * self.radius
    }

    fn trim(&self, bounds: &[EdgeLoop]) -> TriangleMesh {
        let mut polygons = Vec::with_capacity(bounds.len() + 1);
        polygons.push(0);
        let mut end = 0;
        let vertices = bounds
            .iter()
            .map(|bound| {
                let polygon = bound.to_polygon();
                end += polygon.len();
                polygons.push(end);
                polygon
            })
            .flatten()
            .collect::<Vec<_>>();
        let points: Vec<Point2> = vertices.iter().map(|v| self.project(*v)).collect();

        if bounds.len() == 1 {
            let (vertex_indices, concave_points) = utils::compute_vertex_convexity(&points);

            if vertex_indices.len() == concave_points.len() || vertex_indices.len() < 3 {
                // clockwise polygon
                // dbg!(points.len(), vertex_indices.len(), concave_points.len());
                return TriangleMesh::new();
            }

            let triangles = utils::trianglate_polygon(&points, vertex_indices, concave_points);

            TriangleMesh {
                vertices,
                triangles,
            }
        } else {
            // triangulate polygon with holes
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
