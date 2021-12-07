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

    /**
     * | bit | meaning |
     * |  0  | firstInputAssetVirtual (currently always false) |
     * |  1  | secondInputAssetVirtual |
     * |  2  | firstOutputAssetVirtual |
     * |  3  | secondOutputAssetVirtual |
     * |  4  | secondInputValid | is first output note valid and real?
     * |  5  | secondOutputValid | is second output note valid and real?
     */
    struct bit_config {
        bool_ct first_input_asset_virtual;
        bool_ct second_input_asset_virtual;
        bool_ct first_output_asset_virtual;
        bool_ct second_output_asset_virtual;
        bool_ct second_input_valid;
        bool_ct second_output_valid;

        bit_config(){};
        bit_config(Composer* composer, uint256_t const& bridge_id)
        {
            ASSERT(composer != nullptr);
            constexpr auto input_asset_id_shift = DEFI_BRIDGE_ADDRESS_ID_LEN;
            constexpr auto output_asset_id_a_shift = input_asset_id_shift + DEFI_BRIDGE_INPUT_ASSET_ID_LEN;
            constexpr auto output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
            constexpr auto opening_nonce_shift = output_asset_id_b_shift + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
            constexpr auto bitconfig_shift = opening_nonce_shift + DEFI_BRIDGE_OPENING_NONCE_LEN;

            constexpr auto bitconfig_mask = (1ULL << DEFI_BRIDGE_BITCONFIG_LEN) - 1;
            uint256_t config_u256 = (bridge_id >> bitconfig_shift) & bitconfig_mask;
            first_input_asset_virtual = witness_ct(composer, config_u256 & 1ULL);
            second_input_asset_virtual = witness_ct(composer, (config_u256 >> 1) & 1ULL);
            first_output_asset_virtual = witness_ct(composer, (config_u256 >> 2) & 1ULL);
            second_output_asset_virtual = witness_ct(composer, (config_u256 >> 3) & 1ULL);
            second_input_valid = witness_ct(composer, (config_u256 >> 4) & 1ULL);
            second_output_valid = witness_ct(composer, (config_u256 >> 5) & 1ULL);

            // for now we do not support making the first input asset virtual. May change in a future circuit update
            first_input_asset_virtual.assert_equal(false, "first input asset is set to virtual!");
            // for now we do not support making the first output asset virtual. May change in a future circuit update
            first_output_asset_virtual.assert_equal(false, "first output asset is set to virtual!");
            second_input_asset_virtual.must_imply(
                !second_input_valid, "both second_input_asset_virtual AND second_input_valid are set to true");
            second_output_asset_virtual.must_imply(
                !second_output_valid, "both second_output_asset_virtual AND second_output_valid are set to true");
        }

        suint_ct to_suint() const
        {
            constexpr auto input_asset_id_shift = DEFI_BRIDGE_ADDRESS_ID_LEN;
            constexpr auto output_asset_id_a_shift = input_asset_id_shift + DEFI_BRIDGE_INPUT_ASSET_ID_LEN;
            constexpr auto output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
            constexpr auto opening_nonce_shift = output_asset_id_b_shift + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
            constexpr auto bitconfig_shift = opening_nonce_shift + DEFI_BRIDGE_OPENING_NONCE_LEN;

            const suint_ct bitconfig_scaling_factor(uint256_t(1) << bitconfig_shift);

            suint_ct result(first_input_asset_virtual);
            result += suint_ct(second_input_asset_virtual) * 2;
            result += suint_ct(first_output_asset_virtual) * 4;
            result += suint_ct(second_output_asset_virtual) * 8;
            result += suint_ct(second_input_valid) * 16;
            result += suint_ct(second_output_valid) * 32;
            result *= bitconfig_scaling_factor;
            return result;
        }
    };
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

    // bit configuration that describes input/output note structure of bridge:
    // see bit_config constructor for details.
    // Is 32-bits wide.
    bit_config config;
    // 32-bit auxiliary data to be passed on to the bridge contract.
    suint_ct aux_data = 0;

    bridge_id(){};
    bridge_id(Composer* composer, const native::bridge_id& native_id)
        : bridge_id(composer, native_id.to_uint256_t())
    {}

    bridge_id(Composer* composer, uint256_t const& bridge_id)
    {
        // constants
        constexpr auto input_asset_id_shift = DEFI_BRIDGE_ADDRESS_ID_LEN;
        constexpr auto output_asset_id_a_shift = input_asset_id_shift + DEFI_BRIDGE_INPUT_ASSET_ID_LEN;
        constexpr auto output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
        constexpr auto opening_nonce_shift = output_asset_id_b_shift + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
        constexpr auto bitconfig_offset = opening_nonce_shift + DEFI_BRIDGE_OPENING_NONCE_LEN;
        constexpr auto aux_data_shift = bitconfig_offset + DEFI_BRIDGE_BITCONFIG_LEN;
        constexpr auto one = uint256_t(1);

        auto bridge_address_id_value = bridge_id & uint256_t((one << DEFI_BRIDGE_ADDRESS_ID_LEN) - 1);
        auto input_asset_id_value =
            (bridge_id >> input_asset_id_shift) & uint256_t((one << DEFI_BRIDGE_INPUT_ASSET_ID_LEN) - 1);
        auto output_asset_id_a_value =
            (bridge_id >> output_asset_id_a_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN) - 1);
        auto output_asset_id_b_value =
            (bridge_id >> output_asset_id_b_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN) - 1);
        auto opening_nonce_value =
            (bridge_id >> opening_nonce_shift) & uint256_t((one << DEFI_BRIDGE_OPENING_NONCE_LEN) - 1);
        auto aux_data_value = (bridge_id >> aux_data_shift) & uint256_t((one << DEFI_BRIDGE_AUX_DATA) - 1);

        bridge_address_id =
            suint_ct(witness_ct(composer, bridge_address_id_value), DEFI_BRIDGE_ADDRESS_ID_LEN, "bridge_address");
        input_asset_id =
            suint_ct(witness_ct(composer, input_asset_id_value), DEFI_BRIDGE_INPUT_ASSET_ID_LEN, "input_asset_id");
        output_asset_id_a = suint_ct(
            witness_ct(composer, output_asset_id_a_value), DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN, "output_asset_id_a");
        output_asset_id_b = suint_ct(
            witness_ct(composer, output_asset_id_b_value), DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN, "output_asset_id_b");
        opening_nonce = suint_ct(witness_ct(composer, opening_nonce_value), DEFI_TREE_DEPTH, "opening_nonce");
        aux_data = suint_ct(witness_ct(composer, aux_data_value), DEFI_BRIDGE_AUX_DATA, "aux_data");

        config = bit_config(composer, bridge_id);

        config.second_output_valid.must_imply(output_asset_id_a != output_asset_id_b,
                                              "second_output_valid == true AND both output asset ids are identical");
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
                      (opening_nonce * suint_ct(one << (opening_nonce_shift))) + config.to_suint() +
                      (aux_data * suint_ct(one << (aux_data_shift)));
        return result;
    }
};

inline std::ostream& operator<<(std::ostream& os, bridge_id::bit_config const& config)
{
    os << "  first_input_asset_virtual: " << config.first_input_asset_virtual << ",\n"
       << "  second_input_asset_virtual: " << config.second_input_asset_virtual << ",\n"
       << "  first_output_asset_virtual: " << config.first_output_asset_virtual << ",\n"
       << "  second_output_asset_virtual: " << config.second_output_asset_virtual << ",\n"
       << "  second_input_valid: " << config.second_input_valid << ",\n"
       << "  second_output_valid: " << config.second_output_valid << ",\n";
    return os;
}

inline std::ostream& operator<<(std::ostream& os, bridge_id const& bridge_id)
{
    os << "{\n"
       << "  bridge_address_id: " << bridge_id.bridge_address_id << ",\n"
       << "  input_asset_id: " << bridge_id.input_asset_id << ",\n"
       << "  output_asset_id_a: " << bridge_id.output_asset_id_a << ",\n"
       << "  output_asset_id_b: " << bridge_id.output_asset_id_b << ",\n"
       << "  opening_nonce: " << bridge_id.opening_nonce << ",\n"
       << bridge_id.config << "  aux_data: " << bridge_id.aux_data << "\n}";
    return os;
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup