#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

struct defi_interaction_result_note {
    bridge_id bridge_id;
    uint32_t interaction_nonce;
    uint256_t total_input_value;
    uint256_t total_output_a_value;
    // output_b_value defaults to 0 if there is only one output note for a given defi bridge
    uint256_t total_output_b_value;
    // did the rollup smart contract call to the defi bridge succeed or fail?
    bool interaction_result;
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