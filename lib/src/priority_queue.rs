use crate::Error;

#[derive(Debug, Clone)]
pub(crate) struct PriorityQueue {
    /// Priority queue of triangles based on error
    triangle_queue: Vec<usize>,
    /// Vector of triangle indices to their positions in the priority queue for faster retrieval
    triangle_queue_indices: Vec<Option<usize>>,
    /// Errors associated with triangles
    triangle_errors: Vec<Error>,
    /// Pending triangles to be processed
    pending_triangle_indices: Vec<usize>,
}

impl PriorityQueue {
    pub fn new(queue_len: usize) -> Self {
        Self {
            triangle_queue_indices: vec![None; queue_len],
            pending_triangle_indices: Vec::default(),
            triangle_queue: Vec::default(),
            triangle_errors: Vec::default(),
        }
    }

    pub fn add_pending_triangle(&mut self, t: usize) {
        self.pending_triangle_indices.push(t);
    }

    pub fn consume_pending_triangles(&mut self) -> Vec<usize> {
        self.pending_triangle_indices.drain(..).collect()
    }

    pub fn get_max_error(&self) -> Option<&Error> {
        self.triangle_errors.first()
    }

    pub fn push(&mut self, triangle_index: usize, error: Error) {
        let queue_length = self.triangle_queue.len();

        self.triangle_queue_indices[triangle_index] = Some(queue_length);
        self.triangle_queue.push(triangle_index);
        self.triangle_errors.push(error);
        self.up(queue_length);
    }

    pub fn pop(&mut self) -> Option<usize> {
        let last_item_index = self.triangle_queue.len() - 1;
        self.swap(0, last_item_index);
        self.down(0, last_item_index);

        self.pop_back()
    }

    pub fn remove(&mut self, requested_triangle_index: usize) {
        let Some(index) = self.triangle_queue_indices[requested_triangle_index] else {
            let pending_length = self.pending_triangle_indices.len();
            if let Some(pos) = self
                .pending_triangle_indices
                .iter()
                .position(|&triangle_index| triangle_index == requested_triangle_index)
            {
                self.pending_triangle_indices.swap(pos, pending_length - 1);
                self.pending_triangle_indices.pop();
            }

            return;
        };

        let last_item_index = self.triangle_queue.len() - 1;
        if last_item_index != index {
            self.swap(index, last_item_index);
            if !self.down(index, last_item_index) {
                self.up(index);
            }
        }
        self.pop_back();
    }

    fn up(&mut self, mut j: usize) {
        if j == 0 {
            return;
        }

        loop {
            let i: isize = (j as isize - 1) >> 1;
            if i < 0 {
                break;
            }

            let i = i as usize;
            if !self.less(j, i) {
                break;
            }

            self.swap(i, j);
            j = i;
        }
    }

    fn down(&mut self, i0: usize, n: usize) -> bool {
        let mut i = i0;
        loop {
            let j1 = 2 * i + 1;
            if j1 >= n {
                break;
            }
            let j2 = j1 + 1;
            let mut j = j1;
            if j2 < n && self.less(j2, j1) {
                j = j2;
            }
            if !self.less(j, i) {
                break;
            }
            self.swap(i, j);
            i = j;
        }

        i > i0
    }

    fn less(&self, i: usize, j: usize) -> bool {
        self.triangle_errors[i] > self.triangle_errors[j]
    }

    fn swap(&mut self, i: usize, j: usize) {
        let pi = self.triangle_queue[i];
        let pj = self.triangle_queue[j];
        self.triangle_queue_indices[pi] = Some(j);
        self.triangle_queue_indices[pj] = Some(i);
        self.triangle_queue.swap(i, j);
        self.triangle_errors.swap(i, j);
    }

    fn pop_back(&mut self) -> Option<usize> {
        let triangle = self.triangle_queue.pop();
        if let Some(triangle) = triangle {
            self.triangle_errors.pop();
            self.triangle_queue_indices[triangle] = None;
        }

        triangle
    }
}
