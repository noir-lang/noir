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

inline grumpkin::g1::affine_element create_partial_value_note(barretenberg::fr const& secret,
                                                              grumpkin::g1::affine_element const& owner,
                                                              uint32_t nonce)
{
    grumpkin::g1::element sum =
        crypto::pedersen::fixed_base_scalar_mul<254>(secret, GeneratorIndex::JOIN_SPLIT_NOTE_SECRET);
    sum += crypto::pedersen::compress_to_point_native(owner.x, owner.y, GeneratorIndex::JOIN_SPLIT_NOTE_OWNER);

    if (nonce > 0) {
        sum += crypto::pedersen::fixed_base_scalar_mul<32>((uint64_t)nonce, GeneratorIndex::JOIN_SPLIT_NOTE_NONCE);
    }
    sum = sum.normalize();

    return { sum.x, sum.y };
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup