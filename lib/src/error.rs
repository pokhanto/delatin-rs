use std::fmt;

#[derive(Debug)]
pub enum TriangulationError {
    MaxErrorRetrievalError,
    EmptyQueueError,
    InvalidDataLengthError,
}

impl fmt::Display for TriangulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TriangulationError::MaxErrorRetrievalError => write!(f, "No max error in queue."),
            TriangulationError::EmptyQueueError => write!(f, "Priority queue is empty."),
            TriangulationError::InvalidDataLengthError => write!(
                f,
                "Length of heights data is not equal to width * height."
            ),
        }
    }
}

impl std::error::Error for TriangulationError {}
