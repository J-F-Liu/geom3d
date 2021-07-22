use crate::{Grid, Point3, Vec3};

#[derive(Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<Point3>,
    pub normals: Vec<Vec3>,
    /// Indices of points forming triangle list
    pub triangles: Vec<u32>,
}

impl TriangleMesh {
    pub fn new() -> TriangleMesh {
        TriangleMesh {
            vertices: Vec::new(),
            normals: Vec::new(),
            triangles: Vec::new(),
        }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }

    pub fn reverse_winding_direction(self) -> TriangleMesh {
        let triangles = self
            .triangles
            .chunks(3)
            .map(|t| [t[2], t[1], t[0]])
            .flatten()
            .collect::<Vec<_>>();
        TriangleMesh {
            vertices: self.vertices,
            normals: self.normals,
            triangles,
        }
    }

    pub fn write_obj<W: std::io::Write>(
        &self,
        writer: &mut W,
        vertex_start: u32,
        normal_start: u32,
    ) -> std::io::Result<()> {
        for point in &self.vertices {
            writeln!(writer, "v {} {} {}", point.x, point.y, point.z)?;
        }

        for normal in &self.normals {
            writeln!(writer, "vn {} {} {}", normal.x, normal.y, normal.z)?;
        }

        // for line in &self.lines {
        //     writeln!(writer, "l {} {}", line.0 + 1, line.1 + 1)?;
        // }

        if self.normals.is_empty() {
            for triangle in self.triangles.chunks(3) {
                writeln!(
                    writer,
                    "f {} {} {}",
                    triangle[0] + vertex_start,
                    triangle[1] + vertex_start,
                    triangle[2] + vertex_start
                )?;
            }
        } else {
            for triangle in self.triangles.chunks(3) {
                writeln!(
                    writer,
                    "f {}//{} {}//{} {}//{}",
                    triangle[0] + vertex_start,
                    triangle[0] + normal_start,
                    triangle[1] + vertex_start,
                    triangle[1] + normal_start,
                    triangle[2] + vertex_start,
                    triangle[2] + normal_start
                )?;
            }
        }
        Ok(())
    }

    pub fn save_as_obj<P: AsRef<std::path::Path>>(&self, filename: P) -> std::io::Result<()> {
        let file = std::fs::File::create(filename)?;
        let mut writer = std::io::LineWriter::new(file);
        self.write_obj(&mut writer, 1, 1)
    }
}

pub fn create_triangles(grid: &Grid<Point3>) -> Vec<u32> {
    let (rows, cols) = grid.size();
    let mut triangles = Vec::with_capacity((rows - 1) * (cols - 1) * 6);
    for row in 0..rows - 1 {
        for col in 0..cols - 1 {
            // first triangle
            triangles.push((row * cols + col) as u32);
            triangles.push(((row + 1) * cols + col) as u32);
            triangles.push((row * cols + col + 1) as u32);
            // second triangle
            triangles.push((row * cols + col + 1) as u32);
            triangles.push(((row + 1) * cols + col) as u32);
            triangles.push(((row + 1) * cols + col + 1) as u32);
        }
    }
    triangles
}
