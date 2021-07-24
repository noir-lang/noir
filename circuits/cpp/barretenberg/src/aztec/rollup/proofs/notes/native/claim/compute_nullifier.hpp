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

inline auto compute_nullifier(grumpkin::fq const& note_commitment, uint32_t index)
{
    return crypto::pedersen::compress_native({ note_commitment, index }, GeneratorIndex::CLAIM_NOTE_NULLIFIER);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup