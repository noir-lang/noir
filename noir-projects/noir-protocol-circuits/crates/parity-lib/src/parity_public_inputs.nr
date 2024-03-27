use dep::types::{mocked::AggregationObject};

struct ParityPublicInputs {
    aggregation_object: AggregationObject,
    sha_root: Field,
    converted_root: Field,
}
