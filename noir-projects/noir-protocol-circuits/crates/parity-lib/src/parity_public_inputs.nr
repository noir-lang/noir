use dep::types::{
    constants::NUM_FIELDS_PER_SHA256,
    mocked::AggregationObject,
};

struct ParityPublicInputs {
    aggregation_object: AggregationObject,
    sha_root: [Field; NUM_FIELDS_PER_SHA256],
    converted_root: Field,
}
