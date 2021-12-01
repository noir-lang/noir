#include "rollup_circuit.hpp"
#include "./rollup_proof_data.hpp"
#include "../../constants.hpp"
#include "../inner_proof_data/inner_proof_data.hpp"
#include "../add_zero_public_inputs.hpp"
#include "../notes/circuit/claim/index.hpp"
#include <stdlib/merkle_tree/index.hpp>
#include <stdlib/hash/sha256/sha256.hpp>
#include <common/map.hpp>
#include <common/container.hpp>
#include "../notes/constants.hpp"

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace rollup {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;
using namespace notes;

field_ct check_nullifiers_inserted(Composer& composer,
                                   std::vector<field_ct> const& new_null_roots,
                                   std::vector<merkle_tree::hash_path> const& old_null_paths,
                                   uint32_ct const& num_txs,
                                   field_ct latest_null_root,
                                   std::vector<field_ct> const& new_null_indicies)
{
    for (size_t i = 0; i < new_null_indicies.size(); ++i) {
        auto is_real = num_txs > uint32_ct(&composer, i / 2) && new_null_indicies[i] != 0;

        // This makes padding transactions act as noops.
        auto index = (new_null_indicies[i] * is_real);

        update_membership(new_null_roots[i],
                          field_ct(is_real),
                          latest_null_root,
                          old_null_paths[i],
                          field_ct(0),
                          byte_array_ct(index),
                          format(__FUNCTION__, "_", i));

        latest_null_root = new_null_roots[i];
    }

    return latest_null_root;
}

/**
 * Processes a defi deposit proof.
 * - We only process join split proofs with a proof_id == ProofIds::DEFI_DEPOSIT (otherwise noop).
 * - Ensure that the bridge_id matches one within the of set of bridge_ids.
 * - Accumulate the deposit value in relevant defi_deposit_sums slot. These later become public inputs.
 * - Modify the claim note commitment (output_note_1 commitment) to add the relevant interaction nonce to it.
 */
auto process_defi_deposit(Composer& composer,
                          field_ct const& rollup_id,
                          std::vector<field_ct>& public_inputs,
                          std::vector<suint_ct> const& bridge_ids,
                          std::vector<suint_ct>& defi_deposit_sums,
                          field_ct const& num_defi_interactions)
{
    field_ct defi_interaction_nonce = (rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK);

    const auto proof_id = public_inputs[InnerProofFields::PROOF_ID];
    const suint_ct bridge_id(public_inputs[InnerProofFields::BRIDGE_ID], DEFI_BRIDGE_ID_BIT_LENGTH, "bridge_id");
    const suint_ct deposit_value(
        public_inputs[InnerProofFields::DEFI_DEPOSIT_VALUE], DEFI_DEPOSIT_VALUE_BIT_LENGTH, "defi_deposit");
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
    auto is_valid_bridge_id = num_matched == 1 || !is_defi_deposit;
    is_valid_bridge_id.assert_equal(true,
                                    format("proof bridge id matched ", uint64_t(num_matched.get_value()), " times"));

    // Compute claim fee which to be added to the claim note.
    const suint_ct tx_fee(public_inputs[InnerProofFields::TX_FEE], TX_FEE_BIT_LENGTH, "tx_fee");
    const suint_ct defi_deposit_fee = tx_fee / 2;
    const auto claim_fee = (tx_fee - defi_deposit_fee) * is_defi_deposit;
    const auto net_tx_fee = tx_fee * !is_defi_deposit + defi_deposit_fee * is_defi_deposit;

    // Complete the claim note output to mix in the interaction nonce and the claim fee.
    auto note_commitment1 = public_inputs[InnerProofFields::NOTE_COMMITMENT1];
    auto claim_note_commitment =
        notes::circuit::claim::complete_partial_commitment(note_commitment1, note_defi_interaction_nonce, claim_fee);

    public_inputs[InnerProofFields::NOTE_COMMITMENT1] =
        note_commitment1 * !is_defi_deposit + claim_note_commitment * is_defi_deposit;

    return net_tx_fee;
}

/**
 * Check that claim proofs are using the correct defi root.
 */
auto process_claims(std::vector<field_ct>& public_inputs, field_ct const& new_defi_root)
{
    const auto is_claim = public_inputs[InnerProofFields::PROOF_ID] == field_ct(ProofIds::DEFI_CLAIM);
    const auto defi_root = public_inputs[InnerProofFields::DEFI_ROOT];
    auto valid = defi_root == new_defi_root || !is_claim;
    valid.assert_equal(true, format("claim proof has unmatched defi root"));
}

/**
 * Check chained transaction inputs - called once per tx `i`.
 * - Look back over all earlier txs in the rollup for other txs in the chain.
 * - Perform a membership check for the propagated inputs of txs at the start of a split chain.
 * - 'Zero' the commitments and nullifiers of notes propagated to a user's self.
 *
 * @param all_tx_public_inputs is required to extract the allow_chain public input from each tx
 * @returns the (possibly zeroed) nullifiers of this tx
 */
std::pair<field_ct, field_ct> process_chained_txs(size_t const& i,
                                                  bool_ct const& is_tx_real,
                                                  std::vector<field_ct> const& public_inputs,
                                                  std::vector<std::vector<field_ct>> const& all_tx_public_inputs,
                                                  std::vector<std::vector<field_ct>>& propagated_tx_public_inputs,
                                                  std::vector<field_ct>& new_data_values,
                                                  field_ct const& old_data_root,
                                                  std::vector<merkle_tree::hash_path> const& linked_commitment_paths,
                                                  std::vector<field_ct> const& linked_commitment_indices)
{
    const field_ct propagated_input_index = field_ct(public_inputs[InnerProofFields::PROPAGATED_INPUT_INDEX]);
    const field_ct backward_link = field_ct(public_inputs[InnerProofFields::BACKWARD_LINK]);
    field_ct nullifier1 = field_ct(public_inputs[InnerProofFields::NULLIFIER1]);
    field_ct nullifier2 = field_ct(public_inputs[InnerProofFields::NULLIFIER2]);

    const bool_ct chaining = propagated_input_index != 0; // range check in {1, 2} already done in j-s circuit

    // If (chaining), we need to look back at all earlier txs in this rollup, to find a match to this tx's
    // backward_link.
    // This is O(n^2) in the number of txs.
    // Note, there might not be a match if the chain has been split across rollups.

    // Loop accumulators:
    field_ct prev_allow_chain(0);
    bool_ct is_propagating_prev_output1(false);
    bool_ct is_propagating_prev_output2(false);
    bool_ct found_link_in_rollup(false);
    field_ct matched_tx_index(0);

    for (size_t j = 0; j < i; j++) {
        const auto prev_public_inputs = all_tx_public_inputs[j];
        const field_ct prev_note_commitment1 = prev_public_inputs[InnerProofFields::NOTE_COMMITMENT1];
        const field_ct prev_note_commitment2 = prev_public_inputs[InnerProofFields::NOTE_COMMITMENT2];
        const field_ct temp_prev_allow_chain = prev_public_inputs[InnerProofFields::ALLOW_CHAIN];

        const bool_ct temp_is_propagating_prev_output1 =
            (backward_link == prev_note_commitment1) &&
            is_tx_real; // Inclusion of `is_tx_real` prevents `0 == 0` from passing, for padded txs (which have a 0
                        // prev_note_commitment).
        const bool_ct temp_is_propagating_prev_output2 = (backward_link == prev_note_commitment2) && is_tx_real;
        const bool_ct found_link_in_loop = temp_is_propagating_prev_output1 || temp_is_propagating_prev_output2;

        // If we've found a tx which matches this tx's backward_link, then write data to the higher-scoped variables:
        // Note: we don't need to try to prevent multiple matches (and hence multiple writes to the higher-scoped
        // variables) in this loop. Multiple matches would mean there are >1 txs with the same output commitment, which
        // is a bigger problem that will be caught when updating the nullifier tree (duplicate output commitments would
        // share the same input_nullifier).
        // Notice: once found, the below values remain unchanged through future iterations:
        found_link_in_rollup |= found_link_in_loop;
        prev_allow_chain = field_ct::conditional_assign(found_link_in_loop, temp_prev_allow_chain, prev_allow_chain);
        is_propagating_prev_output1 = bool_ct(field_ct::conditional_assign(
            found_link_in_loop, temp_is_propagating_prev_output1, is_propagating_prev_output1));
        is_propagating_prev_output2 = bool_ct(field_ct::conditional_assign(
            found_link_in_loop, temp_is_propagating_prev_output2, is_propagating_prev_output2));

        // Interestingly, we can't just set matched_tx_index = j, since they're incompatible types.
        // This makes sense, since the loop iterator exists 'outside the circuit', so it can't be accessed
        // by circuit variables. We need to iterate our own circuit variable:
        matched_tx_index += !found_link_in_rollup; // increments until a match is found
    }
    matched_tx_index = field_ct::conditional_assign(
        found_link_in_rollup, matched_tx_index, -1); // `-1` means "no match found" (since 0 is a valid match value).

    // start_of_subchain = "no earlier txs in this tx's chain have been included in this rollup"
    const bool_ct start_of_subchain = chaining && !found_link_in_rollup;
    // middle_of_chain = "this tx is not the first tx of its chain to be included in this rollup"
    const bool_ct middle_of_chain = chaining && found_link_in_rollup;

    const bool_ct linked_commitment_exists = merkle_tree::check_membership(
        old_data_root, linked_commitment_paths[i], backward_link, byte_array_ct(linked_commitment_indices[i]));

    (start_of_subchain)
        .must_imply(linked_commitment_exists,
                    format("tx ",
                           i,
                           "'s linked commitment must exist. Membership check failed for backward_link ",
                           backward_link));

    field_ct attempting_to_propagate_output_index = field_ct::conditional_assign(
        is_propagating_prev_output1, 1, field_ct::conditional_assign(is_propagating_prev_output2, 2, 0));

    // Note: prev_allow_chain = 3 => "both outputs of prev_tx may be propagated from"
    (middle_of_chain)
        .must_imply(prev_allow_chain == attempting_to_propagate_output_index || prev_allow_chain == 3,
                    format("tx ",
                           i,
                           " is not permitted to propagate output ",
                           attempting_to_propagate_output_index,
                           " of the prev tx. prev_allow_chain = ",
                           prev_allow_chain));

    // Zeroing of public inputs:
    // If some previous tx's note commitment has been propagated by this i-th tx, then 'zero' both it and this
    // proof's nullifier.
    {
        // Flag our intention to zero certain public inputs:
        const bool_ct zero_prev_output1 = middle_of_chain && is_propagating_prev_output1;
        const bool_ct zero_prev_output2 = middle_of_chain && is_propagating_prev_output2;
        const bool_ct zero_nullifier1 = middle_of_chain && propagated_input_index == 1;
        const bool_ct zero_nullifier2 = middle_of_chain && propagated_input_index == 2;

        // Extra checks to ensure indices match up (if changes are ever made in future). Recommend keeping these checks.
        // These checks MUST come _before_ the zeroing calcs.
        nullifier1.assert_equal(propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER1],
                                format("Unexpected indexing mismatch for index ",
                                       i,
                                       " nullifier1: ",
                                       nullifier1,
                                       " propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER1]: ",
                                       propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER1]));
        nullifier2.assert_equal(propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER2],
                                format("Unexpected indexing mismatch for index ",
                                       i,
                                       " nullifier2: ",
                                       nullifier2,
                                       " propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER2]: ",
                                       propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER2]));

        // Conditionally zero this tx's nullifiers (these variables will ultimately be inserted into the nullifier
        // tree):
        nullifier1 *= !zero_nullifier1;
        nullifier2 *= !zero_nullifier2;

        // Also zero the corresponding nullifiers in the propagated_tx_public_inputs vector, since these will be
        // submitted on-chain for public_inputs_hash reconciliation.
        propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER1] = nullifier1;
        propagated_tx_public_inputs[i][PropagatedInnerProofFields::NULLIFIER2] = nullifier2;

        // In a circuit, we can't dynamically access a previous tx at some unknown-in-advance index.
        // Instead, we'll need to loop through all previous txs and apply the same operations to each (only actually
        // editing <=1 of them).
        field_ct field_j(0);
        for (size_t j = 0; j < i; j++) {
            if (j == i - 1) {
                // Extra checks to ensure indices match up.
                // Recommend keeping these checks, unless there's a neat test that can be written.
                // These checks MUST come _before_ the zeroing calcs.
                // Values of `new_data_values[k]` for k < i - 1 have already been checked (and then mutated) during the
                // previous call of this function for the (i-1)th tx, so don't need to be (and cannot be) checked again.
                new_data_values[2 * j].assert_equal(
                    propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT1],
                    format("Unexpected indexing mismatch for index ",
                           j,
                           " new_data_values[2 * j]: ",
                           new_data_values[2 * j],
                           " propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT1]: ",
                           propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT1]));

                new_data_values[2 * j + 1].assert_equal(
                    propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT2],
                    format("Unexpected indexing mismatch for index ",
                           j,
                           " new_data_values[2 * j + 1]: ",
                           new_data_values[2 * j + 1],
                           " propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT2]: ",
                           propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT2]));
            }

            // Conditionally zero certain commitment values in the vector which will be inserted into the data tree:
            new_data_values[2 * j] *= !(zero_prev_output1 && (field_j == matched_tx_index));
            new_data_values[2 * j + 1] *= !(zero_prev_output2 && (field_j == matched_tx_index));

            // Also zero the corresponding commitments in the propagated_tx_public_inputs vector, since these will be
            // submitted on-chain for public_inputs_hash reconciliation.
            propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT1] = new_data_values[2 * j];
            propagated_tx_public_inputs[j][PropagatedInnerProofFields::NOTE_COMMITMENT2] = new_data_values[2 * j + 1];

            field_j += 1;
        }
    }

    return { nullifier1, nullifier2 };
}

