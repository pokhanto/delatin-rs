use delatin::{triangulate, Error};
use plotters::prelude::*;
use std::{fs::File, path::Path};

pub fn main() {
    let width = 512;
    let height = 512;
    let json_file_path = Path::new("./data/input.json");
    let file = File::open(json_file_path).unwrap();
    let heights: Vec<f64> = serde_json::from_reader(file).unwrap();
    let (points, triangles) = triangulate(&heights, (width, height), Error(1.0)).unwrap();

    let triangles = triangles
        .into_iter()
        .map(|(a, b, c)| {
            let point_a = points[a];
            let point_b = points[b];
            let point_c = points[c];

            let original_index_a = point_a.1 * width + point_a.0;
            let original_index_b = point_b.1 * width + point_b.0;
            let original_index_c = point_c.1 * width + point_c.0;

            (original_index_a, original_index_b, original_index_c)
        })
        .collect::<Vec<(usize, usize, usize)>>();

    plot_triangles(triangles, width, height, "./plot/plot.png").unwrap();
}

pub fn plot_triangles(
    triangles: Vec<(usize, usize, usize)>,
    width: usize,
    height: usize,
    plot_image_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let margin: u32 = 5;
    let width = width as i32;
    let height = height as i32;
    let root = BitMapBackend::new(
        plot_image_path,
        (width as u32 + margin * 2, height as u32 + margin * 2),
    )
    .into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(margin)
        .build_cartesian_2d(0..width, 0..height)?;

    chart.configure_mesh().draw()?;

    for (a, b, c) in triangles {
        let a = a as i32;
        let b = b as i32;
        let c = c as i32;

        let point_a_x = a % width;
        let point_a_y = a / width;
        let point_b_x = b % width;
        let point_b_y = b / width;
        let point_c_x = c % width;
        let point_c_y = c / width;

        chart.draw_series(LineSeries::new(
            vec![
                (point_a_x, point_a_y),
                (point_b_x, point_b_y),
                (point_c_x, point_c_y),
                (point_a_x, point_a_y),
            ],
            &BLUE,
        ))?;
    }

    Ok(())
}
