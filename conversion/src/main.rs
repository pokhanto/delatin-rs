use std::fs::OpenOptions;
use std::io::Write;
use std::{fs::File, path::Path};

use delatin::{triangulate, Error};

fn main() {
    let width = 512;
    let height = 512;
    let json_file_path = Path::new("./data/input.json");
    let file = File::open(json_file_path).unwrap();
    let heights: Vec<f64> = serde_json::from_reader(file).unwrap();
    let (points, triangles) = triangulate(&heights, (width, height), Error(1.0)).unwrap();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./conversion/output.obj")
        .unwrap();

    for point in points {
        let original_index_a = point.1 * width + point.0;
        let height = heights[original_index_a];
        writeln!(file, "v {} {} {}", point.0, point.1, height).unwrap();
    }

    for (a, b, c) in triangles {
        writeln!(file, "f {} {} {}", a + 1, b + 1, c + 1).unwrap();
    }
}
