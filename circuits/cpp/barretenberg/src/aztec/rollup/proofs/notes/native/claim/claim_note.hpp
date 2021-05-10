#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

struct claim_note {
    // value of deposited tokens
    uint256_t deposit_value;
    // defi bridge identifier (address, assets involved, number of output notes)
    uint256_t bridge_id;
    // global rollup variable - total number of defi interactions made
    uint32_t defi_interaction_nonce;

    // binds the claim note to the user - this is a join-split note without the `value` and `asset_id` fields (used by
    // rollup provider to create output notes
    grumpkin::g1::affine_element partial_state;
};

grumpkin::g1::affine_element encrypt(claim_note const& note);

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup