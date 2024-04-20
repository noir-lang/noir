struct GrumpkinScalar {
    low: field,
    high: field,
}

impl GrumpkinScalar {
    pub fn new(low: field, high: field) -> Self {
        // TODO: check that the low and high value fit within the grumpkin modulus
        GrumpkinScalar { low, high }
    }
}

global GRUMPKIN_SCALAR_SERIALIZED_LEN: field = 2;

pub fn deserialize_grumpkin_scalar(fields: [field; GRUMPKIN_SCALAR_SERIALIZED_LEN]) -> GrumpkinScalar {
    GrumpkinScalar { low: fields[0], high: fields[1] }
}

pub fn serialize_grumpkin_scalar(scalar: GrumpkinScalar) -> [field; GRUMPKIN_SCALAR_SERIALIZED_LEN] {
    [scalar.low, scalar.high]
}
