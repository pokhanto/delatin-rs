use crate::error::TriangulationError;
use crate::priority_queue::PriorityQueue;
use crate::utils::{get_signed_area, is_point_in_circumcircle};
use crate::{Error, Height, Point, Triangle};

#[derive(Debug, Clone)]
pub(crate) struct Triangulation<'a> {
    /// Height data of the grid
    height_data: &'a [Height],
    /// Width of the grid
    width: usize,
    /// Height of the grid
    height: usize,
    /// Points of vertices
    vertex_points: Vec<Point>,
    /// A vector of vertex points indices, triplets of these indices form triangles.
    triangles: Vec<usize>,
    /// A vector halfedge indices
    half_edges: Vec<Option<usize>>,
    /// A vector of candidate points
    candidate_points: Vec<Point>,
    // Priority queue for triangles
    priority_queue: PriorityQueue,
}

enum AddTriangleStrategy {
    Create,
    Update(usize),
}

impl<'a> Triangulation<'a> {
    /// Creates a new instance of `Triangulation` with the given height data, width, and height.
    ///
    /// # Arguments
    ///
    /// * `height_data` - Height values of the grid.
    /// * `width` - The width of the grid.
    /// * `height` - The height of the grid.
    ///
    /// # Returns
    ///
    /// A new `Triangulation` instance.
    pub(crate) fn new(height_data: &'a [Height], width: usize, height: usize) -> Self {
        let initial_queue_size = width * height / 4;
        Self {
            height_data,
            width,
            height,
            vertex_points: Vec::default(),
            triangles: Vec::default(),
            half_edges: Vec::default(),
            candidate_points: Vec::default(),
            priority_queue: PriorityQueue::new(initial_queue_size),
        }
    }

    /// Runs the triangulation process until the maximum error is below the specified threshold.
    ///
    /// # Arguments
    ///
    /// * `max_error` - The maximum allowable error for the triangulation process.
    ///
    /// # Returns
    ///
    /// Tuple containing:
    /// - points `Vec<(usize, usize)>`: A vector containing all the vertices of the triangulated mesh. Each point corresponds to heights vector index.
    /// - triangles `Vec<(usize, usize, usize)>`: A vector containing all the triangles of the mesh, each defined by indices into the `points`.
    ///
    /// # Errors
    ///
    /// - `InvalidDataLengthError` - If the length of the height data does not match the width and height of the grid.
    /// - `MaxErrorRetrievalError` - If the maximum error is not found in the priority queue.
    /// - `EmptyQueueError` - If the priority queue is empty during triangulation.
    ///
    pub(crate) fn run(
        &mut self,
        max_error: Error,
    ) -> Result<(Vec<Point>, Vec<Triangle>), TriangulationError> {
        let initial_x = self.width - 1;
        let initial_y = self.height - 1;

        let vertex_a_point_index = self.add_point((0, 0));
        let vertex_b_point_index = self.add_point((initial_x, 0));
        let vertex_c_point_index = self.add_point((initial_x, initial_y));
        let vertex_d_point_index = self.add_point((0, initial_y));

        // add initial two triangles to start with
        let trinagle_0 = self.add_triangle(
            (
                vertex_c_point_index,
                vertex_a_point_index,
                vertex_d_point_index,
            ),
            None,
            None,
            None,
            AddTriangleStrategy::Create,
        );
        let _triangle_1 = self.add_triangle(
            (
                vertex_a_point_index,
                vertex_c_point_index,
                vertex_b_point_index,
            ),
            Some(trinagle_0),
            None,
            None,
            AddTriangleStrategy::Create,
        );
        self.flush();

        while self
            .priority_queue
            .get_max_error()
            .ok_or(TriangulationError::MaxErrorRetrievalError)?
            > &max_error
        {
            self.refine()?;
        }

        Ok((self.get_vertext_points(), self.get_triangle_indices()))
    }

    fn get_triangle_indices(&self) -> Vec<(usize, usize, usize)> {
        let mut triangles = Vec::new();

        for i in 0..self.triangles.len() / 3 {
            let vertex_a_point_index = self.triangles[i * 3];
            let vertex_b_point_index = self.triangles[i * 3 + 1];
            let vertex_c_point_index = self.triangles[i * 3 + 2];

            triangles.push((
                vertex_a_point_index,
                vertex_b_point_index,
                vertex_c_point_index,
            ));
        }

        triangles
    }

