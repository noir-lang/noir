#pragma once
#include <stdlib/types/types.hpp>
#include "../../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types;

inline auto complete_partial_commitment(field_ct const& partial_commitment,
                                        field_ct const& interaction_nonce,
                                        suint_ct const& fee)
{
    return pedersen::compress({ partial_commitment, interaction_nonce, fee.value },
                              GeneratorIndex::CLAIM_NOTE_COMMITMENT);
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace join_split_example