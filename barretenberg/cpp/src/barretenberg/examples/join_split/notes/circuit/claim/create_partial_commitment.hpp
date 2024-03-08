#pragma once
#include "barretenberg/examples/join_split/constants.hpp"
#include "barretenberg/examples/join_split/types.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"

namespace bb::join_split_example::proofs::notes::circuit::claim {

inline auto create_partial_commitment(field_ct const& deposit_value,
                                      field_ct const& bridge_call_data,
                                      field_ct const& value_note_partial_commitment,
                                      field_ct const& input_nullifier)
{
    return pedersen_hash::hash({ deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier },
                               GeneratorIndex::CLAIM_NOTE_PARTIAL_COMMITMENT);
}

} // namespace bb::join_split_example::proofs::notes::circuit::claim
