use dep::types::{mocked::AggregationObject, traits::Empty};

struct ParityPublicInputs {
    aggregation_object: AggregationObject,
    sha_root: Field,
    converted_root: Field,
}

impl Empty for ParityPublicInputs {
    fn empty() -> Self {
        ParityPublicInputs {
            aggregation_object: AggregationObject::empty(),
            sha_root: 0,
            converted_root: 0,
        }
    }
}