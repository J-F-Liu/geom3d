use crate::Point3;
use crate::{curve, CurveGroup, Face};

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

    pub fn save_as_svg<P: AsRef<std::path::Path>>(
        &self,
        filename: P,
        (width, height): (f64, f64),
    ) -> std::io::Result<()> {
        use curve::Curve;
        use svg::{
            node::element::{Group, Path},
            node::Node,
            Document,
        };
        let mut document = Document::new()
            .set("width", format!("{}mm", width))
            .set("height", format!("{}mm", height))
            .set("viewBox", (0, 0, width, height));
        let mut group = Group::new()
            .set("transform", format!("matrix(1 0 0 -1 0 {:.0})", height))
            .set("fill", "none")
            .set("stroke", "black");

        for curve in &self.curves {
            let mut data = String::new();
            for segment in &curve.segments {
                if let Some(line) = segment.curve.downcast_ref::<curve::Line>() {
                    let (u0, u1) = segment.parameter_range;
                    let start = line.get_point(u0);
                    let end = line.get_point(u1);
                    data.push_str(&format!("M {:.2},{:.2}", start.x, start.y));
                    data.push_str(&format!(" L {:.2},{:.2}", end.x, end.y));
                    continue;
                }
                if let Some(polyline) = segment.curve.downcast_ref::<curve::Polyline>() {
                    for (index, point) in polyline.vertices.iter().enumerate() {
                        if index == 0 {
                            data.push_str(&format!("M {:.2},{:.2}", point.x, point.y));
                        } else {
                            data.push_str(&format!(" L {:.2},{:.2}", point.x, point.y));
                        }
                    }
                    continue;
                }
                if let Some(bspline) = segment.curve.downcast_ref::<curve::BSplineCurve<Point3>>() {
                    if bspline.degree == 3 {
                        for (index, bezier_curve) in
                            bspline.to_piecewise_bezier().into_iter().enumerate()
                        {
                            let start = bezier_curve.control_points[0];
                            let cp1 = bezier_curve.control_points[1];
                            let cp2 = bezier_curve.control_points[2];
                            let end = bezier_curve.control_points[3];
                            if index == 0 {
                                data.push_str(&format!("M {:.2},{:.2}", start.x, start.y));
                                data.push_str(&format!(
                                    " C {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
                                    cp1.x, cp1.y, cp2.x, cp2.y, end.x, end.y
                                ));
                            } else {
                                data.push_str(&format!(
                                    " {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
                                    cp1.x, cp1.y, cp2.x, cp2.y, end.x, end.y
                                ));
                            }
                        }
                        continue;
                    }
                }

                let points = segment.get_points();
                for point in points {
                    if data.len() == 0 {
                        data.push_str(&format!("M {:.2},{:.2}", point.x, point.y));
                    } else {
                        data.push_str(&format!(" L {:.2},{:.2}", point.x, point.y));
                    }
                }
            }
            let path = Path::new().set("d", data);
            group.append(path);
        }
        document.append(group);
        svg::save(filename, &document)?;
        Ok(())
    }
}

mod step_reader;
pub use step_reader::ModelReader as StepReader;
