use geom3d::{
    surface::{BezierSurface, SurfacePatch},
    Float, Grid, Point3,
};

type Model = Vec<SurfacePatch<BezierSurface<Point3>>>;
const DIVISIONS: (usize, usize) = (10, 10);

fn load_teapot() -> std::io::Result<Model> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use std::str::FromStr;

    let file = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/teapot.bpt");
    let reader = BufReader::new(File::open(file)?);

    let mut model = Model::new();
    let mut points = Vec::new();
    let mut current_cols = 0;

    for line in reader.lines() {
        let numbers = line?;
        let items = numbers.split_whitespace().collect::<Vec<_>>();
        if items.len() == 1 {
            model.reserve_exact(usize::from_str(items[0]).unwrap());
        } else if items.len() == 2 {
            if points.len() > 0 {
                let surface = SurfacePatch {
                    surface: BezierSurface::new(Grid::from_vec(points, current_cols)),
                    parameter_range: ((0.0, 1.0), (0.0, 1.0)),
                    sample_count: DIVISIONS,
                };
                model.push(surface);
            }
            let m = usize::from_str(items[0]).unwrap();
            let n = usize::from_str(items[1]).unwrap();
            points = Vec::with_capacity((m + 1) * (n + 1));
            current_cols = n + 1;
        } else if items.len() == 3 {
            let point = Point3::new(
                Float::from_str(items[0]).unwrap(),
                Float::from_str(items[1]).unwrap(),
                Float::from_str(items[2]).unwrap(),
            );
            points.push(point);
        }
    }
    // add last surface
    let surface = SurfacePatch {
        surface: BezierSurface::new(Grid::from_vec(points, current_cols)),
        parameter_range: ((0.0, 1.0), (0.0, 1.0)),
        sample_count: (10, 10),
    };
    model.push(surface);
    Ok(model)
}

// cargo run --example bezier > teapot.xyz
fn main() {
    let teapot = load_teapot().unwrap();
    for surface in teapot {
        for point in surface.get_points().iter() {
            println!("{} {} {}", point.x, point.y, point.z);
        }
    }
}
