#pragma once
#include <stdlib/types/types.hpp>
#include "../native/bridge_call_data.hpp"
#include "./asset_id.hpp"
#include "../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types;

constexpr uint32_t input_asset_id_a_shift = DEFI_BRIDGE_ADDRESS_ID_LEN;
constexpr uint32_t input_asset_id_b_shift = input_asset_id_a_shift + DEFI_BRIDGE_INPUT_A_ASSET_ID_LEN;
constexpr uint32_t output_asset_id_a_shift = input_asset_id_b_shift + DEFI_BRIDGE_INPUT_B_ASSET_ID_LEN;
constexpr uint32_t output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
constexpr uint32_t bitconfig_shift = output_asset_id_b_shift + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
constexpr uint32_t aux_data_shift = bitconfig_shift + DEFI_BRIDGE_BITCONFIG_LEN;

struct bridge_call_data {

    /**
     * The 32-bit bit_config comprises the following:
     *
     * | bit | meaning |
     * |  0  | second_input_in_use   |
     * |  1  | second_output_in_use  |
     *
     * (The 0th bit is the least significant bit)
     *
     * Note: the first input and the first output are both always in_use when doing a defi deposit, so
     * we don't need a bit for first_input_in_use nor for first_output_in_use.
     */
    struct bit_config {
        bool_ct second_input_in_use;
        bool_ct second_output_in_use;

        bit_config(){};
        bit_config(Composer* composer, uint256_t const& bridge_call_data)
        {
            ASSERT(composer != nullptr);

            constexpr auto bitconfig_mask = (1ULL << DEFI_BRIDGE_BITCONFIG_LEN) - 1;
            uint32_t config_u32 = uint32_t((bridge_call_data >> bitconfig_shift) & bitconfig_mask);
            second_input_in_use = witness_ct(composer, config_u32 & 1ULL);
            second_output_in_use = witness_ct(composer, (config_u32 >> 1) & 1ULL);
        }

        suint_ct to_suint() const
        {
            const suint_ct bitconfig_scaling_factor(uint256_t(1) << bitconfig_shift);

            suint_ct result(second_input_in_use);
            result += suint_ct(second_output_in_use) * 2;
            result *= bitconfig_scaling_factor;
            return result;
        }
    };

    /**
     * The 248-bit bridge_call_data comprises the following:
     *
     *| aux_data | config | input_asset_id_b | output_asset_id_b | output_asst_id_a | input_asset_id_a |bridge_adress_id
     * ---------- -------- ------------------ ------------------- ------------------ ------------------ ----------------
     *      64        32            30                30                  30                  30                32
     */

    // 32-bit integer which maps to a 20-byte bridge contract address.
    suint_ct bridge_address_id;
    // Note: for virtual assets, the asset_id will be `1` in the 30th-bit, followed by the defi interaction nonce of the
    // interaction which created the asset
    // 30-bit asset_id of first input asset
    suint_ct input_asset_id_a;
    // 30-bit asset_id of second input asset
    suint_ct input_asset_id_b = 0;
    // Note: if the user expects virtual output assets to be returned from a bridge, they must at least specify an
    // output_asset_id of `100...00` (30-bits) to signal the asset's virtualness.
    // 30-bit asset_id of the first output asset.
    suint_ct output_asset_id_a;
    // 30-bit asset_id of the second output asset.
    suint_ct output_asset_id_b;

    // 32-bit bit configuration that describes input/output note structure of bridge:
    // see bit_config constructor for details.
    bit_config config;
    // 64-bit auxiliary data to be passed on to the bridge contract.
    suint_ct aux_data = 0;

    bridge_call_data(){};
    bridge_call_data(Composer* composer, const native::bridge_call_data& native_id)
        : bridge_call_data(composer, native_id.to_uint256_t())
    {}

    bridge_call_data(Composer* composer, uint256_t const& bridge_call_data)
    {
        // constants
        constexpr auto one = uint256_t(1);

        auto bridge_address_id_value = bridge_call_data & uint256_t((one << DEFI_BRIDGE_ADDRESS_ID_LEN) - 1);
        auto input_asset_id_a_value =
            (bridge_call_data >> input_asset_id_a_shift) & uint256_t((one << DEFI_BRIDGE_INPUT_A_ASSET_ID_LEN) - 1);
        auto input_asset_id_b_value =
            (bridge_call_data >> input_asset_id_b_shift) & uint256_t((one << DEFI_BRIDGE_INPUT_B_ASSET_ID_LEN) - 1);
        auto output_asset_id_a_value =
            (bridge_call_data >> output_asset_id_a_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN) - 1);
        auto output_asset_id_b_value =
            (bridge_call_data >> output_asset_id_b_shift) & uint256_t((one << DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN) - 1);
        auto aux_data_value = (bridge_call_data >> aux_data_shift) & uint256_t((one << DEFI_BRIDGE_AUX_DATA) - 1);

