#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

using namespace plonk::stdlib::types::turbo;

namespace RollupProofFields {
enum {
    ROLLUP_ID = 0,
    ROLLUP_SIZE = 1,
    DATA_START_INDEX = 2,
    OLD_DATA_ROOT = 3,
    NEW_DATA_ROOT = 4,
    OLD_NULL_ROOT = 5,
    NEW_NULL_ROOT = 6,
    OLD_DATA_ROOTS_ROOT = 7,
    NEW_DATA_ROOTS_ROOT = 8,
    TOTAL_TX_FEES = 9,
    INNER_PROOFS_DATA = TOTAL_TX_FEES + NUM_ASSETS,
};
} // namespace RollupProofFields

namespace RollupProofOffsets {
enum {
    ROLLUP_ID = RollupProofFields::ROLLUP_ID * 32,
    ROLLUP_SIZE = RollupProofFields::ROLLUP_SIZE * 32,
    DATA_START_INDEX = RollupProofFields::DATA_START_INDEX * 32,
    OLD_DATA_ROOT = RollupProofFields::OLD_DATA_ROOT * 32,
    NEW_DATA_ROOT = RollupProofFields::NEW_DATA_ROOT * 32,
    OLD_NULL_ROOT = RollupProofFields::OLD_NULL_ROOT * 32,
    NEW_NULL_ROOT = RollupProofFields::NEW_NULL_ROOT * 32,
    OLD_DATA_ROOTS_ROOT = RollupProofFields::OLD_DATA_ROOTS_ROOT * 32,
    NEW_DATA_ROOTS_ROOT = RollupProofFields::NEW_DATA_ROOTS_ROOT * 32,
    TOTAL_TX_FEES = RollupProofFields::TOTAL_TX_FEES * 32,
    INNER_PROOFS_DATA = RollupProofFields::INNER_PROOFS_DATA * 32,
};
} // namespace RollupProofOffsets

struct propagated_inner_proof_data {
    uint256_t proof_id;
    uint256_t public_input;
    uint256_t public_output;
    uint256_t asset_id;
    grumpkin::g1::affine_element new_note1;
    grumpkin::g1::affine_element new_note2;
    uint256_t nullifier1;
    uint256_t nullifier2;
    fr input_owner;
    fr output_owner;
};

struct rollup_proof_data {
    uint32_t rollup_id;
    uint32_t rollup_size;
    uint32_t data_start_index;
    fr old_data_root;
    fr new_data_root;
    fr old_null_root;
    fr new_null_root;
    fr old_data_roots_root;
    fr new_data_roots_root;
    std::vector<uint256_t> total_tx_fees;
    std::vector<propagated_inner_proof_data> inner_proofs;
    g1::affine_element recursion_output[2];

    fr new_defi_root;
    std::array<uint256_t, NUM_BRIDGE_CALLS_PER_BLOCK> bridge_ids;
    std::array<uint256_t, NUM_BRIDGE_CALLS_PER_BLOCK> deposit_sums;

    rollup_proof_data(std::vector<uint8_t> const& proof_data);
    rollup_proof_data(std::vector<fr> const& fields);

  private:
    void populate_from_fields(std::vector<fr> const& fields);
};

} // namespace rollup
} // namespace proofs
} // namespace rollup
