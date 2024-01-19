#pragma once
#include "../../constants.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"

namespace bb::join_split_example::proofs::notes::circuit::claim {

using namespace bb::stdlib;

inline auto complete_partial_commitment(field_ct const& partial_commitment,
                                        field_ct const& interaction_nonce,
                                        suint_ct const& fee)
{
    return pedersen_hash::hash({ partial_commitment, interaction_nonce, fee.value },
                               GeneratorIndex::CLAIM_NOTE_COMMITMENT);
}

} // namespace bb::join_split_example::proofs::notes::circuit::claim
