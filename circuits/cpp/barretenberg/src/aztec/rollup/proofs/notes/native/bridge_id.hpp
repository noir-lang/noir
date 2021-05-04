#pragma once
#include <common/serialize.hpp>
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
};

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup