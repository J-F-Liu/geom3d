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

    /// Project the cylinder to a ring on z=0 plane to keep its periodic nature
    pub fn project_to_ring(&self, point: Point3, d: Float) -> Point2 {
        let vector = point - self.origin;
        let z = vector.dot(self.axis);
        let ratio = d / (d - z);
        let x = vector.dot(self.ref_dir) * ratio;
        let y = vector.dot(self.axis.cross(self.ref_dir)) * ratio;
        Point2::new(x, y)
    }

    pub fn generate_point_from_ring(&self, point: Point2, d: Float) -> Point3 {
        let ratio = self.radius / point.length();
        let x = self.ref_dir * (point.x * ratio);
        let y = self.axis.cross(self.ref_dir) * (point.y * ratio);
        let z = self.axis * (d * (1.0 - ratio));
        self.origin + x + y + z
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
        let mut vertices = bounds
            .iter()
            .map(|bound| {
                let polygon = bound.to_polygon();
                if polygon.len() > 0 {
                    end += polygon.len();
                    polygons.push(end);
                }
                polygon
            })
            .flatten()
            .collect::<Vec<_>>();
        let points: Vec<Point2> = vertices.iter().map(|v| self.project(*v)).collect();
        let (min_z, max_z) = utils::get_min_max_by_key(&points, |p| p.x);
        let d = (max_z - min_z) * 2.0;

        let mut points: Vec<Point2> = vertices
            .iter()
            .map(|v| self.project_to_ring(*v, d))
            .collect();
        let boundary_point_count = points.len();

        if polygons.len() == 2 {
            if utils::is_polygon_counter_clockwise(&points) {
                let (points, triangles) = utils::generate_triangular_mesh(&points, &polygons);
                vertices.extend(
                    points[boundary_point_count..]
                        .iter()
                        .map(|&p| self.generate_point_from_ring(p, d)),
                );

                return TriangleMesh {
                    vertices,
                    triangles,
                    normals: Vec::new(),
                }
                .reverse_winding_direction();
            } else {
                vertices.reverse();
                points.reverse();
                let (points, triangles) = utils::generate_triangular_mesh(&points, &polygons);
                vertices.extend(
                    points[boundary_point_count..]
                        .iter()
                        .map(|&p| self.generate_point_from_ring(p, d)),
                );

                return TriangleMesh {
                    vertices,
                    triangles,
                    normals: Vec::new(),
                };
            }
        } else if polygons.len() > 2 {
            // triangulate polygon with holes
            let (points, triangles) = utils::generate_triangular_mesh(&points, &polygons);
            vertices.extend(
                points[boundary_point_count..]
                    .iter()
                    .map(|p| self.get_point(p.x, p.y)),
            );
            return TriangleMesh {
                vertices,
                triangles,
                normals: Vec::new(),
            };
        } else {
            return TriangleMesh::new();
        }
    }
}

#[test]
fn test_cylinder_projection() {
    use crate::face::Face;
    use crate::utils::Tolerance;
    let cylinder = crate::surface::SurfacePatch {
        surface: Cylinder {
            origin: Point3::new(-1.899979869, -30.57252185, 54.63893742),
            axis: Vec3::X,
            ref_dir: Vec3::new(0.0, 0.568151535920372, -0.8229239528846649),
            radius: 3.0,
        },
        parameter_range: ((0.0, 10.0), (0.0, crate::consts::PI)),
        parameter_division: (16, 16),
    };
    let mesh = cylinder.get_triangle_mesh();
    mesh.save_as_obj("tmp/mesh.obj").unwrap();

    let grid = cylinder.get_point_grid();
    // look from inside, extract boundary points in ccw order
    // so that projected points are in ccw order
    let mut vertices = Vec::<Point3>::new();
    vertices.extend(grid.iter_row(0).rev());
    vertices.extend(grid.iter_col(0));
    vertices.extend(grid.iter_row(grid.rows() - 1));
    vertices.extend(grid.iter_col(grid.cols() - 1).rev());
    vertices.dedup_by(|a, b| a.distance_squared(*b).near(0.0));
    vertices.pop();

    let d = 20.0;
    let points = vertices
        .iter()
        .map(|&v| cylinder.surface.project_to_ring(v, d))
        .collect::<Vec<_>>();

    let (points, triangles) = utils::generate_triangular_mesh(&points, &[0, points.len()]);
    let ring = TriangleMesh {
        vertices: points.iter().map(|p| p.extend(0.0)).collect(),
        triangles,
        normals: Vec::new(),
    }
    .reverse_winding_direction();
    ring.save_as_obj("tmp/ring.obj").unwrap();

    let cylinder = TriangleMesh {
        vertices: points
            .iter()
            .map(|&p| cylinder.surface.generate_point_from_ring(p, d))
            .collect(),
        triangles: ring.triangles,
        normals: Vec::new(),
    };

    cylinder.save_as_obj("tmp/cylinder.obj").unwrap();

    for i in 0..vertices.len() {
        assert!((vertices[i] - cylinder.vertices[i]).length().near(0.0));
    }
}
