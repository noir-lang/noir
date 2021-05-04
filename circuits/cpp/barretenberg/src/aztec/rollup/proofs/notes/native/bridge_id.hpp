#pragma once
#include <common/serialize.hpp>
#include "../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

struct bridge_id {
    uint256_t bridge_contract_address;
    uint32_t num_output_notes;
    uint32_t input_asset_id;
    uint32_t output_asset_id_a;
    uint32_t output_asset_id_b;

    barretenberg::fr to_field() const
    {

        // check if the address is 160 bits, num_output_notes is 2 bits and output_asset_id_b is 26 bits
        bool address_check = ((bridge_contract_address >> ADDRESS_BIT_LENGTH) == 0);
        bool num_output_notes_check = ((num_output_notes >> NUM_OUTPUT_NOTES_LEN) == 0);
        bool output_asset_id_b_check = ((output_asset_id_b >> OUTPUT_B_ASSET_ID_LEN) == 0);

        if (!(address_check && num_output_notes_check && output_asset_id_b_check)) {
            throw("Structure of the bridge_id incorrect!");
        }

        uint256_t result = bridge_contract_address | (static_cast<uint256_t>(num_output_notes) << ADDRESS_BIT_LENGTH) |
                           (static_cast<uint256_t>(input_asset_id) << (ADDRESS_BIT_LENGTH + NUM_OUTPUT_NOTES_LEN)) |
                           (static_cast<uint256_t>(output_asset_id_a)
                            << (ADDRESS_BIT_LENGTH + NUM_OUTPUT_NOTES_LEN + INPUT_ASSET_ID_LEN)) |
                           (static_cast<uint256_t>(input_asset_id)
                            << (ADDRESS_BIT_LENGTH + NUM_OUTPUT_NOTES_LEN + 2 * INPUT_ASSET_ID_LEN));

        return barretenberg::fr(result);
    }
};

inline bool operator==(bridge_id const& lhs, bridge_id const& rhs)
{
    return lhs.bridge_contract_address == rhs.bridge_contract_address && lhs.input_asset_id == rhs.input_asset_id &&
           lhs.num_output_notes == rhs.num_output_notes && lhs.output_asset_id_a == rhs.output_asset_id_a &&
           lhs.output_asset_id_b == rhs.output_asset_id_b;
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup