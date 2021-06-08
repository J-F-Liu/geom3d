use geom3d::model::StepReader;

fn main() {
    let file = std::env::args().nth(1).expect("stp file name");
    let file = std::path::Path::new(&file);
    let model = StepReader::read_model(&file).unwrap();
    if model.faces.len() > 0 {
        model.save_as_obj(file.with_extension("obj")).unwrap();
    }
    if model.curves.len() > 0 {
        model
            .save_as_svg(file.with_extension("svg"), (350.0, 245.0))
            .unwrap();
    }
}
