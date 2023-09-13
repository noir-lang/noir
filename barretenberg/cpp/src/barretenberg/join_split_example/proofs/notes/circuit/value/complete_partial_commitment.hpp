#pragma once

#include "../../constants.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
namespace join_split_example {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

inline auto complete_partial_commitment(field_ct const& value_note_partial_commitment,
                                        suint_ct const& value,
                                        suint_ct const& asset_id,
                                        field_ct const& input_nullifier)
{
    return pedersen_commitment::compress(
        { value_note_partial_commitment, value.value, asset_id.value, input_nullifier },
        GeneratorIndex::VALUE_NOTE_COMMITMENT);
}

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace join_split_example