/**
 * Accumulate tx fees from each inner proof depending on the type of proof.
 */
void accumulate_tx_fees(Composer& composer,
                        std::vector<suint_ct>& total_tx_fees,
                        field_ct const& proof_id,
                        field_ct const& asset_id,
                        suint_ct const& tx_fee,
                        std::vector<field_ct> const& asset_ids,
                        field_ct const& num_asset_ids,
                        bool_ct const& is_real)
{
    const auto is_account = proof_id == field_ct(ProofIds::ACCOUNT);

    // Accumulate tx_fee for each asset_id. Note that tx_fee = 0 for padding proofs.
    field_ct num_matched(&composer, 0);
    for (uint32_t k = 0; k < NUM_ASSETS; k++) {
        auto is_asset_id_real = uint32_ct(k) < num_asset_ids;

        const auto matches = asset_id == asset_ids[k] && is_asset_id_real;
        num_matched += matches;

        total_tx_fees[k] = total_tx_fees[k] + tx_fee * static_cast<suint_ct>(matches);
    }

    // Assert this proof matched a single asset_id or it must be a padding proof.
    auto is_valid_asset_id = !is_real || num_matched == 1 || is_account;
    is_valid_asset_id.assert_equal(true,
                                   format("proof asset id matched ", uint64_t(num_matched.get_value()), " times"));
}

