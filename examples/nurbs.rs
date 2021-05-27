use geom3d::model::StepReader;

fn main() {
    let file = std::env::args().nth(1).expect("stp file name");
    let file = std::path::Path::new(&file);
    let model = StepReader::read_model(&file).unwrap();
    model.save_as_obj(file.with_extension("obj")).unwrap();
}