    fn get_vertext_points(&self) -> Vec<(usize, usize)> {
        let mut points = Vec::new();

        for i in 0..self.vertex_points.len() {
            let vertex_point = self.vertex_points[i];

            points.push(vertex_point);
        }

        points
    }

    fn refine(&mut self) -> Result<(), TriangulationError> {
        self.step()?;
        self.flush();

        Ok(())
    }

    fn step(&mut self) -> Result<(), TriangulationError> {
        // pop triangle with highest error from priority queue
        let queued_triangle = self
            .priority_queue
            .pop()
            .ok_or(TriangulationError::EmptyQueueError)?;

        let vertex_a_triangle_index = queued_triangle * 3;
        let vertex_b_triangle_index = queued_triangle * 3 + 1;
        let vertex_c_triangle_index = queued_triangle * 3 + 2;

        let vertex_a_point_index = self.triangles[vertex_a_triangle_index];
        let vertex_b_point_index = self.triangles[vertex_b_triangle_index];
        let vertex_c_point_index = self.triangles[vertex_c_triangle_index];

        let point_a = self.vertex_points[vertex_a_point_index];
        let point_b = self.vertex_points[vertex_b_point_index];
        let point_c = self.vertex_points[vertex_c_point_index];

        let candidate_point = self.candidate_points[queued_triangle];
        let new_vertex_index = self.add_point(candidate_point);

        // check if candidate point is collinear with any of the triangle points
        if get_signed_area(point_a, point_b, candidate_point) == 0 {
            self.handle_collinear(new_vertex_index, vertex_a_triangle_index);
        } else if get_signed_area(point_b, point_c, candidate_point) == 0 {
            self.handle_collinear(new_vertex_index, vertex_b_triangle_index);
        } else if get_signed_area(point_c, point_a, candidate_point) == 0 {
            self.handle_collinear(new_vertex_index, vertex_c_triangle_index);
        } else {
            let half_edge_a = self.half_edges[vertex_a_triangle_index];
            let half_edge_b = self.half_edges[vertex_b_triangle_index];
            let half_edge_c = self.half_edges[vertex_c_triangle_index];

            let new_triangle_0 = self.add_triangle(
                (vertex_a_point_index, vertex_b_point_index, new_vertex_index),
                half_edge_a,
                None,
                None,
                AddTriangleStrategy::Update(vertex_a_triangle_index),
            );
            let new_triangle_1 = self.add_triangle(
                (vertex_b_point_index, vertex_c_point_index, new_vertex_index),
                half_edge_b,
                None,
                Some(new_triangle_0 + 1),
                AddTriangleStrategy::Create,
            );
            let new_triangle_2 = self.add_triangle(
                (vertex_c_point_index, vertex_a_point_index, new_vertex_index),
                half_edge_c,
                Some(new_triangle_0 + 2),
                Some(new_triangle_1 + 1),
                AddTriangleStrategy::Create,
            );

            self.legalize(new_triangle_0);
            self.legalize(new_triangle_1);
            self.legalize(new_triangle_2);
        }

        Ok(())
    }

    fn handle_collinear(&mut self, new_vertex_index: usize, collinear_vertex_index: usize) {
        let collinear_base_index = collinear_vertex_index - collinear_vertex_index % 3;
        let vertex_a_triangle_index = collinear_base_index + (collinear_vertex_index + 1) % 3;
        let vertex_b_triangle_index = collinear_base_index + (collinear_vertex_index + 2) % 3;

        let collinear_vertex_point_index = self.triangles[collinear_vertex_index];
        let vertex_a_point_index = self.triangles[vertex_a_triangle_index];
        let vertex_b_point_index = self.triangles[vertex_b_triangle_index];

        let half_edge_a = self.half_edges[vertex_a_triangle_index];
        let half_edge_b = self.half_edges[vertex_b_triangle_index];

        match self.half_edges[collinear_vertex_index] {
            Some(collinear_half_edge) => {
                let adjacent_triangle_base_index = collinear_half_edge - collinear_half_edge % 3;
                let adjacent_left_edge_index =
                    adjacent_triangle_base_index + (collinear_half_edge + 2) % 3;
                let adjacent_right_edge_index =
                    adjacent_triangle_base_index + (collinear_half_edge + 1) % 3;
                let vertex1 = self.triangles[adjacent_left_edge_index];
                let half_edge_adjacent_left = self.half_edges[adjacent_left_edge_index];
                let half_edge_adjacent_right = self.half_edges[adjacent_right_edge_index];

                self.priority_queue.remove(adjacent_triangle_base_index / 3);

                let new_triangle_0 = self.add_triangle(
                    (
                        vertex_b_point_index,
                        collinear_vertex_point_index,
                        new_vertex_index,
                    ),
                    half_edge_b,
                    None,
                    None,
                    AddTriangleStrategy::Update(collinear_base_index),
                );
                let new_triangle_1 = self.add_triangle(
                    (collinear_vertex_point_index, vertex1, new_vertex_index),
                    half_edge_adjacent_right,
                    None,
                    Some(new_triangle_0 + 1),
                    AddTriangleStrategy::Update(adjacent_triangle_base_index),
                );
                let new_triangle_2 = self.add_triangle(
                    (vertex1, vertex_a_point_index, new_vertex_index),
                    half_edge_adjacent_left,
                    None,
                    Some(new_triangle_1 + 1),
                    AddTriangleStrategy::Create,
                );
                let new_triangle_3 = self.add_triangle(
                    (vertex_a_point_index, vertex_b_point_index, new_vertex_index),
                    half_edge_a,
                    Some(new_triangle_0 + 2),
                    Some(new_triangle_2 + 1),
                    AddTriangleStrategy::Create,
                );

                self.legalize(new_triangle_0);
                self.legalize(new_triangle_1);
                self.legalize(new_triangle_2);
                self.legalize(new_triangle_3);
            }
            None => {
                let new_triangle_0 = self.add_triangle(
                    (
                        new_vertex_index,
                        vertex_b_point_index,
                        collinear_vertex_point_index,
                    ),
                    None,
                    half_edge_b,
                    None,
                    AddTriangleStrategy::Update(collinear_base_index),
                );
                let new_triangle_1 = self.add_triangle(
                    (vertex_b_point_index, new_vertex_index, vertex_a_point_index),
                    Some(new_triangle_0),
                    None,
                    half_edge_a,
                    AddTriangleStrategy::Create,
                );
                self.legalize(new_triangle_0 + 1);
                self.legalize(new_triangle_1 + 2);
            }
        }
    }

    /// If the pair of triangles doesn't satisfy the Delaunay condition
    /// (vertex_1 is inside the circumcircle of \[vertex_0, vertex_right, vertex_left\]), flip them,
    /// then do the same check/flip recursively for the new pair of triangles
    fn legalize(&mut self, requested_triangle_index: usize) {
        // find half edge of requested triangle
        // half edge will form adjacent triangle with same edge as requested
        let Some(half_edge) = self.half_edges[requested_triangle_index] else {
            return;
        };

        let requested_triangle_base_index = requested_triangle_index - requested_triangle_index % 3;
        let adjacent_triangle_base_index = half_edge - half_edge % 3;

        let requested_left_edge_index =
            requested_triangle_base_index + (requested_triangle_index + 1) % 3;
        let requested_right_edge_index =
            requested_triangle_base_index + (requested_triangle_index + 2) % 3;

        let adjacent_left_edge_index = adjacent_triangle_base_index + (half_edge + 2) % 3;
        let adjacent_right_edge_index = adjacent_triangle_base_index + (half_edge + 1) % 3;

        let vertex_0 = self.triangles[requested_right_edge_index];
        let vertex_right = self.triangles[requested_triangle_index];
        let vertex_left = self.triangles[requested_left_edge_index];
        let vertex_1 = self.triangles[adjacent_left_edge_index];

        if !is_point_in_circumcircle(
            self.vertex_points[vertex_1],
            self.vertex_points[vertex_0],
            self.vertex_points[vertex_right],
            self.vertex_points[vertex_left],
        ) {
            return;
        }

        let half_edge_left = self.half_edges[requested_left_edge_index];
        let half_edge_right = self.half_edges[requested_right_edge_index];
        let adjacent_half_edge_left = self.half_edges[adjacent_left_edge_index];
        let adjacent_half_edge_right = self.half_edges[adjacent_right_edge_index];

        self.priority_queue
            .remove(requested_triangle_base_index / 3);
        self.priority_queue.remove(adjacent_triangle_base_index / 3);

        let new_triangle_0 = self.add_triangle(
            (vertex_0, vertex_1, vertex_left),
            None,
            adjacent_half_edge_left,
            half_edge_left,
            AddTriangleStrategy::Update(requested_triangle_base_index),
        );
        let new_triangle_1 = self.add_triangle(
            (vertex_1, vertex_0, vertex_right),
            Some(new_triangle_0),
            half_edge_right,
            adjacent_half_edge_right,
            AddTriangleStrategy::Update(adjacent_triangle_base_index),
        );

        self.legalize(new_triangle_0 + 1);
        self.legalize(new_triangle_1 + 2);
    }

