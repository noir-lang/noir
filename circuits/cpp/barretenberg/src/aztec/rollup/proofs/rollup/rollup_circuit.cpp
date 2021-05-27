#include "rollup_circuit.hpp"
#include "../../constants.hpp"
#include "../inner_proof_data.hpp"
#include "../notes/circuit/claim/index.hpp"
#include <stdlib/merkle_tree/index.hpp>
#include <common/map.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace rollup {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;

void propagate_inner_proof_public_inputs(Composer& composer, std::vector<field_ct> const& public_inputs)
{
    for (size_t i = 0; i < InnerProofFields::NUM_PUBLISHED; ++i) {
        composer.set_public_input(public_inputs[i].witness_index);
    }
}

void add_padding_public_inputs(Composer& composer)
{
    for (size_t i = 0; i < InnerProofFields::NUM_PUBLISHED; ++i) {
        auto zero = witness_ct(&composer, 0);
        composer.assert_equal_constant(zero.witness_index, 0);
        composer.set_public_input(zero.witness_index);
    }
}

field_ct check_nullifiers_inserted(Composer& composer,
                                   std::vector<fr> const& new_null_roots,
                                   std::vector<fr_hash_path> const& old_null_paths,
                                   std::vector<fr_hash_path> const& new_null_paths,
                                   uint32_ct const& num_txs,
                                   field_ct latest_null_root,
                                   std::vector<field_ct> const& new_null_indicies)
{
    for (size_t i = 0; i < new_null_indicies.size(); ++i) {
        auto new_null_root = field_ct(witness_ct(&composer, new_null_roots[i]));

        auto is_real = num_txs > uint32_ct(&composer, i / 2) && new_null_indicies[i] != 0;

        // This makes padding transactions act as noops.
        auto index = (new_null_indicies[i] * is_real);
        auto old_nullifier_value = byte_array_ct(&composer, 64);
        auto new_nullifier_value = byte_array_ct(&composer, 64);
        new_nullifier_value.set_bit(0, is_real);
        auto new_null_path = create_witness_hash_path(composer, new_null_paths[i]);
        auto old_null_path = create_witness_hash_path(composer, old_null_paths[i]);

        update_membership(composer,
                          new_null_root,
                          new_null_path,
                          new_nullifier_value,
                          latest_null_root,
                          old_null_path,
                          old_nullifier_value,
                          byte_array_ct(index),
                          format(__FUNCTION__, "_", i));

        latest_null_root = new_null_root;
    }

    return latest_null_root;
}

void check_data_tree_updated(Composer& composer,
                             size_t rollup_size,
                             merkle_tree::hash_path const& new_data_path,
                             merkle_tree::hash_path const& old_data_path,
                             std::vector<byte_array_ct> const& new_data_values,
                             field_ct const& old_data_root,
                             field_ct const& new_data_root,
                             field_ct const& data_start_index)
{
    size_t height = numeric::get_msb(rollup_size) + 1;
    auto zero_subtree_root = field_ct(zero_hash_at_height(height));

    auto rollup_root = compute_tree_root(new_data_values);

    update_subtree_membership(composer,
                              new_data_root,
                              new_data_path,
                              rollup_root,
                              old_data_root,
                              old_data_path,
                              zero_subtree_root,
                              byte_array_ct(data_start_index),
                              height,
                              __FUNCTION__);
}

/**
 * Processes a defi deposit proof.
 * - We only process join split proofs with a proof_id == ProofIds::DEFI_DEPOSIT (otherwise noop).
 * - Ensure that the bridge_id matches one within the of set of bridge_ids.
 * - Accumulate the deposit value in relevant defi_deposit_sums slot. These later become public inputs.
 * - Modify the claim note encryption (output_note_1 encryption) to add the relevant interaction nonce to it.
 */
auto process_defi_deposit(Composer& composer,
                          field_ct const& rollup_id,
                          std::vector<field_ct>& public_inputs,
                          std::vector<field_ct> const& bridge_ids,
                          std::vector<field_ct>& defi_deposit_sums,
                          field_ct const& num_defi_interactions)
{
    field_ct defi_interaction_nonce = (rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK).normalize();

    const auto proof_id = public_inputs[InnerProofFields::PROOF_ID];
    const auto bridge_id = public_inputs[InnerProofFields::ASSET_ID];
    const auto deposit_value = public_inputs[InnerProofFields::PUBLIC_OUTPUT];
    const auto is_defi_deposit = proof_id == field_ct(ProofIds::DEFI_DEPOSIT);

    field_ct note_defi_interaction_nonce = defi_interaction_nonce;
    field_ct num_matched(&composer, 0);

    for (uint32_t k = 0; k < NUM_BRIDGE_CALLS_PER_BLOCK; k++) {
        auto is_real = uint32_ct(k) < num_defi_interactions;

        const auto matches = bridge_id == bridge_ids[k] && is_real;
        num_matched += matches;

        defi_deposit_sums[k] += deposit_value * is_defi_deposit * matches;
        note_defi_interaction_nonce += (field_ct(&composer, k) * matches);
    }
    note_defi_interaction_nonce *= is_defi_deposit;

    // Assert this proof matched a single bridge_id.
    auto is_valid_bridge_id = (num_matched == 1 || !is_defi_deposit).normalize();
    composer.assert_equal_constant(is_valid_bridge_id.witness_index,
                                   1,
                                   format("proof bridge id matched ", uint64_t(num_matched.get_value()), " times."));

    // Modify the claim note output to mix in the interaction nonce, as the client always leaves it as 0.
    point_ct encrypted_claim_note{ public_inputs[InnerProofFields::NEW_NOTE1_X],
                                   public_inputs[InnerProofFields::NEW_NOTE1_Y] };
    encrypted_claim_note =
        notes::circuit::claim::complete_partial_claim_note(encrypted_claim_note, note_defi_interaction_nonce);

    public_inputs[InnerProofFields::NEW_NOTE1_X] = encrypted_claim_note.x;
    public_inputs[InnerProofFields::NEW_NOTE1_Y] = encrypted_claim_note.y;
}

/**
 * Check that claim proofs are using the correct defi root.
 */
auto process_claims(Composer& composer, std::vector<field_ct>& public_inputs, field_ct const& new_defi_root)
{
    const auto is_claim = public_inputs[InnerProofFields::PROOF_ID] == field_ct(ProofIds::DEFI_CLAIM);
    // For claim proofs, defi root is output in field named PUBLIC_OWNER.
    const auto defi_root = public_inputs[InnerProofFields::INPUT_OWNER];

    auto valid = defi_root == new_defi_root || !is_claim;
    composer.assert_equal_constant(valid.witness_index, 1, format("claim proof has unmatched defi root"));
}

recursion_output<bn254> rollup_circuit(Composer& composer,
                                       rollup_tx const& rollup,
                                       std::vector<std::shared_ptr<waffle::verification_key>> const& verification_keys,
                                       size_t max_num_txs)
{
    // Witnesses.
    auto floor_rollup_size = 1UL << numeric::get_msb(max_num_txs);
    auto rollup_size_pow2_ = floor_rollup_size << (max_num_txs != floor_rollup_size);
    auto rollup_size_pow2 = field_ct(witness_ct(&composer, rollup_size_pow2_));
    composer.assert_equal_constant(rollup_size_pow2.witness_index, rollup_size_pow2_);
    auto rollup_id = field_ct(witness_ct(&composer, rollup.rollup_id));
    auto num_txs = uint32_ct(witness_ct(&composer, rollup.num_txs));
    composer.create_range_constraint(num_txs.get_witness_index(), MAX_TXS_BIT_LENGTH);
    auto data_start_index = field_ct(witness_ct(&composer, rollup.data_start_index));
    auto old_data_root = field_ct(witness_ct(&composer, rollup.old_data_root));
    auto new_data_root = field_ct(witness_ct(&composer, rollup.new_data_root));
    auto old_null_root = field_ct(witness_ct(&composer, rollup.old_null_root));
    auto data_roots_root = field_ct(witness_ct(&composer, rollup.data_roots_root));
    auto recursive_manifest = Composer::create_unrolled_manifest(verification_keys[0]->num_public_inputs);
    auto num_defi_interactions = field_ct(witness_ct(&composer, rollup.num_defi_interactions));
    auto new_defi_root = field_ct(witness_ct(&composer, rollup.new_defi_root));
    auto bridge_ids = map(rollup.bridge_ids, [&](auto bid) { return field_ct(witness_ct(&composer, bid)); });

    // Zero any input bridge_ids that are outside scope, and check in scope bridge_ids are not zero.
    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto in_scope = uint32_ct(i) < num_defi_interactions;
        bridge_ids[i] *= in_scope;
        auto valid = !in_scope || bridge_ids[i] != 0;
        composer.assert_equal_constant(valid.witness_index, 1, "bridge_id out of scope");
    }

    // Loop accumulators.
    auto new_data_values = std::vector<byte_array_ct>();
    auto new_null_indicies = std::vector<field_ct>();
    recursion_output<bn254> recursion_output;
    std::vector<std::vector<field_ct>> inner_public_inputs;
    auto total_tx_fees = std::vector<field_ct>(NUM_ASSETS, field_ct::from_witness_index(&composer, composer.zero_idx));
    std::vector<field_ct> defi_deposit_sums(NUM_BRIDGE_CALLS_PER_BLOCK,
                                            field_ct::from_witness_index(&composer, composer.zero_idx));

    for (size_t i = 0; i < max_num_txs; ++i) {
        // Pick verification key and check it's permitted.
        auto proof_id = from_buffer<uint32_t>(rollup.txs[i], InnerProofOffsets::PROOF_ID + 28);
        auto recursive_verification_key =
            plonk::stdlib::recursion::verification_key<bn254>::from_witness(&composer, verification_keys[proof_id]);
        recursive_verification_key->validate_key_is_in_set(verification_keys);

        // Verify the inner proof.
        recursion_output =
            verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(&composer,
                                                                          recursive_verification_key,
                                                                          recursive_manifest,
                                                                          waffle::plonk_proof{ rollup.txs[i] },
                                                                          recursion_output);

        auto is_real = num_txs > uint32_ct(&composer, i);
        auto& public_inputs = recursion_output.public_inputs;

        // Zero padding public inputs.
        for (size_t i = 0; i < InnerProofFields::NUM_PUBLISHED; ++i) {
            public_inputs[i] *= is_real;
        }

        process_defi_deposit(composer, rollup_id, public_inputs, bridge_ids, defi_deposit_sums, num_defi_interactions);
        process_claims(composer, public_inputs, new_defi_root);

        // Add the proofs data values to the list.
        new_data_values.push_back(byte_array_ct(&composer)
                                      .write(public_inputs[InnerProofFields::NEW_NOTE1_X])
                                      .write(public_inputs[InnerProofFields::NEW_NOTE1_Y]));
        new_data_values.push_back(byte_array_ct(&composer)
                                      .write(public_inputs[InnerProofFields::NEW_NOTE2_X])
                                      .write(public_inputs[InnerProofFields::NEW_NOTE2_Y]));

        // Add nullifiers to the list.
        new_null_indicies.push_back(public_inputs[InnerProofFields::NULLIFIER1]);
        new_null_indicies.push_back(public_inputs[InnerProofFields::NULLIFIER2]);

        // Check this proofs data root exists in the data root tree (unless a padding entry).
        auto data_root = public_inputs[InnerProofFields::MERKLE_ROOT];
        auto data_roots_path = create_witness_hash_path(composer, rollup.data_roots_paths[i]);
        auto data_root_index = uint32_ct(witness_ct(&composer, rollup.data_roots_indicies[i]));
        bool_ct valid =
            data_root != 0 &&
            check_membership(
                composer, data_roots_root, data_roots_path, byte_array_ct(data_root), byte_array_ct(data_root_index));
        composer.assert_equal(is_real.witness_index, valid.witness_index, format("data_root_for_proof_", i));

        // Accumulate tx fee.
        auto asset_id = public_inputs[InnerProofFields::ASSET_ID];
        auto tx_fee = public_inputs[InnerProofFields::TX_FEE];
        for (size_t j = 0; j < NUM_ASSETS; ++j) {
            total_tx_fees[j] += tx_fee * is_real * (asset_id == j);
        }

        inner_public_inputs.push_back(public_inputs);
    }

    auto new_data_path = create_witness_hash_path(composer, rollup.new_data_path);
    auto old_data_path = create_witness_hash_path(composer, rollup.old_data_path);
    new_data_values.resize(rollup_size_pow2_ * 2, byte_array_ct(&composer, 64));
    check_data_tree_updated(composer,
                            rollup_size_pow2_,
                            new_data_path,
                            old_data_path,
                            new_data_values,
                            old_data_root,
                            new_data_root,
                            data_start_index);

    auto new_null_root = check_nullifiers_inserted(composer,
                                                   rollup.new_null_roots,
                                                   rollup.old_null_paths,
                                                   rollup.new_null_paths,
                                                   num_txs,
                                                   old_null_root,
                                                   new_null_indicies);

    // Publish public inputs.
    composer.set_public_input(rollup_id.witness_index);
    composer.set_public_input(rollup_size_pow2.witness_index);
    composer.set_public_input(data_start_index.witness_index);
    composer.set_public_input(old_data_root.witness_index);
    composer.set_public_input(new_data_root.witness_index);
    composer.set_public_input(old_null_root.witness_index);
    composer.set_public_input(new_null_root.witness_index);
    composer.set_public_input(data_roots_root.witness_index);
    composer.set_public_input(witness_ct(&composer, rollup.data_roots_root).witness_index);
    for (auto total_tx_fee : total_tx_fees) {
        composer.set_public_input(total_tx_fee.witness_index);
    }
    for (auto& inner : inner_public_inputs) {
        propagate_inner_proof_public_inputs(composer, inner);
    }

    for (size_t i = max_num_txs; i < rollup_size_pow2_; ++i) {
        add_padding_public_inputs(composer);
    }

    // Publish pairing coords limbs as public inputs.
    recursion_output.add_proof_outputs_as_public_inputs();

    // Defi public inputs.
    composer.set_public_input(new_defi_root.witness_index);
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        composer.set_public_input(bridge_ids[i].witness_index);
        composer.set_public_input(defi_deposit_sums[i].witness_index);
    }

    return recursion_output;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
