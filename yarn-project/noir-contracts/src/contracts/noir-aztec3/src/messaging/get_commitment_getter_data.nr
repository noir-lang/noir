use crate::oracle::get_commitment::COMMITMENT_GETTER_LENGTH;

struct CommitmentGetterData {
    message: Field,
    leaf_index: Field,
    root: Field,
}

fn make_commitment_getter_data(fields: [Field; COMMITMENT_GETTER_LENGTH], start: comptime Field) -> CommitmentGetterData {
    CommitmentGetterData {
        message: fields[start],
        leaf_index: fields[start + 1],
        root: fields[start + 2],
    }
}