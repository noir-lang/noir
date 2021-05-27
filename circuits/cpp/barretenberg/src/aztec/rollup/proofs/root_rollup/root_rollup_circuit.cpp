#include "../../constants.hpp"
#include "../rollup/rollup_circuit.hpp"
#include "./root_rollup_circuit.hpp"
#include "../inner_proof_data.hpp"
#include "../rollup/rollup_proof_data.hpp"
#include <stdlib/merkle_tree/membership.hpp>
#include <stdlib/hash/sha256/sha256.hpp>
#include <common/map.hpp>
#include "../notes/constants.hpp"
#include "../notes/circuit/pedersen_note.hpp"
#include "../notes/circuit/defi_interaction/defi_interaction_note.hpp"
#include "../notes/circuit/claim/complete_partial_claim_note.hpp"

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace rollup;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;
using notes::circuit::defi_interaction::defi_interaction_note;

void add_padding_public_inputs(Composer& composer, size_t inner_size)
{
    for (size_t i = 0; i < InnerProofFields::NUM_PUBLISHED * inner_size; ++i) {
        auto zero = witness_ct(&composer, 0);
        composer.assert_equal_constant(zero.witness_index, 0);
        composer.set_public_input(zero.witness_index);
    }
}

/**
 * Inserts the latest data root into the root tree at location rollup_id + 1.
 */
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
    auto index = byte_array_ct(rollup_id + 1);
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

/**
 * Processes defi deposit proofs within an inner proofs.
 * - We only process join split proofs with a proof_id == defi_deposit_proof_id (otherwise noop).
 * - For the defi deposit proofs, ensure that the bridge_id matches one within the of set of bridge_ids.
 * - Accumulate the deposit value in defi_deposit_sums. These will become public inputs.
 * - Modify the claim note encryptions (output_note_1 encryption) to add the relevant interaction nonce to it.
 */
auto process_defi_deposits(Composer& composer,
                           field_ct const& rollup_id,
                           size_t num_inner_txs_pow2,
                           std::vector<field_ct>& public_inputs,
                           std::vector<field_ct> const& bridge_ids,
                           std::vector<field_ct>& defi_deposit_sums,
                           field_ct const& num_defi_interactions)
{
    field_ct defi_interaction_nonce = (rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK).normalize();

    for (size_t j = 0; j < num_inner_txs_pow2; j++) {
        const auto public_input_start_idx =
            RollupProofFields::INNER_PROOFS_DATA + (j * InnerProofFields::NUM_PUBLISHED);
        const auto proof_id = public_inputs[public_input_start_idx + InnerProofFields::PROOF_ID];
        const auto bridge_id = public_inputs[public_input_start_idx + InnerProofFields::ASSET_ID];
        const auto deposit_value = public_inputs[public_input_start_idx + InnerProofFields::PUBLIC_OUTPUT];
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
        composer.assert_equal_constant(
            is_valid_bridge_id.witness_index, 1, format("proof bridge id matched ", uint64_t(num_matched.get_value())));

        // Modify the claim note output to mix in the interaction nonce, as the client always leaves it as 0.
        point_ct encrypted_claim_note{ public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_X],
                                       public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_Y] };
        encrypted_claim_note =
            notes::circuit::claim::complete_partial_claim_note(encrypted_claim_note, note_defi_interaction_nonce);

        public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_X] = encrypted_claim_note.x;
        public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_Y] = encrypted_claim_note.y;
    }
}

/**
 * Check that claim proofs are using the correct defi root.
 */
auto process_claims(Composer& composer,
                    size_t num_inner_txs_pow2,
                    std::vector<field_ct>& public_inputs,
                    field_ct const& old_defi_root)
{
    for (size_t j = 0; j < num_inner_txs_pow2; j++) {
        const auto public_input_start_idx =
            RollupProofFields::INNER_PROOFS_DATA + (j * InnerProofFields::NUM_PUBLISHED);
        const auto proof_id = public_inputs[public_input_start_idx + InnerProofFields::PROOF_ID];
        // For claim proofs, defi root is output in field named PUBLIC_OWNER.
        const auto defi_root = public_inputs[public_input_start_idx + InnerProofFields::INPUT_OWNER];
        const auto is_claim = proof_id == field_ct(ProofIds::DEFI_CLAIM);

        auto valid = defi_root == old_defi_root || !is_claim;
        composer.assert_equal_constant(valid.witness_index, 1, format("claim proof has unmatched defi root"));
    }
}

/**
 * Computes the encryptions of the defi_interaction_notes to be inserted into the defi tree.
 * Checks the defi tree is updated with the encrypted defi_interaction_notes.
 * Returns the previous_defi_interaction_hash from the defi_interaction_notes.
 */
field_ct process_defi_interaction_notes(Composer& composer,
                                        field_ct const& rollup_id,
                                        field_ct const& new_defi_interaction_root,
                                        field_ct const& old_defi_interaction_root,
                                        merkle_tree::hash_path const& old_defi_interaction_path,
                                        field_ct const& num_previous_defi_interactions,
                                        std::vector<defi_interaction_note> const& defi_interaction_notes,
                                        std::vector<point_ct>& encrypted_defi_interaction_notes)
{
    byte_array_ct hash_input(&composer);
    std::vector<byte_array_ct> defi_interaction_note_leaves;
    auto not_first_rollup = rollup_id != 0;

    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto is_real = uint32_ct(i) < num_previous_defi_interactions && not_first_rollup;
        hash_input.write(defi_interaction_notes[i].to_byte_array(composer, is_real));
        const point_ct encrypted_note = { defi_interaction_notes[i].encrypted.x * is_real,
                                          defi_interaction_notes[i].encrypted.y * is_real };
        encrypted_defi_interaction_notes.push_back(encrypted_note);
        defi_interaction_note_leaves.push_back(
            byte_array_ct(&composer).write(encrypted_note.x).write(encrypted_note.y));
    }

    // Check defi interaction notes have been inserted into the defi interaction tree.
    auto insertion_index = ((rollup_id - 1) * NUM_BRIDGE_CALLS_PER_BLOCK * not_first_rollup).normalize();
    batch_update_membership(composer,
                            new_defi_interaction_root,
                            old_defi_interaction_root,
                            old_defi_interaction_path,
                            defi_interaction_note_leaves,
                            insertion_index,
                            "check_defi_tree_updated");

    const auto hash_output = plonk::stdlib::sha256<Composer>(hash_input);
    return field_ct(byte_array_ct(hash_output));
}

