#include "../../constants.hpp"
#include "../rollup/rollup_circuit.hpp"
#include "./root_rollup_circuit.hpp"
#include "../inner_proof_data.hpp"
#include "../rollup/rollup_proof_data.hpp"
#include <stdlib/merkle_tree/membership.hpp>
#include <common/throw_or_abort.hpp>
#include <stdlib/hash/sha256/sha256.hpp>
#include "../notes/constants.hpp"
#include "../notes/circuit/pedersen_note.hpp"

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::RollupProofFields;

void add_padding_public_inputs(Composer& composer, size_t inner_size)
{
    for (size_t i = 0; i < InnerProofFields::NUM_PUBLISHED * inner_size; ++i) {
        auto zero = witness_ct(&composer, 0);
        composer.assert_equal_constant(zero.witness_index, 0);
        composer.set_public_input(zero.witness_index);
    }
}

void check_root_tree_updated(Composer& composer,
                             merkle_tree::hash_path const& new_data_roots_path,
                             merkle_tree::hash_path const& old_data_roots_path,
                             field_ct const& rollup_id,
                             field_ct const& new_data_root,
                             field_ct const& new_data_roots_root,
                             field_ct const& old_data_roots_root)
{
    auto empty_tree_value = byte_array_ct(&composer, 64);
    auto new_data_root_arr = byte_array_ct(new_data_root);
    auto one = field_ct(witness_ct(&composer, 1));
    auto index = byte_array_ct(rollup_id + one);
    update_membership(composer,
                      new_data_roots_root,
                      new_data_roots_path,
                      new_data_root_arr,
                      old_data_roots_root,
                      old_data_roots_path,
                      empty_tree_value,
                      index,
                      __FUNCTION__);
}

void check_data_tree_updated(Composer& composer,
                             size_t leaves,
                             merkle_tree::hash_path const& new_data_path,
                             merkle_tree::hash_path const& old_data_path,
                             std::vector<byte_array_ct> const& new_data_values,
                             field_ct const& old_data_root,
                             field_ct const& new_data_root,
                             field_ct const& data_start_index)
{
    size_t height = numeric::get_msb(leaves);
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

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& root_rollup,
                                            size_t inner_rollup_size,
                                            size_t outer_rollup_size,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{
    recursion_output<bn254> recursion_output;

    auto num_proofs = root_rollup.rollups.size();
    field_ct rollup_size = witness_ct(&composer, outer_rollup_size);
    composer.assert_equal_constant(rollup_size.witness_index, outer_rollup_size);

    std::vector<field_ct> inner_proof_public_inputs;
    uint32_ct num_inner_proofs = witness_ct(&composer, root_rollup.num_inner_proofs);
    field_ct rollup_id = witness_ct(&composer, root_rollup.rollup_id);
    field_ct data_start_index = witness_ct(&composer, 0);
    field_ct old_data_root = witness_ct(&composer, 0);
    field_ct new_data_root = witness_ct(&composer, 0);
    field_ct old_null_root = witness_ct(&composer, 0);
    field_ct new_null_root = witness_ct(&composer, 0);
    field_ct old_root_root = witness_ct(&composer, root_rollup.old_data_roots_root);
    field_ct new_root_root = witness_ct(&composer, root_rollup.new_data_roots_root);
    // Deprecated.
    field_ct num_txs = field_ct::from_witness_index(&composer, composer.zero_idx);

    auto total_tx_fees = std::vector<field_ct>(NUM_ASSETS, field_ct::from_witness_index(&composer, composer.zero_idx));
    auto recursive_manifest = Composer::create_unrolled_manifest(inner_verification_key->num_public_inputs);

    auto defi_deposit_sums =
        std::vector<field_ct>(NUM_BRIDGE_CALLS_PER_BLOCK, field_ct::from_witness_index(&composer, composer.zero_idx));

    for (size_t i = 0; i < num_proofs; ++i) {
        auto recursive_verification_key =
            plonk::stdlib::recursion::verification_key<bn254>::from_constants(&composer, inner_verification_key);
        recursion_output =
            verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(&composer,
                                                                          recursive_verification_key,
                                                                          recursive_manifest,
                                                                          waffle::plonk_proof{ root_rollup.rollups[i] },
                                                                          recursion_output);

        auto public_inputs = recursion_output.public_inputs;
        auto inner_index = uint32_ct(static_cast<uint32_t>(i));
        auto is_real = num_inner_proofs > inner_index;
        auto data_start_index_inner = public_inputs[DATA_START_INDEX];
        auto old_data_root_inner = public_inputs[OLD_DATA_ROOT];
        auto new_data_root_inner = public_inputs[NEW_DATA_ROOT];
        auto old_null_root_inner = public_inputs[OLD_NULL_ROOT];
        auto new_null_root_inner = public_inputs[NEW_NULL_ROOT];
        auto old_root_root_inner = public_inputs[OLD_DATA_ROOTS_ROOT];

        for (size_t j = 0; j < InnerProofFields::NUM_PUBLISHED * inner_rollup_size; ++j) {
            inner_proof_public_inputs.push_back(public_inputs[INNER_PROOFS_DATA + j] * is_real);
        }

        /**
         * Steps in processing defi deposit proofs:
         *   (i) We need to process only those inner join split proofs which have proof_id as the defi_deposit_proof_id.
         *  (ii) For the defi deposit proofs, ensure that the bridge_id matches one of NUM_BRIDGE_CALLS_PER_BLOCK
         *       bridge_ids provided as input by the rollup_provider.
         * (iii) Only if (ii) succeeds, accumulate the deposit value in the defi_deposit_sums. These would be added as
         *       public inputs in the root rollup proof.
         *  (iv) We also need to modify the claim note encryptions (output_note_1 encryption) to add the interaction
         *       nonce to it.
         */
        for (size_t j = 0; j < inner_rollup_size; j++) {
            const auto public_input_start_idx = INNER_PROOFS_DATA + (j * InnerProofFields::NUM_PUBLISHED);
            const auto proof_id = public_inputs[public_input_start_idx + InnerProofFields::PROOF_ID];
            const auto bridge_id = public_inputs[public_input_start_idx + InnerProofFields::ASSET_ID];
            const auto deposit_value = public_inputs[public_input_start_idx + InnerProofFields::PUBLIC_OUTPUT];
            const auto is_defi_deposit = proof_id == field_ct(DEFI_BRIDGE_DEPOSIT);
            auto is_valid_bridge_id = bool_ct(&composer, false);

            // Modify the claim note output to mix in the interaction nonce, as the client always leaves it as 0.
            point_ct encrypted_claim_note{ public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_X],
                                           public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_Y] };
            encrypted_claim_note = notes::circuit::conditionally_hash_and_accumulate<32>(
                encrypted_claim_note,
                (field_ct(root_rollup.interaction_nonce) * is_defi_deposit),
                notes::GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_DEFI_INTERACTION_NONCE);
            recursion_output.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_X] =
                encrypted_claim_note.x;
            recursion_output.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_Y] =
                encrypted_claim_note.y;

            // Accumulate the sum of defi deposits for each bridge id.
            for (size_t k = 0; k < NUM_BRIDGE_CALLS_PER_BLOCK; k++) {
                const auto matches = bridge_id == root_rollup.bridge_ids[k];
                defi_deposit_sums[k] += deposit_value * is_defi_deposit * matches;
                is_valid_bridge_id = is_valid_bridge_id || matches;
            }

            is_valid_bridge_id = (is_valid_bridge_id || !is_defi_deposit).normalize();
            composer.assert_equal_constant(is_valid_bridge_id.witness_index, 1, "bridge_id_not_in_working_set");
        }

        for (size_t j = 0; j < NUM_ASSETS; ++j) {
            total_tx_fees[j] += public_inputs[TOTAL_TX_FEES + j];
        }

        // Every real inner proof should use the root tree root we've input.
        auto valid_root_root = (!is_real || old_root_root_inner == old_root_root).normalize();
        composer.assert_equal_constant(valid_root_root.witness_index, 1, format("inconsistent_root_roots_", i));

        if (i == 0) {
            // The first proof should always be real.
            composer.assert_equal_constant(is_real.witness_index, 1);
            data_start_index = data_start_index_inner;
            old_data_root = old_data_root_inner;
            new_data_root = new_data_root_inner;
            old_null_root = old_null_root_inner;
            new_null_root = new_null_root_inner;
        } else {
            auto valid_data_start_index =
                !is_real || data_start_index_inner == (data_start_index + (i * inner_rollup_size * 2));
            auto valid_old_data_root = !is_real || old_data_root_inner == new_data_root;
            auto valid_old_null_root = !is_real || old_null_root_inner == new_null_root;

            composer.assert_equal_constant(
                valid_data_start_index.normalize().witness_index, 1, format("incorrect_data_start_index_", i));
            composer.assert_equal_constant(
                valid_old_data_root.normalize().witness_index, 1, format("inconsistent_data_roots_", i));
            composer.assert_equal_constant(
                valid_old_null_root.normalize().witness_index, 1, format("inconsistent_null_roots_", i));

            new_data_root = (new_data_root_inner * is_real) + (new_data_root * !is_real);
            new_null_root = (new_null_root_inner * is_real) + (new_null_root * !is_real);
        }
    }

    /**
     * We do the following to ensure defi notes are added into the data tree. Note that we need to pad the
     * defi_interaction_notes with '0' notes iff the size of input defi_interaction_notes is less than
     * NUM_BRIDGE_CALLS_PER_BLOCK.
     *
     *  (i) Compute the previous_defi_intreaction_hash using the input defi_interaction_notes.
     * (ii) Update the data tree with encryptions of the defi_interaction_notes.
     */
    byte_array_ct hash_input(&composer);
    auto is_defi_interaction_note_present = bool_ct(&composer, false);
    auto num_interaction_notes = witness_ct(&composer, root_rollup.defi_interaction_notes.size());
    auto defi_interaction_note_leaves = std::vector<byte_array_ct>();
    auto encrypted_defi_interaction_notes = std::vector<point_ct>();
    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto is_real = uint32_ct(i) < num_interaction_notes;
        hash_input.write(root_rollup.defi_interaction_notes[i].to_byte_array(composer, is_real));
        const point_ct encrypted_note = { root_rollup.defi_interaction_notes[i].encrypted.x * is_real,
                                          root_rollup.defi_interaction_notes[i].encrypted.y * is_real };
        encrypted_defi_interaction_notes.push_back(encrypted_note);
        defi_interaction_note_leaves.push_back(
            byte_array_ct(&composer).write(encrypted_note.x).write(encrypted_note.y));
        is_defi_interaction_note_present = (is_defi_interaction_note_present || is_real);
    }

    const auto hash_output = plonk::stdlib::sha256<Composer>(hash_input);
    const auto previous_defi_interaction_hash = field_ct(byte_array_ct(hash_output));

    field_ct defi_new_data_root = witness_ct(&composer, root_rollup.new_data_root);
    const auto updated_new_data_root =
        (defi_new_data_root * is_defi_interaction_note_present) + (new_data_root * !is_defi_interaction_note_present);

    // Check defi interaction notes have been inserted into the data tree.
    auto new_data_path = create_witness_hash_path(composer, root_rollup.new_data_path);
    auto old_data_path = create_witness_hash_path(composer, root_rollup.old_data_path);
    check_data_tree_updated(composer,
                            NUM_BRIDGE_CALLS_PER_BLOCK,
                            new_data_path,
                            old_data_path,
                            defi_interaction_note_leaves,
                            old_data_root,
                            root_rollup.new_data_root,
                            (rollup_size * 2) - NUM_BRIDGE_CALLS_PER_BLOCK);

    // Check data root tree is updated with latest data root.
    auto new_data_roots_path = create_witness_hash_path(composer, root_rollup.new_data_roots_path);
    auto old_data_roots_path = create_witness_hash_path(composer, root_rollup.old_data_roots_path);
    check_root_tree_updated(composer,
                            new_data_roots_path,
                            old_data_roots_path,
                            rollup_id,
                            updated_new_data_root,
                            new_root_root,
                            old_root_root);

    composer.set_public_input(rollup_id.witness_index);
    composer.set_public_input(rollup_size.witness_index);
    composer.set_public_input(data_start_index.witness_index);
    composer.set_public_input(old_data_root.witness_index);
    composer.set_public_input(updated_new_data_root.witness_index);
    composer.set_public_input(old_null_root.witness_index);
    composer.set_public_input(new_null_root.witness_index);
    composer.set_public_input(old_root_root.witness_index);
    composer.set_public_input(new_root_root.witness_index);
    composer.set_public_input(previous_defi_interaction_hash.witness_index);

    /**
     * For the defi deposits, we add the following as the public input of the root_rollup circuit:
     *   (i) encrypted defi_interaction_notes
     *  (ii) defi_deposit_sums
     * (iii) previous_defi_interaction_hash
     */
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        composer.set_public_input(root_rollup.bridge_ids[i].witness_index);
        composer.set_public_input(defi_deposit_sums[i].witness_index);
        encrypted_defi_interaction_notes[i].set_public();
    }

    for (auto total_tx_fee : total_tx_fees) {
        composer.set_public_input(total_tx_fee.witness_index);
    }
    composer.set_public_input(num_txs.get_witness_index());

    for (auto& inp : inner_proof_public_inputs) {
        composer.set_public_input(inp.witness_index);
    }

    for (size_t i = num_proofs; i < outer_rollup_size / inner_rollup_size; ++i) {
        add_padding_public_inputs(composer, inner_rollup_size);
    }

    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