    fn flush(&mut self) {
        let pending_triangles = self.priority_queue.consume_pending_triangles();
        for pending_triangle in pending_triangles {
            // rasterize triangle to find maximum pixel error
            self.find_candidate(pending_triangle);
        }
    }

    // rasterize a triangle, find its max error, and queue it for processing
    fn find_candidate(&mut self, triangle_index: usize) {
        let vertex_a_point_index = self.triangles[triangle_index * 3];
        let vertex_b_point_index = self.triangles[triangle_index * 3 + 1];
        let vertex_c_point_index = self.triangles[triangle_index * 3 + 2];
        let point_a = self.vertex_points[vertex_a_point_index];
        let point_b = self.vertex_points[vertex_b_point_index];
        let point_c = self.vertex_points[vertex_c_point_index];

        // triangle bounding box
        let min_x = *[point_a.0, point_b.0, point_c.0]
            .iter()
            .min()
            .expect("No items to get min from.");
        let min_y = *[point_a.1, point_b.1, point_c.1]
            .iter()
            .min()
            .expect("No items to get min from.");
        let max_x = *[point_a.0, point_b.0, point_c.0]
            .iter()
            .max()
            .expect("No items to get max from.");
        let max_y = *[point_a.1, point_b.1, point_c.1]
            .iter()
            .max()
            .expect("No items to get max from.");

        let triangle_abc_signed_area = get_signed_area(point_a, point_b, point_c);
        let mut triangle_bcmin_signed_area = get_signed_area(point_b, point_c, (min_x, min_y));
        let mut triangle_camin_signed_area = get_signed_area(point_c, point_a, (min_x, min_y));
        let mut triangle_abmin_signed_area = get_signed_area(point_a, point_b, (min_x, min_y));

        let ba_y_diff = point_b.1 as isize - point_a.1 as isize;
        let ab_x_diff = point_a.0 as isize - point_b.0 as isize;
        let cb_y_diff = point_c.1 as isize - point_b.1 as isize;
        let bc_x_diff = point_b.0 as isize - point_c.0 as isize;
        let ac_y_diff = point_a.1 as isize - point_c.1 as isize;
        let ca_x_diff = point_c.0 as isize - point_a.0 as isize;

        let normalized_height_at_a = self.height_at(point_a) / triangle_abc_signed_area as f64;
        let normalized_height_at_b = self.height_at(point_b) / triangle_abc_signed_area as f64;
        let normalized_height_at_c = self.height_at(point_c) / triangle_abc_signed_area as f64;

        // iterate over pixels in bounding box
        let mut max_error = Error(0.0);
        let mut max_error_point: Point = (0, 0);
        for y in min_y..=max_y {
            // compute starting offset
            let mut offset_x = 0;
            if triangle_bcmin_signed_area < 0 && cb_y_diff != 0 {
                offset_x = offset_x
                    .max(((-triangle_bcmin_signed_area / cb_y_diff) as f32).floor() as usize);
            }
            if triangle_camin_signed_area < 0 && ac_y_diff != 0 {
                offset_x = offset_x
                    .max(((-triangle_camin_signed_area / ac_y_diff) as f32).floor() as usize);
            }
            if triangle_abmin_signed_area < 0 && ba_y_diff != 0 {
                offset_x = offset_x
                    .max(((-triangle_abmin_signed_area / ba_y_diff) as f32).floor() as usize);
            }

            let signed_offset_x = offset_x as isize;
            let mut triangle_bcmin_adjusted_signed_area =
                triangle_bcmin_signed_area + cb_y_diff * signed_offset_x;
            let mut triangle_camin_adjusted_signed_area =
                triangle_camin_signed_area + ac_y_diff * signed_offset_x;
            let mut triangle_abmin_adjusted_signed_area =
                triangle_abmin_signed_area + ba_y_diff * signed_offset_x;

            let mut was_inside = false;

            for x in min_x + offset_x..=max_x {
                // check if inside triangle
                if triangle_bcmin_adjusted_signed_area >= 0
                    && triangle_camin_adjusted_signed_area >= 0
                    && triangle_abmin_adjusted_signed_area >= 0
                {
                    was_inside = true;

                    // compute z using barycentric coordinates
                    let z = normalized_height_at_a * (triangle_bcmin_adjusted_signed_area as f64)
                        + normalized_height_at_b * (triangle_camin_adjusted_signed_area as f64)
                        + normalized_height_at_c * (triangle_abmin_adjusted_signed_area as f64);
                    let z_diff = (z - self.height_at((x, y))).abs();

                    if z_diff > max_error.0 {
                        max_error = Error(z_diff);
                        max_error_point = (x, y);
                    }
                } else if was_inside {
                    break;
                }

                triangle_bcmin_adjusted_signed_area += cb_y_diff;
                triangle_camin_adjusted_signed_area += ac_y_diff;
                triangle_abmin_adjusted_signed_area += ba_y_diff;
            }

            triangle_bcmin_signed_area += bc_x_diff;
            triangle_camin_signed_area += ca_x_diff;
            triangle_abmin_signed_area += ab_x_diff;
        }

        if max_error_point == point_a || max_error_point == point_b || max_error_point == point_c {
            max_error = Error(0.0);
        }

        self.candidate_points[triangle_index] = max_error_point;
        self.priority_queue.push(triangle_index, max_error);
    }

    fn height_at(&self, point: Point) -> Height {
        self.height_data[self.width * point.1 + point.0]
    }

    fn add_point(&mut self, point: Point) -> usize {
        let index = self.vertex_points.len();
        self.vertex_points.push(point);

        index
    }

    fn add_triangle(
        &mut self,
        triangle: Triangle,
        half_edge_ab: Option<usize>,
        half_edge_bc: Option<usize>,
        half_edge_ca: Option<usize>,
        add_strategy: AddTriangleStrategy,
    ) -> usize {
        let index_to_add = match add_strategy {
            AddTriangleStrategy::Update(index) => {
                self.triangles[index] = triangle.0;
                self.triangles[index + 1] = triangle.1;
                self.triangles[index + 2] = triangle.2;

                self.half_edges[index] = half_edge_ab;
                self.half_edges[index + 1] = half_edge_bc;
                self.half_edges[index + 2] = half_edge_ca;

                index
            }
            AddTriangleStrategy::Create => {
                let index = self.triangles.len();
                self.triangles.push(triangle.0);
                self.triangles.push(triangle.1);
                self.triangles.push(triangle.2);

                self.half_edges.push(half_edge_ab);
                self.half_edges.push(half_edge_bc);
                self.half_edges.push(half_edge_ca);

                index
            }
        };

        let triangle_index = index_to_add / 3;

        // link adjacent half edges
        if let Some(half_edge_ab) = half_edge_ab {
            self.half_edges[half_edge_ab] = Some(index_to_add);
        }
        if let Some(half_edge_bc) = half_edge_bc {
            self.half_edges[half_edge_bc] = Some(index_to_add + 1);
        }
        if let Some(half_edge_ca) = half_edge_ca {
            self.half_edges[half_edge_ca] = Some(index_to_add + 2);
        }

        self.candidate_points.push((0, 0));
        self.priority_queue.add_pending_triangle(triangle_index);

        index_to_add
    }
}
