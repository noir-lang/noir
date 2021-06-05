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

inline grumpkin::g1::affine_element complete_partial_claim_note(grumpkin::g1::affine_element const& claim_note,
                                                                uint32_t nonce)
{
    grumpkin::g1::element sum = claim_note;

    if (nonce > 0) {
        sum += crypto::pedersen::fixed_base_scalar_mul<32>(
            (uint64_t)nonce, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_DEFI_INTERACTION_NONCE);
    }
    sum = sum.normalize();

    return { sum.x, sum.y };
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup