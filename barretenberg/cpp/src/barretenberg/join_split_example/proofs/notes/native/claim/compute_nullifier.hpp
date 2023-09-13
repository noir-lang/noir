#pragma once
#include "../../constants.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

inline auto compute_nullifier(grumpkin::fq const& note_commitment)
{
    return crypto::pedersen_commitment::compress_native({ note_commitment }, GeneratorIndex::CLAIM_NOTE_NULLIFIER);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example
