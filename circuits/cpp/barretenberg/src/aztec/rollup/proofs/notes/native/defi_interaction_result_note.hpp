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

grumpkin::g1::affine_element encrypt_note(defi_interaction_result_note const& note);

inline bool operator==(defi_interaction_result_note const& lhs, defi_interaction_result_note const& rhs)
{
    return lhs.bridge_id == rhs.bridge_id && lhs.total_input_value == rhs.total_input_value &&
           lhs.total_output_a_value == rhs.total_output_a_value &&
           lhs.total_output_b_value == rhs.total_output_b_value && lhs.interaction_nonce == rhs.interaction_nonce &&
           lhs.interaction_result == rhs.interaction_result;
}

inline std::ostream& operator<<(std::ostream& os, defi_interaction_result_note const& note)
{
    os << "{ bridge_contract_address: " << note.bridge_id.bridge_contract_address
       << ", input_asset_id: " << note.bridge_id.input_asset_id
       << ", output_asset_id_a: " << note.bridge_id.output_asset_id_a
       << ", output_asset_id_a: " << note.bridge_id.output_asset_id_a
       << ", total_input_value: " << note.total_input_value << ", total_output_a_value: " << note.total_output_a_value
       << ", total_output_b_value: " << note.total_output_b_value << ", interaction_nonce: " << note.interaction_nonce
       << ", interaction_result: " << note.interaction_result << " }";
    return os;
}

// inline void read(uint8_t const*& it, defi_interaction_result_note& note)
// {
//     using serialize::read;
//     read(it, note.bridge_id);
//     read(it, note.total_input_value);
//     read(it, note.total_output_a_value);
//     read(it, note.total_output_b_value);
//     read(it, note.interaction_nonce);
//     read(it, note.interaction_result);
// }

// inline void write(std::vector<uint8_t>& buf, defi_interaction_result_note const& note)
// {
//     using serialize::write;
//     write(buf, note.bridge_id);
//     write(buf, note.total_input_value);
//     write(buf, note.total_output_a_value);
//     write(buf, note.total_output_b_value);
//     write(buf, note.interaction_nonce);
//     write(buf, note.interaction_result);
// }

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup