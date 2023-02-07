#pragma once
#include <stdlib/types/types.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types;

inline auto create_partial_commitment(field_ct const& deposit_value,
                                      field_ct const& bridge_call_data,
                                      field_ct const& value_note_partial_commitment,
                                      field_ct const& input_nullifier)
{
    return pedersen::compress({ deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier },
                              GeneratorIndex::CLAIM_NOTE_PARTIAL_COMMITMENT);
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace join_split_example