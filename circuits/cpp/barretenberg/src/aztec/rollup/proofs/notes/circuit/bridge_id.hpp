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
    field_ct bridge_contract_address;
    field_ct num_output_notes;
    field_ct input_asset_id;
    field_ct output_asset_id_a;
    field_ct output_asset_id_b;

    static bridge_id from_uint256_t(Composer& composer, uint256_t const& bridge_id)
    {
        // constants
        constexpr auto num_output_notes_shift = DEFI_BRIDGE_ADDRESS_BIT_LENGTH;
        constexpr auto input_asset_id_shift = num_output_notes_shift + DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN;
        constexpr auto output_asset_id_a_shift = input_asset_id_shift + DEFI_BRIDGE_INPUT_ASSET_ID_LEN;
        constexpr auto output_asset_id_b_shift = output_asset_id_a_shift + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
        constexpr auto big_one = uint256_t(1);

        auto bridge_contract_address = bridge_id & uint256_t((big_one << DEFI_BRIDGE_ADDRESS_BIT_LENGTH) - 1);
        auto input_asset_id =
            (bridge_id >> input_asset_id_shift) & uint256_t((big_one << DEFI_BRIDGE_INPUT_ASSET_ID_LEN) - 1);
        auto num_output_notes =
            (bridge_id >> num_output_notes_shift) & uint256_t((big_one << DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN) - 1);
        auto output_asset_id_a =
            (bridge_id >> output_asset_id_a_shift) & uint256_t((big_one << DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN) - 1);
        auto output_asset_id_b =
            (bridge_id >> output_asset_id_b_shift) & uint256_t((big_one << DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN) - 1);

        field_ct bridge_contract_address_ct = witness_ct(&composer, bridge_contract_address);
        field_ct num_output_notes_ct = witness_ct(&composer, num_output_notes);
        field_ct input_asset_id_ct = witness_ct(&composer, input_asset_id);
        field_ct output_asset_id_a_ct = witness_ct(&composer, output_asset_id_a);
        field_ct output_asset_id_b_ct = witness_ct(&composer, output_asset_id_b);

        bridge_contract_address_ct.create_range_constraint(DEFI_BRIDGE_ADDRESS_BIT_LENGTH);
        num_output_notes_ct.create_range_constraint(DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN);
        input_asset_id_ct.create_range_constraint(DEFI_BRIDGE_INPUT_ASSET_ID_LEN);
        output_asset_id_a_ct.create_range_constraint(DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN);
        output_asset_id_b_ct.create_range_constraint(DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN);

        return { bridge_contract_address_ct,
                 num_output_notes_ct,
                 input_asset_id_ct,
                 output_asset_id_a_ct,
                 output_asset_id_b_ct };
    }

    field_ct to_field() const
    {
        auto result = bridge_contract_address +
                      (num_output_notes * field_ct(uint256_t(1) << DEFI_BRIDGE_ADDRESS_BIT_LENGTH)) +
                      (input_asset_id *
                       field_ct(uint256_t(1) << (DEFI_BRIDGE_ADDRESS_BIT_LENGTH + DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN))) +
                      (output_asset_id_a *
                       field_ct(uint256_t(1) << (DEFI_BRIDGE_ADDRESS_BIT_LENGTH + DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN +
                                                 DEFI_BRIDGE_INPUT_ASSET_ID_LEN))) +
                      (output_asset_id_b *
                       field_ct(uint256_t(1) << (DEFI_BRIDGE_ADDRESS_BIT_LENGTH + DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN +
                                                 DEFI_BRIDGE_INPUT_ASSET_ID_LEN + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN)));

        return result;
    }
};

inline std::ostream& operator<<(std::ostream& os, bridge_id const& bridge_id)
{
    os << "{\n"
       << "  bridge_contract_address: " << bridge_id.bridge_contract_address << ",\n"
       << "  num_output_notes: " << bridge_id.num_output_notes << ",\n"
       << "  input_asset_id: " << bridge_id.input_asset_id << ",\n"
       << "  output_asset_id_a: " << bridge_id.output_asset_id_a << ",\n"
       << "  output_asset_id_b: " << bridge_id.output_asset_id_b << "\n}";
    return os;
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup