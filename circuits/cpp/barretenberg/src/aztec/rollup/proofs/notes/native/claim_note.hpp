#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

struct claim_note {
    // value of deposited tokens
    uint256_t deposit_value;
    // defi bridge identifier (address, assets involved, number of output notes)
    bridge_id bridge_id;
    // global rollup variable - total number of defi interactions made
    uint32_t defi_interaction_nonce;
    // binds the claim note to the user - this is a join-split note without the `value` and `asset_id` fields (used by
    // rollup provider to create output notes
    grumpkin::g1::affine_element partial_state;
};

// grumpkin::g1::affine_element encrypt_note(claim_note const& note);

// barretenberg::fr compute_nullifier(grumpkin::g1::affine_element const& encrypted_note,
//                                    const uint32_t tree_index);

// inline bool operator==(claim_note const& lhs, claim_note const& rhs)
// {
//     return lhs.owner == rhs.owner && lhs.value == rhs.value && lhs.secret == rhs.secret && lhs.nonce == rhs.nonce;
// }

// inline std::ostream& operator<<(std::ostream& os, claim_note const& note)
// {
//     os << "{ owner_x: " << note.owner.x << ", owner_y: " << note.owner.y << ", view_key: " << note.secret
//        << ", value: " << note.value << ", asset_id: " << note.asset_id << ", nonce: " << note.nonce << " }";
//     return os;
// }

// inline void read(uint8_t const*& it, claim_note& note)
// {
//     using serialize::read;
//     read(it, note.value);
//     read(it, note.asset_id);
//     read(it, note.nonce);
//     read(it, note.owner);
//     read(it, note.secret);
// }

// inline void write(std::vector<uint8_t>& buf, claim_note const& note)
// {
//     using serialize::write;
//     write(buf, note.value);
//     write(buf, note.asset_id);
//     write(buf, note.nonce);
//     write(buf, note.owner);
//     write(buf, note.secret);
// }
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup