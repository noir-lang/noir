#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

using namespace plonk::stdlib::types::turbo;

namespace RollupProofFields {
enum {
    ROLLUP_ID,
    ROLLUP_SIZE,
    DATA_START_INDEX,
    OLD_DATA_ROOT,
    NEW_DATA_ROOT,
    OLD_NULL_ROOT,
    NEW_NULL_ROOT,
    OLD_DATA_ROOTS_ROOT,
    NEW_DATA_ROOTS_ROOT,
    OLD_DEFI_ROOT,
    NEW_DEFI_ROOT,
    DEFI_BRIDGE_IDS,
    DEFI_BRIDGE_DEPOSITS = DEFI_BRIDGE_IDS + NUM_BRIDGE_CALLS_PER_BLOCK,
    TOTAL_TX_FEES = DEFI_BRIDGE_DEPOSITS + NUM_BRIDGE_CALLS_PER_BLOCK,
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
    OLD_DEFI_ROOT = RollupProofFields::OLD_DEFI_ROOT * 32,
    NEW_DEFI_ROOT = RollupProofFields::NEW_DEFI_ROOT * 32,
    DEFI_BRIDGE_IDS = RollupProofFields::DEFI_BRIDGE_IDS * 32,
    DEFI_BRIDGE_DEPOSITS = RollupProofFields::DEFI_BRIDGE_DEPOSITS * 32,
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
    fr old_defi_root;
    fr new_defi_root;
    std::array<uint256_t, NUM_BRIDGE_CALLS_PER_BLOCK> bridge_ids;
    std::array<uint256_t, NUM_BRIDGE_CALLS_PER_BLOCK> deposit_sums;
    std::array<uint256_t, NUM_ASSETS> total_tx_fees;
    std::vector<propagated_inner_proof_data> inner_proofs;
    g1::affine_element recursion_output[2];

    rollup_proof_data(std::vector<uint8_t> const& proof_data);
    rollup_proof_data(std::vector<fr> const& fields);

  private:
    void populate_from_fields(std::vector<fr> const& fields);
};

} // namespace rollup
} // namespace proofs
} // namespace rollup
