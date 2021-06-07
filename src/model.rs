use crate::{CurveGroup, Face};

pub struct Model<F: Face> {
    pub faces: Vec<F>,
    pub curves: Vec<CurveGroup>,
}

impl<F: Face> Model<F> {
    pub fn new() -> Model<F> {
        Model {
            faces: Vec::new(),
            curves: Vec::new(),
        }
    }

    pub fn add_face(&mut self, face: F) {
        self.faces.push(face);
    }

    pub fn add_curve(&mut self, curve: CurveGroup) {
        self.curves.push(curve);
    }

    pub fn save_as_stl<P: AsRef<std::path::Path>>(&self, filename: P) -> std::io::Result<()> {
        use std::io::{Seek, SeekFrom, Write};
        let path = std::path::Path::new(filename.as_ref());
        let name = path.file_stem().unwrap().to_string_lossy();
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        let header_size = 80;
        let mut header = Vec::with_capacity(header_size);
        writeln!(header, "Binary STL file\nName: {:57}", name)?;
        header.truncate(header_size);
        writer.write(&header)?;

        let mut triangle_count: usize = 0;
        writer.write(&(triangle_count as u32).to_le_bytes())?;

        for face in &self.faces {
            let mesh = face.get_triangle_mesh();
            triangle_count += mesh.triangle_count();
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
        writer.seek(SeekFrom::Start(header_size as u64))?;
        dbg!(triangle_count);
        writer.write(&(triangle_count as u32).to_le_bytes())?;
        Ok(())
    }

    pub fn save_as_obj<P: AsRef<std::path::Path>>(&self, filename: P) -> std::io::Result<()> {
        use std::io::Write;
        let file = std::fs::File::create(filename)?;
        let mut writer = std::io::LineWriter::new(file);

        let mut start = 1;
        for face in &self.faces {
            let mesh = face.get_triangle_mesh();
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

    pub fn save_as_svg<P: AsRef<std::path::Path>>(&self, filename: P) -> std::io::Result<()> {
        use svg::{
            node::element::{Group, Path},
            node::Node,
            Document,
        };
        let (width, height) = (300.0, 300.0);
        let mut document = Document::new()
            .set("width", format!("{}mm", width))
            .set("height", format!("{}mm", height))
            .set("viewBox", (0, 0, width, height));
        let mut group = Group::new().set("transform", "matrix(1 0 0 -1 0 300)");

        for curve in &self.curves {
            let mut data = String::new();
            for segment in &curve.segments {
                let points = segment.get_points();
                for point in points {
                    if data.len() == 0 {
                        data.push_str(&format!("M {},{}", point.x, point.y));
                    } else {
                        data.push_str(&format!(" L {},{}", point.x, point.y));
                    }
                }
            }
            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("d", data);
            group.append(path);
        }
        document.append(group);
        svg::save(filename, &document)?;
        Ok(())
    }
}

mod step_reader;
pub use step_reader::ModelReader as StepReader;
