/*!
Simple and fast [**TIN**](https://en.wikipedia.org/wiki/Triangulated_irregular_network) generation library. 
Uses [Delaunay triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation).

# Example

```rust
use delatin::triangulate;

let heights = vec![100.1, 123.4, 111.5, 121.4];
let width = 2;
let height = 2;
let max_error = 1.0;
// triangles is a vector of tuples containing three indices to original height data 
// every tuple forms a triangle
let triangles = triangulate(&heights, width, height, max_error)?;
```
*/

use std::fmt;

pub use error::TriangulationError;
use triangulation::Triangulation;

mod error;
mod priority_queue;
mod triangulation;
mod utils;

// TODO: consider NewTypes
type Point = (usize, usize);
type Triangle = (usize, usize, usize);
type Height = f64;

/// Error for the triangulation process.
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Error(pub f64);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Runs the triangulation process until the maximum error is below the specified threshold.
///
/// # Arguments
///
/// * `height_data` - Height values of the grid.
/// * `dimesions` - Tuple width and height of the grid.
/// * `max_error` - The maximum allowable error for the triangulation process.
///
/// # Returns
///
/// Array of triangles, where every triangle is tuple of three indices of original heigth data array.
///
/// # Errors
///
/// `InvalidDataLengthError` - If the length of the height data does not match the width and height of the grid.
/// `MaxErrorRetrievalError` - If the maximum error is not found in the priority queue.
/// `EmptyQueueError` - If the priority queue is empty during triangulation.
///
pub fn triangulate(
    height_data: &[f64],
    dimesions: (usize, usize),
    max_error: Error,
) -> Result<Vec<(usize, usize, usize)>, TriangulationError> {
    let width = dimesions.0;
    let height = dimesions.1;
    (height_data.len() == width * height)
        .then_some(())
        .ok_or(TriangulationError::InvalidDataLengthError)?;

    let mut delatin = Triangulation::new(height_data, width, height);

    let (points, triangles) = delatin.run(max_error)?;

    Ok(triangles
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
        .collect::<Vec<(usize, usize, usize)>>())
}
