#pragma once
#include <stdlib/types/turbo.hpp>
#include "../rollup/rollup_proof_data.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace plonk::stdlib::types::turbo;

struct root_rollup_proof_data : rollup::rollup_proof_data {
    std::array<grumpkin::g1::affine_element, NUM_BRIDGE_CALLS_PER_BLOCK> defi_interaction_notes;
    uint256_t previous_defi_interaction_hash;

    root_rollup_proof_data(std::vector<uint8_t> const& proof_data);
    root_rollup_proof_data(std::vector<fr> const& public_inputs);
};

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