void assert_inner_proof_sequential(Composer& composer,
                                   size_t num_inner_txs_pow2,
                                   uint32_t i,
                                   field_ct& data_start_index,
                                   field_ct& old_data_root,
                                   field_ct& new_data_root,
                                   field_ct& old_null_root,
                                   field_ct& new_null_root,
                                   field_ct& old_root_root,
                                   std::vector<field_ct> const& public_inputs,
                                   bool_ct const& is_real)
{
    auto data_start_index_inner = public_inputs[RollupProofFields::DATA_START_INDEX];
    auto old_data_root_inner = public_inputs[RollupProofFields::OLD_DATA_ROOT];
    auto new_data_root_inner = public_inputs[RollupProofFields::NEW_DATA_ROOT];
    auto old_null_root_inner = public_inputs[RollupProofFields::OLD_NULL_ROOT];
    auto new_null_root_inner = public_inputs[RollupProofFields::NEW_NULL_ROOT];
    auto old_root_root_inner = public_inputs[RollupProofFields::OLD_DATA_ROOTS_ROOT];

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
            !is_real || data_start_index_inner == (data_start_index + (i * num_inner_txs_pow2 * 2));
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

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& root_rollup,
                                            size_t num_inner_txs_pow2,
                                            size_t num_outer_txs_pow2,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{

    auto max_num_inner_proofs = root_rollup.rollups.size();
    field_ct rollup_size = witness_ct(&composer, num_outer_txs_pow2);
    composer.assert_equal_constant(rollup_size.witness_index, num_outer_txs_pow2);

    uint32_ct num_inner_proofs = witness_ct(&composer, root_rollup.num_inner_proofs);
    field_ct rollup_id = witness_ct(&composer, root_rollup.rollup_id);
    field_ct data_start_index = witness_ct(&composer, 0);
    field_ct old_data_root = witness_ct(&composer, 0);
    field_ct new_data_root = witness_ct(&composer, 0);
    field_ct old_null_root = witness_ct(&composer, 0);
    field_ct new_null_root = witness_ct(&composer, 0);
    field_ct old_root_root = witness_ct(&composer, root_rollup.old_data_roots_root);
    field_ct new_root_root = witness_ct(&composer, root_rollup.new_data_roots_root);
    auto new_data_roots_path = create_witness_hash_path(composer, root_rollup.new_data_roots_path);
    auto old_data_roots_path = create_witness_hash_path(composer, root_rollup.old_data_roots_path);
    auto recursive_manifest = Composer::create_unrolled_manifest(inner_verification_key->num_public_inputs);
    auto recursive_verification_key =
        plonk::stdlib::recursion::verification_key<bn254>::from_constants(&composer, inner_verification_key);

    // Defi witnesses.
    field_ct num_defi_interactions = witness_ct(&composer, root_rollup.num_defi_interactions);
    field_ct num_previous_defi_interactions = witness_ct(&composer, root_rollup.num_previous_defi_interactions);
    field_ct old_defi_interaction_root = witness_ct(&composer, root_rollup.old_defi_interaction_root);
    field_ct new_defi_interaction_root = witness_ct(&composer, root_rollup.new_defi_interaction_root);
    auto old_defi_interaction_path = create_witness_hash_path(composer, root_rollup.old_defi_interaction_path);
    auto new_defi_interaction_path = create_witness_hash_path(composer, root_rollup.new_defi_interaction_path);
    auto bridge_ids = map(root_rollup.bridge_ids, [&](auto bid) { return field_ct(witness_ct(&composer, bid)); });
    auto defi_interaction_notes = map(root_rollup.defi_interaction_notes, [&](auto n) {
        return defi_interaction_note({ composer, n });
    });

    // Zero any input bridge_ids that are outside scope, and check in scope bridge_ids are not zero.
    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto in_scope = uint32_ct(i) < num_defi_interactions;
        bridge_ids[i] *= in_scope;
        auto valid = !in_scope || bridge_ids[i] != 0;
        composer.assert_equal_constant(valid.witness_index, 1, "bridge_id out of scope");
    }

    // Loop accumulators.
    recursion_output<bn254> recursion_output;
    std::vector<field_ct> tx_proof_public_inputs;
    std::vector<field_ct> total_tx_fees(NUM_ASSETS, field_ct::from_witness_index(&composer, composer.zero_idx));
    std::vector<field_ct> defi_deposit_sums(NUM_BRIDGE_CALLS_PER_BLOCK,
                                            field_ct::from_witness_index(&composer, composer.zero_idx));

    for (uint32_t i = 0; i < max_num_inner_proofs; ++i) {
        auto is_real = num_inner_proofs > i;

        recursion_output =
            verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(&composer,
                                                                          recursive_verification_key,
                                                                          recursive_manifest,
                                                                          waffle::plonk_proof{ root_rollup.rollups[i] },
                                                                          recursion_output);

        auto& public_inputs = recursion_output.public_inputs;

        // Zero all public inputs for padding proofs.
        for (auto& inp : public_inputs) {
            inp *= is_real;
        }

        // Accumulate tx fees.
        for (size_t j = 0; j < NUM_ASSETS; ++j) {
            total_tx_fees[j] += public_inputs[RollupProofFields::TOTAL_TX_FEES + j];
        }

        process_defi_deposits(composer,
                              rollup_id,
                              num_inner_txs_pow2,
                              public_inputs,
                              bridge_ids,
                              defi_deposit_sums,
                              num_defi_interactions);

        assert_inner_proof_sequential(composer,
                                      num_inner_txs_pow2,
                                      i,
                                      data_start_index,
                                      old_data_root,
                                      new_data_root,
                                      old_null_root,
                                      new_null_root,
                                      old_root_root,
                                      public_inputs,
                                      is_real);

        // Accumulate tx public inputs.
        for (size_t j = 0; j < InnerProofFields::NUM_PUBLISHED * num_inner_txs_pow2; ++j) {
            tx_proof_public_inputs.push_back(public_inputs[RollupProofFields::INNER_PROOFS_DATA + j]);
        }
    }

    // Check defi interaction notes are inserted and computes previous_defi_interaction_hash.
    std::vector<point_ct> encrypted_defi_interaction_notes;
    auto previous_defi_interaction_hash = process_defi_interaction_notes(composer,
                                                                         rollup_id,
                                                                         new_defi_interaction_root,
                                                                         old_defi_interaction_root,
                                                                         old_defi_interaction_path,
                                                                         num_previous_defi_interactions,
                                                                         defi_interaction_notes,
                                                                         encrypted_defi_interaction_notes);

    // Check data root tree is updated with latest data root.
    check_root_tree_updated(
        composer, new_data_roots_path, old_data_roots_path, rollup_id, new_data_root, new_root_root, old_root_root);

    // Publish public inputs.
    composer.set_public_input(rollup_id.witness_index);
    composer.set_public_input(rollup_size.witness_index);
    composer.set_public_input(data_start_index.witness_index);
    composer.set_public_input(old_data_root.witness_index);
    composer.set_public_input(new_data_root.witness_index);
    composer.set_public_input(old_null_root.witness_index);
    composer.set_public_input(new_null_root.witness_index);
    composer.set_public_input(old_root_root.witness_index);
    composer.set_public_input(new_root_root.witness_index);

    for (auto total_tx_fee : total_tx_fees) {
        composer.set_public_input(total_tx_fee.witness_index);
    }

    for (auto& inp : tx_proof_public_inputs) {
        composer.set_public_input(inp.witness_index);
    }

    for (size_t i = max_num_inner_proofs; i < num_outer_txs_pow2 / num_inner_txs_pow2; ++i) {
        add_padding_public_inputs(composer, num_inner_txs_pow2);
    }

    recursion_output.add_proof_outputs_as_public_inputs();

    // The root rollup has the same public input structure as the inner rollup, until this point.
    // The following public inputs support the defi bridge.
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        composer.set_public_input(bridge_ids[i].witness_index);
        composer.set_public_input(defi_deposit_sums[i].witness_index);
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        encrypted_defi_interaction_notes[i].set_public();
    }
    composer.set_public_input(previous_defi_interaction_hash.witness_index);

    return recursion_output;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
