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

grumpkin::g1::affine_element encrypt_note(claim_note const& note);

// barretenberg::fr compute_nullifier(grumpkin::g1::affine_element const& encrypted_note,
//                                    const uint32_t tree_index);

inline bool operator==(claim_note const& lhs, claim_note const& rhs)
{
    return lhs.bridge_id == rhs.bridge_id && lhs.deposit_value == rhs.deposit_value &&
           lhs.partial_state == rhs.partial_state && lhs.defi_interaction_nonce == rhs.defi_interaction_nonce;
}

inline std::ostream& operator<<(std::ostream& os, claim_note const& note)
{
    os << "{ partial_state_x: " << note.partial_state.x << ", partial_state_y: " << note.partial_state.y
       << ", deposit value: " << note.deposit_value << ", input_asset_id: " << note.bridge_id.input_asset_id
       << ", output_asset_id_a: " << note.bridge_id.output_asset_id_a
       << ", output_asset_id_b: " << note.bridge_id.output_asset_id_b
       << ", bridge contract address: " << note.bridge_id.bridge_contract_address
       << ", defi interation nonce: " << note.defi_interaction_nonce << " }";
    return os;
}

// inline void read(uint8_t const*& it, claim_note& note)
// {
//     using serialize::read;
//     read(it, note.deposit_value);
//     read(it, note.bridge_id);
//     read(it, note.partial_state);
//     read(it, note.defi_interaction_nonce);
// }

// inline void write(std::vector<uint8_t>& buf, claim_note const& note)
// {
//     using serialize::write;
//     write(buf, note.deposit_value);
//     write(buf, note.bridge_id);
//     write(buf, note.partial_state);
//     write(buf, note.defi_interaction_nonce);
// }

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup