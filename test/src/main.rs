use delatin::{triangulate, Error};
use std::time::Instant;
use std::{fs::File, path::Path};

// TODO: rework to separate benchmark and tests
fn main() {
    let file = File::open(Path::new("./data/input.json")).unwrap();
    let heights: Vec<f64> = serde_json::from_reader(file).unwrap();

    let start = Instant::now();
    let triangles = triangulate(&heights, (512, 512), Error(0.2)).unwrap();
    let duration = start.elapsed();

    println!("Time elapsed in delatin triangulation is: {:?}.", duration);

    assert_eq!(triangles.len(), 32147);
}
