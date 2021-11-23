#pragma once
#include "../rollup/rollup_proof_data.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

namespace RootRollupBroadcastFields {
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
    ROLLUP_BENEFICIARY,
    NUM_INNER_PROOFS,
    INNER_PROOFS_DATA,
};
} // namespace RootRollupBroadcastFields

struct tx_broadcast_data {
    fr proof_id;
    fr note_commitment1;
    fr note_commitment2;
    fr nullifier1;
    fr nullifier2;
    fr public_value;
    fr public_owner;
    fr asset_id;
};

template <typename B> inline void read(B& buf, tx_broadcast_data& data)
{
    read(buf, data.proof_id);
    read(buf, data.note_commitment1);
    read(buf, data.note_commitment2);
    read(buf, data.nullifier1);
    read(buf, data.nullifier2);
    read(buf, data.public_value);
    read(buf, data.public_owner);
    read(buf, data.asset_id);
}

template <typename B> inline void write(B& buf, tx_broadcast_data const& data)
{
    write(buf, data.proof_id);
    write(buf, data.note_commitment1);
    write(buf, data.note_commitment2);
    write(buf, data.nullifier1);
    write(buf, data.nullifier2);
    write(buf, data.public_value);
    write(buf, data.public_owner);
    write(buf, data.asset_id);
}

struct root_rollup_broadcast_data {
    fr rollup_id;
    fr rollup_size;
    fr data_start_index;
    fr old_data_root;
    fr new_data_root;
    fr old_null_root;
    fr new_null_root;
    fr old_data_roots_root;
    fr new_data_roots_root;
    fr old_defi_root;
    fr new_defi_root;
    fr rollup_beneficiary;
    std::array<fr, NUM_BRIDGE_CALLS_PER_BLOCK> bridge_ids;
    std::array<fr, NUM_BRIDGE_CALLS_PER_BLOCK> deposit_sums;
    std::array<fr, NUM_ASSETS> asset_ids;
    std::array<fr, NUM_ASSETS> total_tx_fees;
    std::array<fr, NUM_BRIDGE_CALLS_PER_BLOCK> defi_interaction_notes;
    fr previous_defi_interaction_hash;
    fr num_inner_proofs;
    std::vector<tx_broadcast_data> tx_data;

    root_rollup_broadcast_data(std::vector<fr> const& public_inputs);

    fr compute_hash() const;

    bool operator==(const root_rollup_broadcast_data& other) const = default;
};

template <typename B> inline void read(B& buf, root_rollup_broadcast_data& data)
{
    using serialize::read;
    read(buf, data.rollup_id);
    read(buf, data.rollup_size);
    read(buf, data.data_start_index);

    read(buf, data.old_data_root);
    read(buf, data.new_data_root);
    read(buf, data.old_null_root);
    read(buf, data.new_null_root);
    read(buf, data.old_data_roots_root);
    read(buf, data.new_data_roots_root);
    read(buf, data.old_defi_root);
    read(buf, data.new_defi_root);

    read(buf, data.bridge_ids);
    read(buf, data.deposit_sums);
    read(buf, data.asset_ids);
    read(buf, data.total_tx_fees);
    read(buf, data.defi_interaction_notes);
    read(buf, data.previous_defi_interaction_hash);
    read(buf, data.rollup_beneficiary);
    read(buf, data.num_inner_proofs);

    for (auto& tx : data.tx_data) {
        read(buf, tx);
    }
}

template <typename B> inline void write(B& buf, root_rollup_broadcast_data const& data)
{
    using serialize::write;
    write(buf, data.rollup_id);
    write(buf, data.rollup_size);
    write(buf, data.data_start_index);

    write(buf, data.old_data_root);
    write(buf, data.new_data_root);
    write(buf, data.old_null_root);
    write(buf, data.new_null_root);
    write(buf, data.old_data_roots_root);
    write(buf, data.new_data_roots_root);
    write(buf, data.old_defi_root);
    write(buf, data.new_defi_root);

    write(buf, data.bridge_ids);
    write(buf, data.deposit_sums);
    write(buf, data.asset_ids);
    write(buf, data.total_tx_fees);
    write(buf, data.defi_interaction_notes);
    write(buf, data.previous_defi_interaction_hash);
    write(buf, data.rollup_beneficiary);
    write(buf, data.num_inner_proofs);

    for (auto& tx : data.tx_data) {
        write(buf, tx);
    }
}
} // namespace root_rollup
} // namespace proofs
} // namespace rollup
