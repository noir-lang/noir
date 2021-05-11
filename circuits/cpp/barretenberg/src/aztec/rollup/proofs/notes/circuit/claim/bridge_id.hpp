#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../native/claim/bridge_id.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types::turbo;

struct bridge_id {
    // TODO: range constrain to be 20 bytes (160 bits)
    field_ct bridge_contract_address;
    // TODO: range constrain to be 2 bits (1 or 2)
    field_ct num_output_notes;

    // TODO: 32 bit range check
    field_ct input_asset_id;

    // TODO: 32 bit range check
    field_ct output_asset_id_a;

    // TODO: 32 bit range check (this should be 26-bit range check, right?)
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

        composer.create_range_constraint(bridge_contract_address_ct.witness_index, DEFI_BRIDGE_ADDRESS_BIT_LENGTH);
        composer.create_range_constraint(num_output_notes_ct.witness_index, DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN);
        composer.create_range_constraint(input_asset_id_ct.witness_index, DEFI_BRIDGE_INPUT_ASSET_ID_LEN);
        composer.create_range_constraint(output_asset_id_a_ct.witness_index, DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN);
        composer.create_range_constraint(output_asset_id_b_ct.witness_index, DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN);

        return { bridge_contract_address_ct.normalize(),
                 num_output_notes_ct.normalize(),
                 input_asset_id_ct.normalize(),
                 output_asset_id_a_ct.normalize(),
                 output_asset_id_b_ct.normalize() };
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
       << "  output_asset_id_b: " << bridge_id.output_asset_id_a << "\n}";
    return os;
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup