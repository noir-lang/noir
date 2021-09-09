#include "../../constants.hpp"
#include "../rollup/index.hpp"
#include "../inner_proof_data.hpp"
#include "../notes/constants.hpp"
#include "../notes/circuit/index.hpp"
#include "root_rollup_circuit.hpp"
#include <stdlib/merkle_tree/index.hpp>
#include <stdlib/hash/sha256/sha256.hpp>
#include <common/map.hpp>
#include "./root_rollup_proof_data.hpp"

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

field_ct compute_sha256_of_zeroes(Composer& composer, const size_t num_txs_per_rollup)
{
    std::vector<uint8_t> data;
    for (size_t i = 0; i < 32 * PropagatedInnerProofFields::NUM_FIELDS * num_txs_per_rollup; ++i) {
        data.emplace_back(0);
    }
    auto hash_result = sha256::sha256(data);
    fr hash_reduced = fr::serialize_from_buffer(&hash_result[0]);
    return field_ct(&composer, hash_reduced);
}

void add_rollup_padding_public_inputs(Composer& composer, size_t inner_size)
{
    for (size_t i = 0; i < inner_size; ++i) {
        add_tx_padding_public_inputs(composer);
    }
}

void add_zero_public_input(Composer& composer)
{
    auto zero = field_ct(witness_ct(&composer, 0));
    zero.assert_is_zero();
    zero.set_public();
}

/**
 * Inserts the latest data root into the root tree at location rollup_id + 1.
 */
void check_root_tree_updated(merkle_tree::hash_path const& old_data_roots_path,
                             field_ct const& rollup_id,
                             field_ct const& new_data_root,
                             field_ct const& new_data_roots_root,
                             field_ct const& old_data_roots_root)
{
    auto index = byte_array_ct(rollup_id + 1);
    update_membership(
        new_data_roots_root, new_data_root, old_data_roots_root, old_data_roots_path, field_ct(0), index, __FUNCTION__);
}

/**
 * Computes the commitments to the defi_interaction_notes to be inserted into the defi tree.
 * Checks the defi tree is updated with the defi_interaction_notes commitments.
 * Returns the previous_defi_interaction_hash from the defi_interaction_notes.
 */
field_ct process_defi_interaction_notes(Composer& composer,
                                        field_ct const& rollup_id,
                                        field_ct const& new_defi_interaction_root,
                                        field_ct const& old_defi_interaction_root,
                                        merkle_tree::hash_path const& old_defi_interaction_path,
                                        field_ct const& num_previous_defi_interactions,
                                        std::vector<circuit::defi_interaction::note> const& defi_interaction_notes,
                                        std::vector<field_ct>& defi_interaction_note_commitments)
{
    byte_array_ct hash_input(&composer);
    auto not_first_rollup = rollup_id != 0;

    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto is_real = uint32_ct(i) < num_previous_defi_interactions && not_first_rollup;
        hash_input.write(defi_interaction_notes[i].to_byte_array(composer, is_real));
        auto note_commitment = defi_interaction_notes[i].commitment * is_real;
        defi_interaction_note_commitments.push_back(note_commitment);
    }

    // Check defi interaction notes have been inserted into the defi interaction tree.
    auto insertion_index = ((rollup_id - 1) * NUM_BRIDGE_CALLS_PER_BLOCK * not_first_rollup);
    batch_update_membership(new_defi_interaction_root,
                            old_defi_interaction_root,
                            old_defi_interaction_path,
                            defi_interaction_note_commitments,
                            insertion_index,
                            "check_defi_tree_updated");

    auto hash_output = byte_array_ct(plonk::stdlib::sha256<Composer>(hash_input));
    // Zero the first 4 bits to ensure field conversion doesn't wrap around prime.
    for (size_t i = 252; i < 256; ++i) {
        hash_output.set_bit(i, false);
    }
    return field_ct(hash_output);
}

void check_asset_ids_and_accumulate_tx_fees(Composer& composer,
                                            uint32_t const i,
                                            std::vector<field_ct>& total_tx_fees,
                                            std::vector<field_ct> const& asset_ids,
                                            std::vector<field_ct> const& public_inputs,
                                            bool_ct const& is_real)
{
    // Check every real tx rollup proof has correct asset ids.
    for (size_t j = 0; j < NUM_ASSETS; j++) {

        field_ct num_matched(&composer, 0);
        auto inner_asset_id = public_inputs[RollupProofFields::ASSET_IDS + j];
        auto inner_tx_fee = public_inputs[RollupProofFields::TOTAL_TX_FEES + j];
        auto is_asset_id_padded = (inner_asset_id == field_ct(MAX_NUM_ASSETS));

        for (uint32_t k = 0; k < NUM_ASSETS; k++) {
            const auto matches = (inner_asset_id == asset_ids[k]);
            num_matched += matches;

            // Sum the real tx rollup proof's tx fee according to the matched asset id.
            total_tx_fees[k] += (inner_tx_fee * matches * !is_asset_id_padded);
        }

        // Assert that the tx rollup proof's asset_id matched a single asset_id.
        auto is_valid_asset_id = !is_real || num_matched == 1 || is_asset_id_padded;
        is_valid_asset_id.assert_equal(true,
                                       format("rollup proof ",
                                              i,
                                              "'s asset id ",
                                              uint64_t(inner_asset_id.get_value()),
                                              " matched ",
                                              uint64_t(num_matched.get_value()),
                                              " times."));
    }
}

void check_bridge_ids_and_accumulate_defi_deposits(Composer& composer,
                                                   uint32_t const i,
                                                   std::vector<field_ct>& defi_deposit_sums,
                                                   std::vector<field_ct> const& bridge_ids,
                                                   std::vector<field_ct> const& public_inputs,
                                                   bool_ct const& is_real)
{
    // Check every real tx rollup proof has correct bridge id.
    for (size_t j = 0; j < NUM_BRIDGE_CALLS_PER_BLOCK; j++) {

        field_ct num_matched(&composer, 0);
        auto inner_bridge_id = public_inputs[RollupProofFields::DEFI_BRIDGE_IDS + j];
        auto inner_defi_deposit_sum = public_inputs[RollupProofFields::DEFI_BRIDGE_DEPOSITS + j];
        auto is_bridge_id_zero = inner_bridge_id.is_zero();

        for (uint32_t k = 0; k < NUM_BRIDGE_CALLS_PER_BLOCK; k++) {
            const auto matches = (inner_bridge_id == bridge_ids[k]);
            num_matched += matches;

            // Sum the real tx rollup proof's tx fee according to the matched asset id.
            defi_deposit_sums[k] += (inner_defi_deposit_sum * matches * !is_bridge_id_zero);
        }

        // Assert that the tx rollup proof's asset_id matched a single asset_id.
        auto is_valid_bridge_id = !is_real || (num_matched == 1 || is_bridge_id_zero);
        is_valid_bridge_id.assert_equal(true,
                                        format("rollup proof ",
                                               i,
                                               "'s bridge id at index ",
                                               j,
                                               " matched ",
                                               uint64_t(num_matched.get_value()),
                                               " times."));
    }
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

    // Every real inner proof should use the root tree root we've input.
    auto valid_root_root = !is_real || old_root_root_inner == old_root_root;
    valid_root_root.assert_equal(true, format("inconsistent_roots_root_", i));

    // Every real inner proof should use the defi root we've input.
    auto valid_defi_root = !is_real || new_defi_root_inner == new_defi_root;
    valid_defi_root.assert_equal(true, format("inconsistent_defi_root_", i));

    if (i == 0) {
        // The first proof should always be real.
        is_real.assert_equal(true, "root rollup first proof is not real");
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

        valid_rollup_id.assert_equal(true, format("incorrect_rollup_id_", i));
        valid_data_start_index.assert_equal(true, format("incorrect_data_start_index_", i));
        valid_old_data_root.assert_equal(true, format("inconsistent_old_data_root_", i));
        valid_old_null_root.assert_equal(true, format("inconsistent_old_null_root_", i));

        new_data_root = (new_data_root_inner * is_real) + (new_data_root * !is_real);
        new_null_root = (new_null_root_inner * is_real) + (new_null_root * !is_real);
    }
}

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& tx,
                                            size_t num_inner_txs_pow2,
                                            size_t num_outer_txs_pow2,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key,
                                            root_rollup_proof_data& data)
{
    auto max_num_inner_proofs = tx.rollups.size();
    // Witnesses.
    const auto rollup_id = field_ct(witness_ct(&composer, tx.rollup_id));
    const auto rollup_size_pow2 = field_ct(witness_ct(&composer, num_outer_txs_pow2));
    rollup_size_pow2.assert_equal(num_outer_txs_pow2);
    const auto num_inner_proofs = uint32_ct(witness_ct(&composer, tx.num_inner_proofs));
    const auto old_root_root = field_ct(witness_ct(&composer, tx.old_data_roots_root));
    const auto new_root_root = field_ct(witness_ct(&composer, tx.new_data_roots_root));
    const auto old_root_path = create_witness_hash_path(composer, tx.old_data_roots_path);
    const auto old_defi_root = field_ct(witness_ct(&composer, tx.old_defi_root));
    const auto new_defi_root = field_ct(witness_ct(&composer, tx.new_defi_root));
    const auto old_defi_path = create_witness_hash_path(composer, tx.old_defi_path);
    const auto bridge_ids = map(tx.bridge_ids, [&](auto& bid) { return field_ct(witness_ct(&composer, bid)); });
    const auto asset_ids = map(tx.asset_ids, [&](auto& aid) { return field_ct(witness_ct(&composer, aid)); });
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
    std::vector<field_ct> total_tx_fees(NUM_ASSETS, field_ct(witness_ct::create_constant_witness(&composer, 0)));
    std::vector<field_ct> defi_deposit_sums(NUM_BRIDGE_CALLS_PER_BLOCK,
                                            field_ct(witness_ct::create_constant_witness(&composer, 0)));

    field_ct zero_hash = compute_sha256_of_zeroes(composer, num_inner_txs_pow2);
    std::vector<field_ct> inner_input_hashes;
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
        check_asset_ids_and_accumulate_tx_fees(composer, i, total_tx_fees, asset_ids, public_inputs, is_real);

        // Accumulate defi deposits.
        check_bridge_ids_and_accumulate_defi_deposits(
            composer, i, defi_deposit_sums, bridge_ids, public_inputs, is_real);

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
                                      public_inputs,
                                      is_real);

        field_ct hash = field_ct::conditional_assign(is_real, public_inputs[RollupProofFields::INPUTS_HASH], zero_hash);
        inner_input_hashes.push_back(hash);
        // Accumulate tx public inputs.
        for (size_t j = 0; j < PropagatedInnerProofFields::NUM_FIELDS * num_inner_txs_pow2; ++j) {
            tx_proof_public_inputs.push_back(public_inputs[RollupProofFields::INNER_PROOFS_DATA + j]);
        }
    }

    // Check defi interaction notes are inserted and computes previous_defi_interaction_hash.
    std::vector<field_ct> defi_interaction_note_commitments;
    auto previous_defi_interaction_hash = process_defi_interaction_notes(composer,
                                                                         rollup_id,
                                                                         new_defi_root,
                                                                         old_defi_root,
                                                                         old_defi_path,
                                                                         num_previous_defi_interactions,
                                                                         defi_interaction_notes,
                                                                         defi_interaction_note_commitments);

    // Check data root tree is updated with latest data root.
    check_root_tree_updated(old_root_path, rollup_id, new_data_root, new_root_root, old_root_root);
    /**
     * Construct vector of broadcasted inputs to SHA256
     * A "broadcasted" input is a variable that is published on-chain when publishing a rollup block.
     * However they are not public inputs to the proof.
     * We SHA256 hash the broadcasted inputs and set the hash output as a public input.
     * This reduces the number of public inputs to the proof, which reduces verifier gas costs
     *
     * N.B. The order of this vector MUST be the same as the enum structure in RootRollupProofFields
     */
    std::vector<field_ct> broadcasted_inputs;
    broadcasted_inputs.resize(
        RootRollupProofFields::INNER_PROOFS_DATA); // max number of fields we can create absolute offsets to

    broadcasted_inputs[RootRollupProofFields::ROLLUP_ID] = rollup_id;
    broadcasted_inputs[RootRollupProofFields::ROLLUP_SIZE] = rollup_size_pow2;
    broadcasted_inputs[RootRollupProofFields::DATA_START_INDEX] = data_start_index;
    broadcasted_inputs[RootRollupProofFields::OLD_DATA_ROOT] = old_data_root;
    broadcasted_inputs[RootRollupProofFields::NEW_DATA_ROOT] = new_data_root;
    broadcasted_inputs[RootRollupProofFields::OLD_NULL_ROOT] = old_null_root;
    broadcasted_inputs[RootRollupProofFields::NEW_NULL_ROOT] = new_null_root;
    broadcasted_inputs[RootRollupProofFields::OLD_DATA_ROOTS_ROOT] = old_root_root;
    broadcasted_inputs[RootRollupProofFields::NEW_DATA_ROOTS_ROOT] = new_root_root;
    broadcasted_inputs[RootRollupProofFields::OLD_DEFI_ROOT] = old_defi_root;
    broadcasted_inputs[RootRollupProofFields::NEW_DEFI_ROOT] = new_defi_root;
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        broadcasted_inputs[RootRollupProofFields::DEFI_BRIDGE_IDS + i] = bridge_ids[i];
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        broadcasted_inputs[RootRollupProofFields::DEFI_BRIDGE_DEPOSITS + i] = defi_deposit_sums[i];
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        broadcasted_inputs[RootRollupProofFields::ASSET_IDS + i] = asset_ids[i];
    }

    for (size_t i = 0; i < total_tx_fees.size(); ++i) {
        broadcasted_inputs[RootRollupProofFields::TOTAL_TX_FEES + i] = total_tx_fees[i];
    }

    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        broadcasted_inputs[RootRollupProofFields::DEFI_INTERACTION_NOTES + i] = defi_interaction_note_commitments[i];
    }
    broadcasted_inputs[RootRollupProofFields::PREVIOUS_DEFI_INTERACTION_HASH] = previous_defi_interaction_hash;
    const size_t num_rollups = num_outer_txs_pow2 / num_inner_txs_pow2;
    // TODO: RootRollupProofFields::NUM_ROLLUP_TXS doesn't need to go into the SHA256 hash or be a public input! Just
    // need to add into RollupProofData as it is required to correctly encode input data
    broadcasted_inputs[RootRollupProofFields::NUM_ROLLUP_TXS] = field_ct(witness_ct(&composer, num_rollups));
    // The root rollup has the same public input structure as the inner rollup, until this point.
    // rollup_data_fields.push_back(previous_defi_interaction_hash);

    // The broadcasted public inputs includes the inner inputs of our join-split proofs
    // However the data we want to hash substitutes these for their aggregated SHA256 hashes computed in each rollup
    // proof (this is to reduce verifier gas costs. We hash inner inputs in blocks of 32 transactions, and then
    //  insert these intermediate hashes into the root rollup hash. EVM memory consumption gas costs scales
    //  quadratically with the amount of memory used - better to hash in small chunks than to hash the whole dataset at
    //  once)
    std::vector<field_ct> inputs_to_hash(broadcasted_inputs);
    for (size_t i = 0; i < max_num_inner_proofs; ++i) {
        inputs_to_hash.push_back(inner_input_hashes[i]);
    }
    for (size_t i = max_num_inner_proofs; i < num_rollups; ++i) {
        inputs_to_hash.push_back(zero_hash);
    }
    packed_byte_array_ct input_msg = packed_byte_array_ct::from_field_element_vector(inputs_to_hash);
    std::vector<field_ct> broadcasted_inputs_hash = stdlib::sha256<Composer>(input_msg).to_unverified_byte_slices(16);
    field_ct broadcasted_inputs_hash_reduced =
        broadcasted_inputs_hash[1] + (broadcasted_inputs_hash[0] * (uint256_t(1) << 128));
    // N.B. don't want the remaining pseudo public input data in the sha256 hash as they are present as public inputs or
    // are derived
    for (auto& inp : tx_proof_public_inputs) {
        broadcasted_inputs.push_back(inp);
    }
    for (size_t i = max_num_inner_proofs; i < num_outer_txs_pow2 / num_inner_txs_pow2; ++i) {

        for (size_t j = 0; j < PropagatedInnerProofFields::NUM_FIELDS * num_inner_txs_pow2; ++j) {
            broadcasted_inputs.push_back(field_ct(witness_ct(&composer, 0))); // todo force to be zero
        }
    }
    data = root_rollup_proof_data(broadcasted_inputs);

    broadcasted_inputs_hash_reduced.set_public();

    recursion_output.add_proof_outputs_as_public_inputs();
    return recursion_output;
}

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& tx,
                                            size_t num_inner_txs_pow2,
                                            size_t num_outer_txs_pow2,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{
    root_rollup_proof_data data;
    return root_rollup_circuit(composer, tx, num_inner_txs_pow2, num_outer_txs_pow2, inner_verification_key, data);
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
