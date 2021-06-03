use crate::surface::SurfacePatch;

pub struct Model<S: SurfacePatch> {
    pub surfaces: Vec<S>,
}

impl<S: SurfacePatch> Model<S> {
    pub fn new() -> Model<S> {
        Model {
            surfaces: Vec::new(),
        }
    }

    pub fn add_surface(&mut self, surface: S) {
        self.surfaces.push(surface);
    }

    pub fn save_as_stl<P: AsRef<std::path::Path>>(&self, filename: P) -> std::io::Result<()> {
        use std::io::Write;
        let path = std::path::Path::new(filename.as_ref());
        let name = path.file_stem().unwrap().to_string_lossy();
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        let mut header = Vec::with_capacity(80);
        writeln!(header, "Binary STL file\nName: {:57}", name)?;
        header.truncate(80);
        writer.write(&header)?;

        let triangle_count: usize = self
            .surfaces
            .iter()
            .map(|surface| surface.get_triangle_count())
            .sum();
        writer.write(&(triangle_count as u32).to_le_bytes())?;

        for surface in &self.surfaces {
            let mesh = surface.get_triangle_mesh();
            for triangle in mesh.triangles.chunks(3) {
                // normal
                writer.write(&0.0_f32.to_le_bytes())?;
                writer.write(&0.0_f32.to_le_bytes())?;
                writer.write(&0.0_f32.to_le_bytes())?;
                // vertices
                for index in triangle {
                    let point = &mesh.vertices[*index as usize];
                    writer.write(&(point.x as f32).to_le_bytes())?;
                    writer.write(&(point.y as f32).to_le_bytes())?;
                    writer.write(&(point.z as f32).to_le_bytes())?;
                }
                // attribute byte count
                writer.write(&[0u8, 0u8])?;
            }
        }
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

mod step_reader;
pub use step_reader::ModelReader as StepReader;
