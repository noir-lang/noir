#pragma once
#include <stdlib/types/turbo.hpp>
#include "../native/bridge_id.hpp"
#include "../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types::turbo;

constexpr uint32_t input_asset_id_a_shift = DEFI_BRIDGE_ADDRESS_ID_LEN;
constexpr uint32_t input_asset_id_b_shift = input_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
constexpr uint32_t output_asset_id_a_shift = input_asset_id_b_shift + DEFI_BRIDGE_INPUT_A_ASSET_ID_LEN;
constexpr uint32_t output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
constexpr uint32_t bitconfig_shift = output_asset_id_b_shift + DEFI_BRIDGE_INPUT_B_ASSET_ID_LEN;
constexpr uint32_t aux_data_shift = bitconfig_shift + DEFI_BRIDGE_BITCONFIG_LEN;

struct bridge_id {

    /**
     * The 32-bit bit_config comprises the following:
     *
     * | bit | meaning |
     * |  0  | firstInputVirtual   | (currently always false)
     * |  1  | secondInputVirtual  |
     * |  2  | firstOutputVirtual  | (currently always false)
     * |  3  | secondOutputVirtual |
     * |  4  | secondInputReal     |
     * |  5  | secondOutputReal    |
     *
     * (The 0th bit is the least significant bit)
     */
    struct bit_config {
        bool_ct first_input_virtual;
        bool_ct second_input_virtual;
        bool_ct first_output_virtual;
        bool_ct second_output_virtual;
        bool_ct second_input_real;
        bool_ct second_output_real;

        bit_config(){};
        bit_config(Composer* composer, uint256_t const& bridge_id)
        {
            ASSERT(composer != nullptr);

            constexpr auto bitconfig_mask = (1ULL << DEFI_BRIDGE_BITCONFIG_LEN) - 1;
            uint32_t config_u32 = uint32_t((bridge_id >> bitconfig_shift) & bitconfig_mask);
            first_input_virtual = witness_ct(composer, config_u32 & 1ULL);
            second_input_virtual = witness_ct(composer, (config_u32 >> 1) & 1ULL);
            first_output_virtual = witness_ct(composer, (config_u32 >> 2) & 1ULL);
            second_output_virtual = witness_ct(composer, (config_u32 >> 3) & 1ULL);
            second_input_real = witness_ct(composer, (config_u32 >> 4) & 1ULL);
            second_output_real = witness_ct(composer, (config_u32 >> 5) & 1ULL);

            // Prevent contradictions:
            (second_input_virtual & second_input_real)
                .assert_equal(false, "Contradiction: second_input_virtual AND second_input_real cannot both be true");
            (second_output_virtual & second_output_real)
                .assert_equal(false, "Contradiction: second_output_virtual AND second_output_real cannot both be true");
        }

        suint_ct to_suint() const
        {
            const suint_ct bitconfig_scaling_factor(uint256_t(1) << bitconfig_shift);

            suint_ct result(first_input_virtual);
            result += suint_ct(second_input_virtual) * 2;
            result += suint_ct(first_output_virtual) * 4;
            result += suint_ct(second_output_virtual) * 8;
            result += suint_ct(second_input_real) * 16;
            result += suint_ct(second_output_real) * 32;
            result *= bitconfig_scaling_factor;
            return result;
        }
    };

    /**
     * The 250-bit bridge_id comprises the following:
     *
     * | aux_data | config | input_asset_id_b | output_asset_id_b | output_asst_id_a | input_asset_id_a |
     * bridge_adress_id |
     *  ---------- -------- --------------- ------------------- ------------------ ---------------- ------------------
     *      64        32          32                30                  30                30                32
     */

    // 32-bit integer which maps to a 20-byte bridge contract address.
    // N.B. for virtual assets, the asset_id will be the defi interaction nonce that created the asset
    suint_ct bridge_address_id;
    // 30-bit asset_id of first input asset
    suint_ct input_asset_id_a;
    // 30-bit asset_id of second input asset
    suint_ct input_asset_id_b = 0;
    // 30-bit asset_id of the first output asset.
    suint_ct output_asset_id_a;
    // 30-bit asset_id of the second output asset.
    suint_ct output_asset_id_b;

    // 32-bit bit configuration that describes input/output note structure of bridge:
    // see bit_config constructor for details.
    bit_config config;
    // 64-bit auxiliary data to be passed on to the bridge contract.
    suint_ct aux_data = 0;

