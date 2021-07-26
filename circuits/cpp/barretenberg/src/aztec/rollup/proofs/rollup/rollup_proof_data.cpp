#include "rollup_proof_data.hpp"
#include "../../constants.hpp"
#include "../inner_proof_data.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

rollup_proof_data::rollup_proof_data(std::vector<uint8_t> const& proof_data)
{
    using serialize::read;
    auto ptr = proof_data.data();
    ptr += 60;
    read(ptr, rollup_size);

    auto num_fields =
        RollupProofFields::INNER_PROOFS_DATA + (rollup_size * PropagatedInnerProofFields::NUM_FIELDS) + 16;
    std::vector<fr> fields(num_fields);

    ptr = proof_data.data();
    for (size_t i = 0; i < num_fields; ++i) {
        read(ptr, fields[i]);
    }

    populate_from_fields(fields);
}

rollup_proof_data::rollup_proof_data(std::vector<fr> const& fields)
{
    populate_from_fields(fields);
}

void rollup_proof_data::populate_from_fields(std::vector<fr> const& fields)
{
    rollup_id = static_cast<uint32_t>(fields[RollupProofFields::ROLLUP_ID]);
    rollup_size = static_cast<uint32_t>(fields[RollupProofFields::ROLLUP_SIZE]);
    data_start_index = static_cast<uint32_t>(fields[RollupProofFields::DATA_START_INDEX]);
    old_data_root = fields[RollupProofFields::OLD_DATA_ROOT];
    new_data_root = fields[RollupProofFields::NEW_DATA_ROOT];
    old_null_root = fields[RollupProofFields::OLD_NULL_ROOT];
    new_null_root = fields[RollupProofFields::NEW_NULL_ROOT];
    old_data_roots_root = fields[RollupProofFields::OLD_DATA_ROOTS_ROOT];
    new_data_roots_root = fields[RollupProofFields::NEW_DATA_ROOTS_ROOT];
    old_defi_root = fields[RollupProofFields::OLD_DEFI_ROOT];
    new_defi_root = fields[RollupProofFields::NEW_DEFI_ROOT];
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        bridge_ids[i] = fields[RollupProofFields::DEFI_BRIDGE_IDS + i];
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        deposit_sums[i] = fields[RollupProofFields::DEFI_BRIDGE_DEPOSITS + i];
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        asset_ids[i] = fields[RollupProofFields::ASSET_IDS + i];
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        total_tx_fees[i] = fields[RollupProofFields::TOTAL_TX_FEES + i];
    }

    inner_proofs.resize(rollup_size);
    for (size_t i = 0; i < rollup_size; ++i) {
        auto offset = RollupProofFields::INNER_PROOFS_DATA + (i * PropagatedInnerProofFields::NUM_FIELDS);
        inner_proofs[i].proof_id = fields[offset + PropagatedInnerProofFields::PROOF_ID];
        inner_proofs[i].public_input = fields[offset + PropagatedInnerProofFields::PUBLIC_INPUT];
        inner_proofs[i].public_output = fields[offset + PropagatedInnerProofFields::PUBLIC_OUTPUT];
        inner_proofs[i].asset_id = fields[offset + PropagatedInnerProofFields::ASSET_ID];
        inner_proofs[i].note_commitment1 = fields[offset + PropagatedInnerProofFields::NOTE_COMMITMENT1];
        inner_proofs[i].note_commitment2 = fields[offset + PropagatedInnerProofFields::NOTE_COMMITMENT2];
        inner_proofs[i].nullifier1 = fields[offset + PropagatedInnerProofFields::NULLIFIER1];
        inner_proofs[i].nullifier2 = fields[offset + PropagatedInnerProofFields::NULLIFIER2];
        inner_proofs[i].input_owner = fields[offset + PropagatedInnerProofFields::INPUT_OWNER];
        inner_proofs[i].output_owner = fields[offset + PropagatedInnerProofFields::OUTPUT_OWNER];
    }

    auto offset = RollupProofFields::INNER_PROOFS_DATA + (rollup_size * PropagatedInnerProofFields::NUM_FIELDS);
    for (auto& coord :
         { &recursion_output[0].x, &recursion_output[0].y, &recursion_output[1].x, &recursion_output[1].y }) {
        uint256_t limb[4];
        for (size_t li = 0; li < 4; ++li) {
            limb[li] = fields[offset++];
        }
        *coord = limb[0] + (uint256_t(1) << 68) * limb[1] + (uint256_t(1) << 136) * limb[2] +
                 (uint256_t(1) << 204) * limb[3];
    }
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
