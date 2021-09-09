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
    ASSET_IDS = DEFI_BRIDGE_DEPOSITS + NUM_BRIDGE_CALLS_PER_BLOCK,
    TOTAL_TX_FEES = ASSET_IDS + NUM_ASSETS,
    INPUTS_HASH = TOTAL_TX_FEES + NUM_ASSETS,
    INNER_PROOFS_DATA,
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
    ASSET_IDS = RollupProofFields::ASSET_IDS * 32,
    TOTAL_TX_FEES = RollupProofFields::TOTAL_TX_FEES * 32,
    INNER_PROOFS_DATA = RollupProofFields::INNER_PROOFS_DATA * 32,
};
} // namespace RollupProofOffsets

namespace PropagatedInnerProofFields {
enum {
    PROOF_ID,
    PUBLIC_INPUT,
    PUBLIC_OUTPUT,
    ASSET_ID,
    NOTE_COMMITMENT1,
    NOTE_COMMITMENT2,
    NULLIFIER1,
    NULLIFIER2,
    INPUT_OWNER,
    OUTPUT_OWNER,
    NUM_FIELDS
};
}

struct propagated_inner_proof_data {
    uint256_t proof_id;
    uint256_t public_input;
    uint256_t public_output;
    uint256_t asset_id;
    grumpkin::fq note_commitment1;
    grumpkin::fq note_commitment2;
    uint256_t nullifier1;
    uint256_t nullifier2;
    fr input_owner;
    fr output_owner;

    bool operator==(const propagated_inner_proof_data& other) const = default;
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
    std::array<uint256_t, NUM_ASSETS> asset_ids;
    std::array<uint256_t, NUM_ASSETS> total_tx_fees;
    fr input_hash;
    std::vector<propagated_inner_proof_data> inner_proofs;
    g1::affine_element recursion_output[2];

    rollup_proof_data() {}
    rollup_proof_data(std::vector<field_ct> const& fields);
    rollup_proof_data(std::vector<uint8_t> const& proof_data);
    rollup_proof_data(std::vector<fr> const& fields);

    bool operator==(const rollup_proof_data& other) const = default;

  private:
    virtual void populate_from_fields(std::vector<fr> const& fields);
};

inline std::ostream& operator<<(std::ostream& os, rollup_proof_data const& data)
{
    // clang-format off
    return os << "{\n"
        << "  data_start_index: " << data.data_start_index << "\n"
        << "  old_data_root: " << data.old_data_root << "\n"
        << "  new_data_root: " << data.new_data_root << "\n"
        << "  old_null_root: " << data.old_null_root << "\n"
        << "  new_null_root: " << data.new_null_root << "\n"
        << "  old_data_roots_root: " << data.old_data_roots_root << "\n"
        << "  new_data_roots_root: " << data.new_data_roots_root << "\n"
        << "  old_defi_root: " << data.old_defi_root << "\n"
        << "  new_defi_root: " << data.new_defi_root << "\n"
        << "  bridge_ids: " << data.bridge_ids << "\n"
        << "  deposit_sums: " << data.deposit_sums << "\n"
        << "  asset_ids: " << data.asset_ids << "\n"
        << "  total_tx_fees: " << data.total_tx_fees << "\n"
        // << "  inner_proofs: " << data.inner_proofs << "\n"
        // << "  recursion_output: " << data.recursion_output << "\n"
        << "}";
    // clang-format on
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
