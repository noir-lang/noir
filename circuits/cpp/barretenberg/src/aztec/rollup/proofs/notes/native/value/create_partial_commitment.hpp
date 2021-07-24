#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace value {

inline auto create_partial_commitment(barretenberg::fr const& secret,
                                      grumpkin::g1::affine_element const& owner,
                                      uint32_t nonce)
{
    return crypto::pedersen::compress_native({ secret, owner.x, owner.y, nonce },
                                             GeneratorIndex::VALUE_NOTE_PARTIAL_COMMITMENT);
}

} // namespace value
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup