struct GrumpkinScalar {
    low: Field,
    high: Field,
}

impl GrumpkinScalar {
    fn new(low: Field, high: Field) -> Self {
        // TODO: check that the low and high value fit within the grumpkin modulus
        GrumpkinScalar { low, high }
    }
}

global GRUMPKIN_SCALAR_SERIALIZED_LEN: Field = 2;

fn deserialize_grumpkin_scalar(fields: [Field; GRUMPKIN_SCALAR_SERIALIZED_LEN]) -> GrumpkinScalar {
    GrumpkinScalar { low: fields[0], high: fields[1] }
}

fn serialize_grumpkin_scalar(scalar: GrumpkinScalar) -> [Field; GRUMPKIN_SCALAR_SERIALIZED_LEN] {
    [scalar.low, scalar.high]
}
