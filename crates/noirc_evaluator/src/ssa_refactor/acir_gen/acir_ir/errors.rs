use super::memory::ArrayId;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum AcirGenError {
    InvalidRangeConstraint { num_bits: u32 },
    IndexOutOfBounds { index: usize, array_size: usize },
    UninitializedElementInArray { index: usize, array_id: ArrayId },
}

impl AcirGenError {
    pub(crate) fn message(&self) -> String {
        match self {
            AcirGenError::InvalidRangeConstraint { num_bits } => {
                // Don't apply any constraints if the range is for the maximum number of bits or more.
                format!(
             "All Witnesses are by default u{num_bits} Applying this type does not apply any constraints.\n We also currently do not allow integers of size more than {num_bits}, this will be handled by BigIntegers.")
            }
            AcirGenError::IndexOutOfBounds { index, array_size } => {
                format!("Index out of bounds, array has size {array_size}, but index was {index}")
            }
            AcirGenError::UninitializedElementInArray { index, array_id } => {
                format!("The element at index {index} was never set in array {array_id:?}")
            }
        }
    }
}
