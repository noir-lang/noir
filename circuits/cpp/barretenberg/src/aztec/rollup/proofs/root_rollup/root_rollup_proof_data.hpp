#pragma once
#include <stdlib/types/turbo.hpp>
#include "../rollup/rollup_proof_data.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

namespace RootRollupProofFields {
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
    DEFI_INTERACTION_NOTES = TOTAL_TX_FEES + NUM_ASSETS,
    PREVIOUS_DEFI_INTERACTION_HASH = DEFI_INTERACTION_NOTES + NUM_BRIDGE_CALLS_PER_BLOCK,
    NUM_ROLLUP_TXS,
    INNER_PROOFS_DATA,
};
} // namespace RootRollupProofFields

using namespace plonk::stdlib::types::turbo;

struct root_rollup_proof_data : rollup::rollup_proof_data {
    std::array<grumpkin::fq, NUM_BRIDGE_CALLS_PER_BLOCK> defi_interaction_notes;
    uint256_t previous_defi_interaction_hash;
    uint32_t num_inner_txs; // number of inner rollup proofs
    root_rollup_proof_data(std::vector<uint8_t> const& proof_data);
    root_rollup_proof_data(std::vector<fr> const& public_inputs);
    root_rollup_proof_data(std::vector<field_ct> const& public_inputs);
    root_rollup_proof_data() {}

    std::vector<uint8_t> encode_proof_data() const;
    std::vector<uint8_t> compute_hash_from_encoded_inputs() const;

    bool operator==(const root_rollup_proof_data& other) const
    {
        auto lhs = encode_proof_data();
        auto rhs = other.encode_proof_data();
        return (lhs == rhs);
    }

  private:
    void populate_from_fields(std::vector<fr> const& fields) override;
};

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
