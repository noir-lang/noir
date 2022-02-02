#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

using namespace barretenberg;

inline auto compute_nullifier(grumpkin::fq const& note_commitment)
{
    return crypto::pedersen::compress_native({ note_commitment }, GeneratorIndex::CLAIM_NOTE_NULLIFIER);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup