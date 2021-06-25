#include "../../constants.hpp"
#include "../rollup/index.hpp"
#include "../inner_proof_data.hpp"
#include "../notes/constants.hpp"
#include "../notes/circuit/index.hpp"
#include "root_rollup_circuit.hpp"
#include <stdlib/merkle_tree/index.hpp>
#include <stdlib/hash/sha256/sha256.hpp>
#include <common/map.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace rollup;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;
using namespace notes;

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
                      {},
                      new_data_root_arr,
                      old_data_roots_root,
                      old_data_roots_path,
                      empty_tree_value,
                      index,
                      __FUNCTION__);
}

/**
 * Computes the commitments to the defi_interaction_notes to be inserted into the defi tree.
 * Checks the defi tree is updated with the defi_interaction_notes' commitments.
 * Returns the previous_defi_interaction_hash from the defi_interaction_notes.
 */
field_ct process_defi_interaction_notes(Composer& composer,
                                        field_ct const& rollup_id,
                                        field_ct const& new_defi_interaction_root,
                                        field_ct const& old_defi_interaction_root,
                                        merkle_tree::hash_path const& old_defi_interaction_path,
                                        field_ct const& num_previous_defi_interactions,
                                        std::vector<circuit::defi_interaction::note> const& defi_interaction_notes,
                                        std::vector<point_ct>& defi_interaction_note_commitments)
{
    byte_array_ct hash_input(&composer);
    std::vector<byte_array_ct> defi_interaction_note_leaves;
    auto not_first_rollup = rollup_id != 0;

    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto is_real = uint32_ct(i) < num_previous_defi_interactions && not_first_rollup;
        hash_input.write(defi_interaction_notes[i].to_byte_array(composer, is_real));
        const point_ct note_commitment = { defi_interaction_notes[i].commitment.x * is_real,
                                           defi_interaction_notes[i].commitment.y * is_real };
        defi_interaction_note_commitments.push_back(note_commitment);
        defi_interaction_note_leaves.push_back(
            byte_array_ct(&composer).write(note_commitment.x).write(note_commitment.y));
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

    auto hash_output = byte_array_ct(plonk::stdlib::sha256<Composer>(hash_input));
    // Zero the first 4 bits to ensure field conversion doesn't wrap around prime.
    for (size_t i = 252; i < 256; ++i) {
        hash_output.set_bit(i, false);
    }
    return field_ct(hash_output);
}

void assert_inner_proof_sequential(Composer& composer,
                                   size_t const num_inner_txs_pow2,
                                   uint32_t const i,
                                   field_ct const& rollup_id,
                                   field_ct& data_start_index,
                                   field_ct& old_data_root,
                                   field_ct& new_data_root,
                                   field_ct& old_null_root,
                                   field_ct& new_null_root,
                                   field_ct const& old_root_root,
                                   field_ct const& new_defi_root,
                                   std::vector<field_ct> const& bridge_ids,
                                   std::vector<field_ct> const& public_inputs,
                                   bool_ct const& is_real)
{
    auto rollup_id_inner = public_inputs[RollupProofFields::ROLLUP_ID];
    auto data_start_index_inner = public_inputs[RollupProofFields::DATA_START_INDEX];
    auto old_data_root_inner = public_inputs[RollupProofFields::OLD_DATA_ROOT];
    auto new_data_root_inner = public_inputs[RollupProofFields::NEW_DATA_ROOT];
    auto old_null_root_inner = public_inputs[RollupProofFields::OLD_NULL_ROOT];
    auto new_null_root_inner = public_inputs[RollupProofFields::NEW_NULL_ROOT];
    auto old_root_root_inner = public_inputs[RollupProofFields::OLD_DATA_ROOTS_ROOT];
    auto new_defi_root_inner = public_inputs[RollupProofFields::NEW_DEFI_ROOT];

    // Check every real inner proof has matching bridge ids.
    for (size_t j = 0; j < NUM_BRIDGE_CALLS_PER_BLOCK; ++j) {
        auto valid_bid = !is_real || public_inputs[RollupProofFields::DEFI_BRIDGE_IDS + j] == bridge_ids[j];
        composer.assert_equal_constant(valid_bid, 1, format("inconsistent_bridge_id_", j));
    }

    // Every real inner proof should use the root tree root we've input.
    auto valid_root_root = !is_real || old_root_root_inner == old_root_root;
    composer.assert_equal_constant(valid_root_root, 1, format("inconsistent_roots_root_", i));

    // Every real inner proof should use the defi root we've input.
    auto valid_defi_root = !is_real || new_defi_root_inner == new_defi_root;
    composer.assert_equal_constant(valid_defi_root, 1, format("inconsistent_defi_root_", i));

    if (i == 0) {
        // The first proof should always be real.
        composer.assert_equal_constant(is_real, 1);
        data_start_index = data_start_index_inner;
        old_data_root = old_data_root_inner;
        new_data_root = new_data_root_inner;
        old_null_root = old_null_root_inner;
        new_null_root = new_null_root_inner;
    } else {
        auto valid_rollup_id = !is_real || rollup_id_inner == rollup_id;
        auto valid_data_start_index =
            !is_real || data_start_index_inner == (data_start_index + (i * num_inner_txs_pow2 * 2));
        auto valid_old_data_root = !is_real || old_data_root_inner == new_data_root;
        auto valid_old_null_root = !is_real || old_null_root_inner == new_null_root;

        composer.assert_equal_constant(valid_rollup_id, 1, format("incorrect_rollup_id_", i));
        composer.assert_equal_constant(valid_data_start_index, 1, format("incorrect_data_start_index_", i));
        composer.assert_equal_constant(valid_old_data_root, 1, format("inconsistent_old_data_root_", i));
        composer.assert_equal_constant(valid_old_null_root, 1, format("inconsistent_old_null_root_", i));

        new_data_root = (new_data_root_inner * is_real) + (new_data_root * !is_real);
        new_null_root = (new_null_root_inner * is_real) + (new_null_root * !is_real);
    }
}

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& tx,
                                            size_t num_inner_txs_pow2,
                                            size_t num_outer_txs_pow2,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{
    auto max_num_inner_proofs = tx.rollups.size();

    // Witnesses.
    const auto rollup_id = field_ct(witness_ct(&composer, tx.rollup_id));
    const auto rollup_size_pow2 = field_ct(witness_ct(&composer, num_outer_txs_pow2));
    composer.assert_equal_constant(rollup_size_pow2, num_outer_txs_pow2);
    const auto num_inner_proofs = uint32_ct(witness_ct(&composer, tx.num_inner_proofs));
    const auto old_root_root = field_ct(witness_ct(&composer, tx.old_data_roots_root));
    const auto new_root_root = field_ct(witness_ct(&composer, tx.new_data_roots_root));
    const auto old_root_path = create_witness_hash_path(composer, tx.old_data_roots_path);
    const auto old_defi_root = field_ct(witness_ct(&composer, tx.old_defi_root));
    const auto new_defi_root = field_ct(witness_ct(&composer, tx.new_defi_root));
    const auto old_defi_path = create_witness_hash_path(composer, tx.old_defi_path);
    const auto bridge_ids = map(tx.bridge_ids, [&](auto& bid) { return field_ct(witness_ct(&composer, bid)); });
    const auto defi_interaction_notes = map(tx.defi_interaction_notes, [&](auto n) {
        return circuit::defi_interaction::note(circuit::defi_interaction::witness_data(composer, n));
    });
    const auto num_previous_defi_interactions = field_ct(witness_ct(&composer, tx.num_previous_defi_interactions));
    const auto recursive_manifest = Composer::create_unrolled_manifest(inner_verification_key->num_public_inputs);
    const auto recursive_verification_key =
        plonk::stdlib::recursion::verification_key<bn254>::from_constants(&composer, inner_verification_key);

    // To be extracted from inner proofs.
    field_ct data_start_index = witness_ct(&composer, 0);
    field_ct old_data_root = witness_ct(&composer, 0);
    field_ct new_data_root = witness_ct(&composer, 0);
    field_ct old_null_root = witness_ct(&composer, 0);
    field_ct new_null_root = witness_ct(&composer, 0);

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
                                                                          waffle::plonk_proof{ tx.rollups[i] },
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

        // Accumulate defi deposits.
        for (size_t j = 0; j < NUM_BRIDGE_CALLS_PER_BLOCK; ++j) {
            defi_deposit_sums[j] += public_inputs[RollupProofFields::DEFI_BRIDGE_DEPOSITS + j];
        }

        assert_inner_proof_sequential(composer,
                                      num_inner_txs_pow2,
                                      i,
                                      rollup_id,
                                      data_start_index,
                                      old_data_root,
                                      new_data_root,
                                      old_null_root,
                                      new_null_root,
                                      old_root_root,
                                      new_defi_root,
                                      bridge_ids,
                                      public_inputs,
                                      is_real);

        // Accumulate tx public inputs.
        for (size_t j = 0; j < InnerProofFields::NUM_PUBLISHED * num_inner_txs_pow2; ++j) {
            tx_proof_public_inputs.push_back(public_inputs[RollupProofFields::INNER_PROOFS_DATA + j]);
        }
    }

    // Check defi interaction notes are inserted and computes previous_defi_interaction_hash.
    std::vector<point_ct> defi_interaction_note_commitments;
    auto previous_defi_interaction_hash = process_defi_interaction_notes(composer,
                                                                         rollup_id,
                                                                         new_defi_root,
                                                                         old_defi_root,
                                                                         old_defi_path,
                                                                         num_previous_defi_interactions,
                                                                         defi_interaction_notes,
                                                                         defi_interaction_note_commitments);

    // Check data root tree is updated with latest data root.
    check_root_tree_updated(composer, old_root_path, rollup_id, new_data_root, new_root_root, old_root_root);

    // Publish public inputs.
    composer.set_public_input(rollup_id.witness_index);
    composer.set_public_input(rollup_size_pow2.witness_index);
    composer.set_public_input(data_start_index.witness_index);
    composer.set_public_input(old_data_root.witness_index);
    composer.set_public_input(new_data_root.witness_index);
    composer.set_public_input(old_null_root.witness_index);
    composer.set_public_input(new_null_root.witness_index);
    composer.set_public_input(old_root_root.witness_index);
    composer.set_public_input(new_root_root.witness_index);
    composer.set_public_input(old_defi_root.witness_index);
    composer.set_public_input(new_defi_root.witness_index);
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        composer.set_public_input(bridge_ids[i].witness_index);
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        composer.set_public_input(defi_deposit_sums[i].witness_index);
    }
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
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        defi_interaction_note_commitments[i].set_public();
    }
    composer.set_public_input(previous_defi_interaction_hash.witness_index);

    return recursion_output;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
