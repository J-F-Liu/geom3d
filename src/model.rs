use crate::surface::{Surface, SurfacePatch};

pub struct Model {
    pub surfaces: Vec<SurfacePatch<Box<dyn Surface>>>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            surfaces: Vec::new(),
        }
    }

    pub fn add_surface(&mut self, surface: SurfacePatch<Box<dyn Surface>>) {
        self.surfaces.push(surface);
    }

    pub fn save_as_stl<P: AsRef<std::path::Path>>(&self, filename: P) -> std::io::Result<()> {
        use std::io::Write;
        let path = std::path::Path::new(filename.as_ref());
        let name = path.file_stem().unwrap().to_string_lossy();
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::LineWriter::new(file);

        writeln!(writer, "solid {}", name)?;
        for surface in &self.surfaces {
            let mesh = surface.get_triangle_mesh();
            for triangle in mesh.triangles.chunks(3) {
                writeln!(writer, "  facet normal 0 0 0")?;
                writeln!(writer, "    outer loop")?;
                for index in triangle {
                    let point = &mesh.vertices[*index as usize];
                    writeln!(writer, "      vertex {} {} {}", point.x, point.y, point.z)?;
                }
                writeln!(writer, "    endloop")?;
                writeln!(writer, "  endfacet")?;
            }
        }
        writeln!(writer, "endsolid {}", name)?;
        Ok(())
    }

    pub fn save_as_obj<P: AsRef<std::path::Path>>(&self, filename: P) -> std::io::Result<()> {
        use std::io::Write;
        let file = std::fs::File::create(filename)?;
        let mut writer = std::io::LineWriter::new(file);

        let mut start = 1;
        for surface in &self.surfaces {
            let mesh = surface.get_triangle_mesh();
            for point in &mesh.vertices {
                writeln!(writer, "v {} {} {}", point.x, point.y, point.z)?;
            }
            for triangle in mesh.triangles.chunks(3) {
                writeln!(
                    writer,
                    "f {} {} {}",
                    start + triangle[0],
                    start + triangle[1],
                    start + triangle[2]
                )?;
            }
            start += mesh.vertices.len() as u32;
        }
        Ok(())
    }
}