        bridge_address_id =
            suint_ct(witness_ct(composer, bridge_address_id_value), DEFI_BRIDGE_ADDRESS_ID_LEN, "bridge_address");
        input_asset_id_a = suint_ct(
            witness_ct(composer, input_asset_id_a_value), DEFI_BRIDGE_INPUT_A_ASSET_ID_LEN, "input_asset_id_a");
        input_asset_id_b = suint_ct(
            witness_ct(composer, input_asset_id_b_value), DEFI_BRIDGE_INPUT_B_ASSET_ID_LEN, "input_asset_id_b");
        output_asset_id_a = suint_ct(
            witness_ct(composer, output_asset_id_a_value), DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN, "output_asset_id_a");
        output_asset_id_b = suint_ct(
            witness_ct(composer, output_asset_id_b_value), DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN, "output_asset_id_b");
        aux_data = suint_ct(witness_ct(composer, aux_data_value), DEFI_BRIDGE_AUX_DATA, "aux_data");

        config = bit_config(composer, bridge_call_data);

        validate_bit_config();
    }

    void validate_bit_config()
    {
        // If the second input/output asset_id is nonzero, then it must be in-use.
        //
        // Note: If it's zero, it could be ETH (which has asset_id = 0), so we can't enforce the opposite direction of
        // implication (that if second_input_in_use, then it must be nonzero).
        // This does mean that if someone deposits ETH as the second input asset_id, but forgets to set
        // second_input_in_use to be `true`, then this circuit cannot catch this mistake. Therefore it's important that
        // every bridge contract validates the inputs it receives.
        (!input_asset_id_b.is_zero())
            .must_imply(config.second_input_in_use, "Expected second_input_in_use, given input_asset_id_b != 0");
        (!output_asset_id_b.is_zero())
            .must_imply(config.second_output_in_use, "Expected second_output_in_use, given output_asset_id_b != 0");

        /**
         * Note: since ETH's asset_id = 0, we have no 'null' asset_id, and so we can't enforce the opposite direction
         * of implication (that different asset_ids must imply that the second bridge input is in-use). The check - to
         * ensure differing asset_ids implies the second input to the bridge is in_use - can be done in the join-split
         * circuit, where it can check that the second input note to the circuit (not to be confused with the second
         * input to the bridge) is in_use.
         */
        config.second_input_in_use.must_imply(
            input_asset_id_a != input_asset_id_b,
            "input asset ids must be different for the second bridge input to be in-use");

        /**
         * Note:
         *
         *   - If either or both outputs from the bridge are expected to be real, the user must specify the output
         * asset_ids that they expect will be returned by the bridge. The below check ensures both real output asset_ids
         * must be _different_, in order for the bridge's second output to be in-use (otherwise, if they are the same
         * asset_id, then they should be combined (summed) into the same output asset).
         *
         *   - If either or  both outputs from the bridge are expected to be virtual, we don't enforce that the user
         * specifies the output asset_ids that they expect will be returned by the bridge. We allow the user to provide
         * 'placeholder' asset_ids. In fact, we enforce that the user provides virtual asset_id placeholders exactly
         * equal to `2**29`. Obviously, this means we can't then enforce that both output virtual asset_ids are
         * different, since they'll actually always be equal placeholder values.
         *
         * Note also: the virtual asset_id placeholders are contained in both the claim note's bridge_call_data and the
         * defi interaction note's bridge_call_data. The correct virtual asset_ids (which contain the defi interaction
         * nonce) are only calculated in the claim circuit, when generating the output notes of the claim.
         */
        const bool_ct first_output_virtual = get_asset_id_flag(output_asset_id_a);
        const bool_ct second_output_virtual = get_asset_id_flag(output_asset_id_b);
        const bool_ct both_outputs_real = (!first_output_virtual && !second_output_virtual);

        (config.second_output_in_use && both_outputs_real)
            .must_imply(output_asset_id_a != output_asset_id_b,
                        "real output asset ids must be different for the second bridge output to be in-use");

        const suint_ct virtual_asset_id_placeholder = suint_ct(1 << (MAX_NUM_ASSETS_BIT_LENGTH - 1)); // 2**29
        first_output_virtual.must_imply(output_asset_id_a == virtual_asset_id_placeholder,
                                        "output_asset_id_a detected as virtual, but has incorrect placeholder value");
        second_output_virtual.must_imply(output_asset_id_b == virtual_asset_id_placeholder,
                                         "output_asset_id_b detected as virtual, but has incorrect placeholder value");
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

inline std::ostream& operator<<(std::ostream& os, bridge_call_data::bit_config const& config)
{
    os << "  second_input_in_use: " << config.second_input_in_use << ",\n"
       << "  second_output_in_use: " << config.second_output_in_use << ",\n";
    return os;
}

inline std::ostream& operator<<(std::ostream& os, bridge_call_data const& bridge_call_data)
{
    os << "{\n"
       << "  bridge_address_id: " << bridge_call_data.bridge_address_id << ",\n"
       << "  input_asset_id_a: " << bridge_call_data.input_asset_id_a << ",\n"
       << "  input_asset_id_b: " << bridge_call_data.input_asset_id_b << ",\n"
       << "  output_asset_id_a: " << bridge_call_data.output_asset_id_a << ",\n"
       << "  output_asset_id_b: " << bridge_call_data.output_asset_id_b << ",\n"
       << bridge_call_data.config << "  aux_data: " << bridge_call_data.aux_data << "\n}";
    return os;
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace join_split_example