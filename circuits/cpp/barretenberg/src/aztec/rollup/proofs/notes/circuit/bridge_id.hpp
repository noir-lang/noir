#pragma once
#include <stdlib/types/turbo.hpp>
#include "../native/bridge_id.hpp"
#include "../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types::turbo;

struct bridge_id {
    // 32-bit integer which maps to a 20-byte bridge contract address.
    suint_ct bridge_address_id;
    // 30-bit asset_id of the input asset.
    suint_ct input_asset_id;
    // 30-bit asset_id of the first output asset.
    suint_ct output_asset_id_a;
    // 30-bit asset_id of the second output asset.
    suint_ct output_asset_id_b;
    // Defi interaction nonce of opening a loan/LP position.
    suint_ct opening_nonce = 0;
    // True if a bridge interaction returns two valid assets.
    bool_ct second_asset_valid = false;
    // True if the defi interaction requires creation of a virtual_note.
    bool_ct second_asset_virtual = false;
    // 32-bit auxiliary data to be passed on to the bridge contract.
    suint_ct aux_data = 0;

    static bridge_id from_uint256_t(Composer& composer, uint256_t const& bridge_id)
    {
        // constants
        constexpr auto input_asset_id_shift = DEFI_BRIDGE_ADDRESS_ID_LEN;
        constexpr auto output_asset_id_a_shift = input_asset_id_shift + DEFI_BRIDGE_INPUT_ASSET_ID_LEN;
        constexpr auto output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
        constexpr auto opening_nonce_shift = output_asset_id_b_shift + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
        constexpr auto bitconfig_offset = opening_nonce_shift + DEFI_BRIDGE_OPENING_NONCE_LEN;
        constexpr auto aux_data_shift = bitconfig_offset + DEFI_BRIDGE_BITCONFIG_LEN;
        constexpr auto one = uint256_t(1);

        auto bridge_address_id = bridge_id & uint256_t((one << DEFI_BRIDGE_ADDRESS_ID_LEN) - 1);
        auto input_asset_id =
            (bridge_id >> input_asset_id_shift) & uint256_t((one << DEFI_BRIDGE_INPUT_ASSET_ID_LEN) - 1);
        auto output_asset_id_a =
            (bridge_id >> output_asset_id_a_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN) - 1);
        auto output_asset_id_b =
            (bridge_id >> output_asset_id_b_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN) - 1);
        auto opening_nonce = (bridge_id >> opening_nonce_shift) & uint256_t((one << DEFI_BRIDGE_OPENING_NONCE_LEN) - 1);
        auto second_asset_valid = ((bridge_id >> bitconfig_offset) & one) == one;
        auto second_asset_virtual = ((bridge_id >> (bitconfig_offset + 1)) & one) == one;
        auto aux_data = (bridge_id >> aux_data_shift) & uint256_t((one << DEFI_BRIDGE_AUX_DATA) - 1);

        suint_ct bridge_address_id_ct(
            witness_ct(&composer, bridge_address_id), DEFI_BRIDGE_ADDRESS_ID_LEN, "bridge_address");
        suint_ct input_asset_id_ct(
            witness_ct(&composer, input_asset_id), DEFI_BRIDGE_INPUT_ASSET_ID_LEN, "input_asset_id");
        suint_ct output_asset_id_a_ct(
            witness_ct(&composer, output_asset_id_a), DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN, "output_asset_id_a");
        suint_ct output_asset_id_b_ct(
            witness_ct(&composer, output_asset_id_b), DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN, "output_asset_id_b");
        suint_ct opening_nonce_ct(witness_ct(&composer, opening_nonce), DEFI_TREE_DEPTH, "opening_nonce");
        bool_ct second_asset_valid_ct = witness_ct(&composer, second_asset_valid);
        bool_ct second_asset_virtual_ct = witness_ct(&composer, second_asset_virtual);
        suint_ct aux_data_ct(witness_ct(&composer, aux_data), DEFI_BRIDGE_AUX_DATA, "aux_data");

        return { bridge_address_id_ct, input_asset_id_ct,     output_asset_id_a_ct,    output_asset_id_b_ct,
                 opening_nonce_ct,     second_asset_valid_ct, second_asset_virtual_ct, aux_data_ct };
    }

    suint_ct to_safe_uint() const
    {
        // constants
        constexpr auto input_asset_id_shift = DEFI_BRIDGE_ADDRESS_ID_LEN;
        constexpr auto output_asset_id_a_shift = input_asset_id_shift + DEFI_BRIDGE_INPUT_ASSET_ID_LEN;
        constexpr auto output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
        constexpr auto opening_nonce_shift = output_asset_id_b_shift + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
        constexpr auto bitconfig_offset = opening_nonce_shift + DEFI_BRIDGE_OPENING_NONCE_LEN;
        constexpr auto aux_data_shift = bitconfig_offset + DEFI_BRIDGE_BITCONFIG_LEN;
        constexpr auto one = uint256_t(1);

        auto result = bridge_address_id + (input_asset_id * suint_ct(one << (input_asset_id_shift))) +
                      (output_asset_id_a * suint_ct(one << (output_asset_id_a_shift))) +
                      (output_asset_id_b * suint_ct(one << (output_asset_id_b_shift))) +
                      (opening_nonce * suint_ct(one << (opening_nonce_shift))) +
                      (suint_ct(second_asset_valid) * suint_ct(one << bitconfig_offset)) +
                      (suint_ct(second_asset_virtual) * suint_ct(one << (bitconfig_offset + 1))) +
                      (aux_data * suint_ct(one << (aux_data_shift)));

        return result;
    }
};

inline std::ostream& operator<<(std::ostream& os, bridge_id const& bridge_id)
{
    os << "{\n"
       << "  bridge_address_id: " << bridge_id.bridge_address_id << ",\n"
       << "  input_asset_id: " << bridge_id.input_asset_id << ",\n"
       << "  output_asset_id_a: " << bridge_id.output_asset_id_a << ",\n"
       << "  output_asset_id_b: " << bridge_id.output_asset_id_b << "\n"
       << "  opening_nonce: " << bridge_id.opening_nonce << "\n}"
       << "  second_asset_valid: " << bridge_id.second_asset_valid << ",\n"
       << "  second_asset_virtual: " << bridge_id.second_asset_virtual << ",\n"
       << "  aux_data: " << bridge_id.aux_data << "\n}";
    return os;
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup