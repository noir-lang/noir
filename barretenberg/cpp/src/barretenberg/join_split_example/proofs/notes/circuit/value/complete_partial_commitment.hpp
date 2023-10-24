#pragma once

#include "../../constants.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
namespace join_split_example::proofs::notes::circuit::value {

inline auto complete_partial_commitment(field_ct const& value_note_partial_commitment,
                                        suint_ct const& value,
                                        suint_ct const& asset_id,
                                        field_ct const& input_nullifier)
{
    return pedersen_hash::hash({ value_note_partial_commitment, value.value, asset_id.value, input_nullifier },
                               GeneratorIndex::VALUE_NOTE_COMMITMENT);
}

} // namespace join_split_example::proofs::notes::circuit::value