    bridge_id(){};
    bridge_id(Composer* composer, const native::bridge_id& native_id)
        : bridge_id(composer, native_id.to_uint256_t())
    {}

    bridge_id(Composer* composer, uint256_t const& bridge_id)
    {
        // constants
        constexpr auto one = uint256_t(1);

        auto bridge_address_id_value = bridge_id & uint256_t((one << DEFI_BRIDGE_ADDRESS_ID_LEN) - 1);
        auto input_asset_id_a_value =
            (bridge_id >> input_asset_id_a_shift) & uint256_t((one << DEFI_BRIDGE_INPUT_A_ASSET_ID_LEN) - 1);
        auto input_asset_id_b_value =
            (bridge_id >> input_asset_id_b_shift) & uint256_t((one << DEFI_BRIDGE_INPUT_B_ASSET_ID_LEN) - 1);
        auto output_asset_id_a_value =
            (bridge_id >> output_asset_id_a_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN) - 1);
        auto output_asset_id_b_value =
            (bridge_id >> output_asset_id_b_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN) - 1);
        auto aux_data_value = (bridge_id >> aux_data_shift) & uint256_t((one << DEFI_BRIDGE_AUX_DATA) - 1);

        // Given the above bit-shifting, the below range constraints aren't strictly necessary, but they don't hurt.
        bridge_address_id =
            suint_ct(witness_ct(composer, bridge_address_id_value), DEFI_BRIDGE_ADDRESS_ID_LEN, "bridge_address");
        input_asset_id_a = suint_ct(
            witness_ct(composer, input_asset_id_a_value), DEFI_BRIDGE_INPUT_A_ASSET_ID_LEN, "input_asset_id_a");
        input_asset_id_b = suint_ct(witness_ct(composer, input_asset_id_b_value), DEFI_TREE_DEPTH, "input_asset_id_b");
        output_asset_id_a = suint_ct(
            witness_ct(composer, output_asset_id_a_value), DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN, "output_asset_id_a");
        output_asset_id_b = suint_ct(
            witness_ct(composer, output_asset_id_b_value), DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN, "output_asset_id_b");
        aux_data = suint_ct(witness_ct(composer, aux_data_value), DEFI_BRIDGE_AUX_DATA, "aux_data");

        config = bit_config(composer, bridge_id);

        config.second_output_real.must_imply(output_asset_id_a != output_asset_id_b,
                                             "second_output_real == true AND both output asset ids are identical");
    }

    suint_ct to_safe_uint() const
    {
        // constants
        constexpr auto one = uint256_t(1);

        auto result = bridge_address_id + (input_asset_id_a * suint_ct(one << (input_asset_id_a_shift))) +
                      (input_asset_id_b * suint_ct(one << (input_asset_id_b_shift))) +
                      (output_asset_id_a * suint_ct(one << (output_asset_id_a_shift))) +
                      (output_asset_id_b * suint_ct(one << (output_asset_id_b_shift))) + config.to_suint() +
                      (aux_data * suint_ct(one << (aux_data_shift)));
        return result;
    }
};

inline std::ostream& operator<<(std::ostream& os, bridge_id::bit_config const& config)
{
    os << "  first_input_virtual: " << config.first_input_virtual << ",\n"
       << "  second_input_virtual: " << config.second_input_virtual << ",\n"
       << "  first_output_virtual: " << config.first_output_virtual << ",\n"
       << "  second_output_virtual: " << config.second_output_virtual << ",\n"
       << "  second_input_real: " << config.second_input_real << ",\n"
       << "  second_output_real: " << config.second_output_real << ",\n";
    return os;
}

inline std::ostream& operator<<(std::ostream& os, bridge_id const& bridge_id)
{
    os << "{\n"
       << "  bridge_address_id: " << bridge_id.bridge_address_id << ",\n"
       << "  input_asset_id_a: " << bridge_id.input_asset_id_a << ",\n"
       << "  input_asset_id_b: " << bridge_id.input_asset_id_b << ",\n"
       << "  output_asset_id_a: " << bridge_id.output_asset_id_a << ",\n"
       << "  output_asset_id_b: " << bridge_id.output_asset_id_b << ",\n"
       << bridge_id.config << "  aux_data: " << bridge_id.aux_data << "\n}";
    return os;
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup