pub(crate) enum AcirGenError {
    InvalidRangeConstraint { num_bits: u32 },
}

impl AcirGenError {
    pub(crate) fn message(&self) -> String {
        match self {
            AcirGenError::InvalidRangeConstraint { num_bits } => {
                // Don't apply any constraints if the range is for the maximum number of bits
                return format!(
            "All Witnesses are by default u{num_bits}. Applying this type does not apply any constraints.");
            }
        }
    }
}
