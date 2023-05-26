#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum AcirGenError {
    InvalidRangeConstraint { num_bits: u32 },
    IndexOutOfBounds { index: usize, array_size: usize },
}

impl AcirGenError {
    pub(crate) fn message(&self) -> String {
        match self {
            AcirGenError::InvalidRangeConstraint { num_bits } => {
                // Don't apply any constraints if the range is for the maximum number of bits
                format!(
             "All Witnesses are by default u{num_bits}. Applying this type does not apply any constraints.")
            }
            AcirGenError::IndexOutOfBounds { index, array_size } => {
                format!("Index out of bounds, array has size {array_size}, but index was {index}")
            }
        }
    }
}
