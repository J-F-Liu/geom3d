use crate::surface::{Surface, SurfacePatch, TrimmedSurface};
use crate::{utils, Grid, Point2, TriangleMesh};

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
        let (u_range, v_range) = self.parameter_range;
        let (u_div, v_div) = self.parameter_division;
        let u_parameters = utils::uniform_divide(u_range, u_div);
        let v_parameters = utils::uniform_divide(v_range, v_div);
        let params = u_parameters
            .into_iter()
            .map(|u| v_parameters.iter().map(move |&v| Point2::new(u, v)))
            .flatten()
            .collect::<Vec<Point2>>();
        let points = self.get_points(&params);
        let mut normals = self.surface.get_normals(&params);
        let grid = Grid::from_vec(points, v_div + 1);
        let triangles = crate::mesh::create_triangles(&grid);
        for i in 0..normals.len() {
            if normals[i].is_nan() {
                let row = i / grid.cols();
                let col = i % grid.cols();
                let mut neighbors = Vec::new();
                if row > 0 {
                    neighbors.push((row - 1, col));
                }
                if row < grid.rows() - 1 {
                    neighbors.push((row + 1, col));
                }
                if col > 0 {
                    neighbors.push((row, col - 1));
                }
                if col < grid.cols() - 1 {
                    neighbors.push((row, col + 1));
                }
                let normal = neighbors.into_iter().find_map(|(r, c)| {
                    let n = normals[r * grid.cols() + c];
                    if n.is_nan() {
                        None
                    } else {
                        Some(n)
                    }
                });
                if let Some(normal) = normal {
                    normals[i] = normal;
                }
            }
        }
        TriangleMesh {
            vertices: grid.into_vec(),
            normals,
            triangles,
        }
    }
}

impl<S: Surface> Face for TrimmedSurface<S> {
    fn get_triangle_mesh(&self) -> TriangleMesh {
        self.surface.trim(&self.bounds)
    }
}
