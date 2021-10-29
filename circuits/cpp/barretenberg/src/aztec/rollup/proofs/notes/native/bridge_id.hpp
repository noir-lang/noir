#pragma once
#include <common/serialize.hpp>
#include "../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

/**
 * The bridge_id structure (with bit-lengths) is defined as follows:
 *
 * (auxData || bitConfig || openingNonce || outputAssetB || outputAssetA || inputAsset || bridgeAddressId)
 *     64          32            32              30              30             30              32
 *
 * bridgeAddressId : 32-bit integer mapped to a bridge contract address
 * inputAsset      : Input asset id
 * outputAssetA    : First output asset id
 * outputAssetB    : Second output asset id
 * openingNonce    : Defi interaction nonce when a loan/LP position was opened
 * bitConfig       : 32-bit configuration (0 || 0 || ... || 0 || secondAssetVirtual || secondAssetValid)
 * auxData         : Additional (optional) data to be used by the bridge contract.
 *
 */
struct bridge_id {
    uint32_t bridge_address_id;
    uint32_t input_asset_id;
    uint32_t output_asset_id_a;
    uint32_t output_asset_id_b;
    uint32_t opening_nonce = 0;
    bool second_asset_valid = false;
    bool second_asset_virtual = false;
    uint64_t aux_data = 0;

    uint256_t to_uint256_t() const
    {
        // The bridge contract address is the 160-bit address mapped to a 32-bit integer just like asset ids.
        // check if the asset ids are 30 bits.
        bool input_asset_id_check = ((input_asset_id >> DEFI_BRIDGE_INPUT_ASSET_ID_LEN) == 0);
        bool output_asset_id_a_check = ((output_asset_id_a >> DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN) == 0);
        bool output_asset_id_b_check = ((output_asset_id_b >> DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN) == 0);

        if (!(input_asset_id_check && output_asset_id_a_check && output_asset_id_b_check)) {
            barretenberg::errors::throw_or_abort("Structure of the bridge_id incorrect!");
        }

        constexpr uint32_t input_asset_id_offset = DEFI_BRIDGE_ADDRESS_ID_LEN;
        constexpr uint32_t output_asset_id_a_offset = input_asset_id_offset + DEFI_BRIDGE_INPUT_ASSET_ID_LEN;
        constexpr uint32_t output_asset_id_b_offset = output_asset_id_a_offset + DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN;
        constexpr uint32_t opening_nonce_offset = output_asset_id_b_offset + DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN;
        constexpr uint32_t bitconfig_offset = opening_nonce_offset + DEFI_BRIDGE_OPENING_NONCE_LEN;
        constexpr uint32_t aux_data_offset = bitconfig_offset + DEFI_BRIDGE_BITCONFIG_LEN;

        uint256_t result = static_cast<uint256_t>(bridge_address_id) +
                           (static_cast<uint256_t>(input_asset_id) << input_asset_id_offset) +
                           (static_cast<uint256_t>(output_asset_id_a) << output_asset_id_a_offset) +
                           (static_cast<uint256_t>(output_asset_id_b) << output_asset_id_b_offset) +
                           (static_cast<uint256_t>(opening_nonce) << opening_nonce_offset) +
                           (static_cast<uint256_t>(second_asset_valid) << bitconfig_offset) +
                           (static_cast<uint256_t>(second_asset_virtual) << (bitconfig_offset + 1)) +
                           (static_cast<uint256_t>(aux_data) << aux_data_offset);

        return result;
    }

    operator uint256_t() { return to_uint256_t(); }

    bool operator==(bridge_id const&) const = default;
};

inline std::ostream& operator<<(std::ostream& os, bridge_id const& bridge_id)
{
    os << "{\n"
       << "  bridge_address_id: " << bridge_id.bridge_address_id << ",\n"
       << "  input_asset_id: " << bridge_id.input_asset_id << ",\n"
       << "  output_asset_id_a: " << bridge_id.output_asset_id_a << ",\n"
       << "  output_asset_id_b: " << bridge_id.output_asset_id_a << "\n"
       << "  opening_nonce: " << bridge_id.opening_nonce << "\n}"
       << "  second_asset_valid: " << bridge_id.second_asset_valid << ",\n"
       << "  second_asset_virtual: " << bridge_id.second_asset_virtual << ",\n"
       << "  aux_data: " << bridge_id.aux_data << "\n}";
    return os;
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup