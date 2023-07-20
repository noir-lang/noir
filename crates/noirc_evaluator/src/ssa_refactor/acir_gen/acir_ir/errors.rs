use acvm::FieldElement;
use noirc_errors::Location;

use crate::errors::{RuntimeError, RuntimeErrorKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum AcirGenError {
    InvalidRangeConstraint { num_bits: u32, location: Option<Location> },
    IndexOutOfBounds { index: usize, array_size: usize, location: Option<Location> },
    UnsupportedIntegerSize { num_bits: u32, max_num_bits: u32, location: Option<Location> },
    BadConstantEquality { lhs: FieldElement, rhs: FieldElement, location: Option<Location> },
}

impl AcirGenError {
    pub(crate) fn message(&self) -> String {
        match self {
            AcirGenError::InvalidRangeConstraint { num_bits, .. } => {
                // Don't apply any constraints if the range is for the maximum number of bits or more.
                format!(
             "All Witnesses are by default u{num_bits} Applying this type does not apply any constraints.\n We also currently do not allow integers of size more than {num_bits}, this will be handled by BigIntegers.")
            }
            AcirGenError::IndexOutOfBounds { index, array_size, .. } => {
                format!("Index out of bounds, array has size {array_size}, but index was {index}")
            }
            AcirGenError::UnsupportedIntegerSize { num_bits, max_num_bits, .. } => {
                format!("Integer sized {num_bits} is over the max supported size of {max_num_bits}")
            }
            AcirGenError::BadConstantEquality { lhs, rhs, .. } => {
                format!("{lhs} and {rhs} constrained to be equal though they never can be")
            }
        }
    }
}

impl From<AcirGenError> for RuntimeError {
    fn from(error: AcirGenError) -> Self {
        match error {
            AcirGenError::InvalidRangeConstraint { num_bits, location } => {
                let kind = RuntimeErrorKind::UnstructuredError {
                    message: format!(
                        "Failed range constraint when constraining to {num_bits} bits"
                    ),
                };
                RuntimeError::new(kind, location)
            }
            AcirGenError::IndexOutOfBounds { index, array_size, location } => {
                let kind = RuntimeErrorKind::ArrayOutOfBounds {
                    index: index as u128,
                    bound: array_size as u128,
                };
                RuntimeError::new(kind, location)
            }
            AcirGenError::UnsupportedIntegerSize { num_bits, max_num_bits, location } => {
                let kind = RuntimeErrorKind::UnstructuredError {
                    message: format!("Unsupported integer size of {num_bits} bits. The maximum supported size is {max_num_bits} bits.")
                };
                RuntimeError::new(kind, location)
            }
            AcirGenError::BadConstantEquality { lhs: _, rhs: _, location } => {
                // We avoid showing the actual lhs and rhs since most of the time they are just 0
                // and 1 respectively. This would confuse users if a constraint such as
                // assert(foo < bar) fails with "failed constraint: 0 = 1."
                let kind =
                    RuntimeErrorKind::UnstructuredError { message: "Failed constraint".into() };
                RuntimeError::new(kind, location)
            }
        }
    }
}