recursion_output<bn254> rollup_circuit(Composer& composer,
                                       rollup_tx const& rollup,
                                       std::vector<std::shared_ptr<waffle::verification_key>> const& verification_keys,
                                       size_t max_num_txs)
{
    // Compute a constant witness of the next power of 2 > max_num_txs.
    const auto floor_rollup_size = 1UL << numeric::get_msb(max_num_txs);
    const auto rollup_size_pow2_ = floor_rollup_size << (max_num_txs != floor_rollup_size);
    const auto rollup_size_pow2 = field_ct(witness_ct(&composer, rollup_size_pow2_));
    rollup_size_pow2.assert_equal(rollup_size_pow2_, format("rollup size != ", rollup_size_pow2_));

    // Witnesses from rollup_tx data.
    const auto rollup_id = field_ct(witness_ct(&composer, rollup.rollup_id));
    const auto num_txs = uint32_ct(witness_ct(&composer, rollup.num_txs));
    field_ct(num_txs).create_range_constraint(MAX_TXS_BIT_LENGTH);
    const auto data_start_index =
        suint_ct(witness_ct(&composer, rollup.data_start_index), DATA_TREE_DEPTH, "data_start_index");
    const auto old_data_root = field_ct(witness_ct(&composer, rollup.old_data_root));
    const auto new_data_root = field_ct(witness_ct(&composer, rollup.new_data_root));
    const auto old_data_path = create_witness_hash_path(composer, rollup.old_data_path);

    const auto linked_commitment_paths =
        map(rollup.linked_commitment_paths, [&](auto& p) { return create_witness_hash_path(composer, p); });
    const auto linked_commitment_indices =
        map(rollup.linked_commitment_indices, [&](auto& i) { return field_ct(witness_ct(&composer, i)); });

    const auto old_null_root = field_ct(witness_ct(&composer, rollup.old_null_root));
    const auto new_null_roots = map(rollup.new_null_roots, [&](auto& r) { return field_ct(witness_ct(&composer, r)); });
    const auto old_null_paths =
        map(rollup.old_null_paths, [&](auto& p) { return create_witness_hash_path(composer, p); });

    const auto data_roots_root = field_ct(witness_ct(&composer, rollup.data_roots_root));
    const auto data_roots_paths =
        map(rollup.data_roots_paths, [&](auto& p) { return create_witness_hash_path(composer, p); });
    const auto data_root_indicies =
        map(rollup.data_roots_indicies, [&](auto& i) { return field_ct(witness_ct(&composer, i)); });

    const auto new_defi_root = field_ct(witness_ct(&composer, rollup.new_defi_root));
    const auto num_defi_interactions = field_ct(witness_ct(&composer, rollup.num_defi_interactions));
    auto bridge_ids = map(rollup.bridge_ids, [&](auto& bid) {
        return suint_ct(witness_ct(&composer, bid), DEFI_BRIDGE_ID_BIT_LENGTH, "bridge_id");
    });
    const auto recursive_manifest = Composer::create_unrolled_manifest(verification_keys[0]->num_public_inputs);

    const auto num_asset_ids = field_ct(witness_ct(&composer, rollup.num_asset_ids));
    auto asset_ids = map(rollup.asset_ids, [&](auto& aid) { return field_ct(witness_ct(&composer, aid)); });
    // Zero any input bridge_ids that are outside scope, and check in scope bridge_ids are not zero.
    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto in_scope = uint32_ct(i) < num_defi_interactions;
        bridge_ids[i] *= in_scope;
        auto valid = !in_scope || bridge_ids[i] != 0;
        valid.assert_equal(true, "bridge_id out of scope");
    }

    // Input asset_ids that are outside scope are set to 2^{30} (NUM_MAX_ASSETS).
    for (uint32_t i = 0; i < NUM_ASSETS; i++) {
        auto in_scope = uint32_ct(i) < num_asset_ids;
        asset_ids[i] = asset_ids[i].madd(field_ct(in_scope), field_ct(MAX_NUM_ASSETS) * !in_scope);
        auto valid = !in_scope || asset_ids[i] != field_ct(MAX_NUM_ASSETS);
        valid.assert_equal(true, "asset_id out of scope");
    }

    // Loop accumulators.
    auto new_data_values = std::vector<field_ct>();
    auto new_null_indicies = std::vector<field_ct>();
    recursion_output<bn254> recursion_output;
    // Public inputs of the inner txs which will be 'made public' ('propagated' - not to be confused with chained txs
    // propagation) by this rollup circuit:
    std::vector<std::vector<field_ct>> propagated_tx_public_inputs;
    // All public inputs of the inner txs (including public inputs which will not be made public by this rollup
    // circuit):
    std::vector<std::vector<field_ct>> all_tx_public_inputs;
    auto total_tx_fees = std::vector<suint_ct>(NUM_ASSETS, suint_ct::create_constant_witness(&composer, 0));
    std::vector<suint_ct> defi_deposit_sums(NUM_BRIDGE_CALLS_PER_BLOCK,
                                            suint_ct::create_constant_witness(&composer, 0));

    for (size_t i = 0; i < max_num_txs; ++i) {
        // Pick verification key and check it's permitted.
        auto proof_id_u32 = from_buffer<uint32_t>(rollup.txs[i], InnerProofOffsets::PROOF_ID + 28);
        auto recursive_verification_key =
            plonk::stdlib::recursion::verification_key<bn254>::from_witness(&composer, verification_keys[proof_id_u32]);
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
        for (size_t j = 0; j < InnerProofFields::NUM_FIELDS; ++j) {
            public_inputs[j] *= is_real;
        }

        auto tx_fee = process_defi_deposit(
            composer, rollup_id, public_inputs, bridge_ids, defi_deposit_sums, num_defi_interactions);

        process_claims(public_inputs, new_defi_root);

        // Ordering matters. This `push_back` must happen after any mutations to `public_inputs` in the
        // `process_defi_deposit()` & `process_claims()` functions, but before `process_chained_txs`.
        propagated_tx_public_inputs.push_back(slice(public_inputs, 0, PropagatedInnerProofFields::NUM_FIELDS));

        field_ct nullifier1, nullifier2;
        std::tie(nullifier1, nullifier2) =
            process_chained_txs(i,
                                is_real,
                                public_inputs,
                                all_tx_public_inputs,
                                propagated_tx_public_inputs,
                                new_data_values,
                                old_data_root,
                                linked_commitment_paths,
                                linked_commitment_indices); // this function might 'zero' some elements of
                                                            // new_data_values and propagated_tx_public_inputs

        // Add this proof's data values to the list.
        new_data_values.push_back(public_inputs[InnerProofFields::NOTE_COMMITMENT1]);
        new_data_values.push_back(public_inputs[InnerProofFields::NOTE_COMMITMENT2]);

        // Add input note nullifiers to the list.
        new_null_indicies.push_back(nullifier1);
        new_null_indicies.push_back(nullifier2);

        // Check this proof's data root exists in the data root tree (unless a padding entry).
        auto data_root = public_inputs[InnerProofFields::MERKLE_ROOT];
        bool_ct data_root_exists =
            data_root != 0 &&
            check_membership(data_roots_root, data_roots_paths[i], data_root, byte_array_ct(data_root_indicies[i]));
        is_real.assert_equal(data_root_exists, format("data_root_for_proof_", i));

        // Accumulate tx fee.
        auto proof_id = public_inputs[InnerProofFields::PROOF_ID];
        auto asset_id = public_inputs[InnerProofFields::TX_FEE_ASSET_ID];
        accumulate_tx_fees(composer, total_tx_fees, proof_id, asset_id, tx_fee, asset_ids, num_asset_ids, is_real);

        all_tx_public_inputs.push_back(public_inputs);
    }

    new_data_values.resize(rollup_size_pow2_ * 2, fr(0));
    batch_update_membership(new_data_root, old_data_root, old_data_path, new_data_values, data_start_index.value);

    auto new_null_root =
        check_nullifiers_inserted(composer, new_null_roots, old_null_paths, num_txs, old_null_root, new_null_indicies);

    // Compute hash of the tx public inputs. Used to reduce number of public inputs published in root rollup.
    auto sha_input = flatten(propagated_tx_public_inputs);
    sha_input.resize(rollup_size_pow2_ * PropagatedInnerProofFields::NUM_FIELDS, field_ct(0));
    auto hash_output = stdlib::sha256_to_field(packed_byte_array_ct::from_field_element_vector(sha_input));

    // Publish public inputs.
    rollup_id.set_public();
    rollup_size_pow2.set_public();
    data_start_index.set_public();
    old_data_root.set_public();
    new_data_root.set_public();
    old_null_root.set_public();
    new_null_root.set_public();
    data_roots_root.set_public();
    public_witness_ct(&composer, rollup.data_roots_root);
    add_zero_public_inputs(composer, 1); // old_defi_root
    new_defi_root.set_public();
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        bridge_ids[i].set_public();
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        defi_deposit_sums[i].set_public();
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        asset_ids[i].set_public();
    }
    for (auto total_tx_fee : total_tx_fees) {
        total_tx_fee.set_public();
    }
    hash_output.set_public();
    for (auto& tx : propagated_tx_public_inputs) {
        for (auto& public_input : tx) {
            public_input.set_public();
        }
    }
    // Add tx padding public inputs.
    add_zero_public_inputs(composer, (rollup_size_pow2_ - max_num_txs) * PropagatedInnerProofFields::NUM_FIELDS);

    // Publish pairing coords limbs as public inputs.
    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
