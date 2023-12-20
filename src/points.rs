use crate::{Float, Point2, Point3, Vec3};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

pub fn scale_points(points: &[Point3], scale: Float) -> Vec<Point3> {
    points.iter().map(|&point| point * scale).collect()
}

pub fn load_point_cloud<P: AsRef<Path>>(file: P) -> std::io::Result<Vec<Point3>> {
    let bytes = std::fs::read(file)?;
    let cursor = std::io::Cursor::new(bytes);
    let mut points = Vec::new();
    for line in cursor.lines() {
        let cords = line?;
        if cords.starts_with('#') {
            // skip comment
            continue;
        }
        match cords
            .split([' ', ',', '\t'])
            .filter(|item| !item.is_empty())
            .collect::<Vec<&str>>()
            .as_slice()
        {
            [x, y, z] => points.push(Point3::new(
                Float::from_str(x).unwrap(),
                Float::from_str(y).unwrap(),
                Float::from_str(z).unwrap(),
            )),
            [_, x, y, z] => points.push(Point3::new(
                Float::from_str(x).unwrap(),
                Float::from_str(y).unwrap(),
                Float::from_str(z).unwrap(),
            )),
            _ => {}
        }
    }
    Ok(points)
}

pub fn save_point_cloud<P: AsRef<Path>>(points: &[Point3], file: P) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    for point in points {
        file.write(format!("{} {} {}\n", point.x, point.y, point.z).as_bytes())?;
    }
    Ok(())
}

pub fn save_point_cloud_with_normal<P: AsRef<Path>>(
    points: &[(Point3, Vec3)],
    file: P,
) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    for (point, normal) in points {
        file.write(
            format!(
                "{} {} {} {} {} {}\n",
                point.x, point.y, point.z, normal.x, normal.y, normal.z
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}

pub fn save_points<P: AsRef<Path>>(points: &[Point2], file: P) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    for point in points {
        file.write(format!("{},{}\n", point.x, point.y).as_bytes())?;
    }
    Ok(())
}
