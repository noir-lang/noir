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

using namespace plonk::stdlib::types;
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
                          index.decompose_into_bits(NULL_TREE_DEPTH),
                          format(__FUNCTION__, "_", i));

        latest_null_root = new_null_roots[i];
    }

    return latest_null_root;
}

/**
 * Processes a defi deposit proof.
 * - We only process join split proofs with a proof_id == ProofIds::DEFI_DEPOSIT (otherwise noop).
 * - Ensure that the bridge_call_data matches one within the of set of bridge_call_datas.
 * - Accumulate the deposit value in relevant defi_deposit_sums slot. These later become public inputs.
 * - Modify the claim note commitment (output_note_1 commitment) to add the relevant interaction nonce to it.
 */
auto process_defi_deposit(Composer& composer,
                          field_ct const& rollup_id,
                          std::vector<field_ct>& public_inputs,
                          std::vector<suint_ct> const& bridge_call_datas,
                          std::vector<suint_ct>& defi_deposit_sums,
                          field_ct const& num_defi_interactions)
{
    field_ct defi_interaction_nonce = (rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK);

    const auto proof_id = public_inputs[InnerProofFields::PROOF_ID];
    const suint_ct bridge_call_data(
        public_inputs[InnerProofFields::BRIDGE_CALL_DATA], DEFI_BRIDGE_CALL_DATA_BIT_LENGTH, "bridge_call_data");
    const suint_ct deposit_value(
        public_inputs[InnerProofFields::DEFI_DEPOSIT_VALUE], DEFI_DEPOSIT_VALUE_BIT_LENGTH, "defi_deposit");
    const auto is_defi_deposit = proof_id == field_ct(ProofIds::DEFI_DEPOSIT);

    /**
     * There is one defi_interaction_nonce for each interaction ('bridge call').
     * The defi deposit being processed by this function will belong to one of these bridge calls
     * (based on the bridge_call_data) - say it's the k-th bridge call of this rollup.
     * Then the defi_interaction_nonce = rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK + k.
     */
    field_ct note_defi_interaction_nonce = defi_interaction_nonce;
    field_ct num_matched(&composer, 0);

    for (uint32_t k = 0; k < NUM_BRIDGE_CALLS_PER_BLOCK; k++) {
        auto is_real = uint32_ct(k) < num_defi_interactions;

        const auto matches = bridge_call_data == bridge_call_datas[k] && is_real;
        num_matched += matches;

        defi_deposit_sums[k] += deposit_value * is_defi_deposit * matches;
        note_defi_interaction_nonce += (field_ct(&composer, k) * matches);
    }
    note_defi_interaction_nonce *= is_defi_deposit;

    // Assert this proof matched a single bridge_call_data.
    auto is_valid_bridge_call_data = num_matched == 1 || !is_defi_deposit;
    is_valid_bridge_call_data.assert_equal(
        true, format("proof bridge call data matched ", uint64_t(num_matched.get_value()), " times"));

    // Compute claim fee which to be added to the claim note.
    const suint_ct tx_fee(public_inputs[InnerProofFields::TX_FEE], TX_FEE_BIT_LENGTH, "tx_fee");
    const suint_ct defi_deposit_fee = tx_fee / 2;
    const auto claim_fee = (tx_fee - defi_deposit_fee) * is_defi_deposit;
    const auto net_tx_fee = suint_ct::conditional_assign(is_defi_deposit, defi_deposit_fee, tx_fee);

    // Complete the claim note output to mix in the interaction nonce and the claim fee.
    auto note_commitment1 = public_inputs[InnerProofFields::NOTE_COMMITMENT1];
    auto claim_note_commitment =
        notes::circuit::claim::complete_partial_commitment(note_commitment1, note_defi_interaction_nonce, claim_fee);

    public_inputs[InnerProofFields::NOTE_COMMITMENT1] =
        field_ct::conditional_assign(is_defi_deposit, claim_note_commitment, note_commitment1);

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
 * @param prev_txs_public_inputs is required to extract the allow_chain public input from each tx
 * @returns the (possibly zeroed) nullifiers of this tx
 */
void process_chained_txs(size_t const& i,
                         bool_ct const& is_tx_real,
                         std::vector<field_ct> const& public_inputs,
                         std::vector<std::vector<field_ct>> const& prev_txs_public_inputs,
                         field_ct const& old_data_root,
                         std::vector<merkle_tree::hash_path> const& linked_commitment_paths,
                         std::vector<field_ct> const& linked_commitment_indices)
{
    const field_ct backward_link = field_ct(public_inputs[InnerProofFields::BACKWARD_LINK]);

    const bool_ct chaining = backward_link != 0;

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
        const auto prev_public_inputs = prev_txs_public_inputs[j];
        const field_ct prev_note_commitment1 = prev_public_inputs[InnerProofFields::NOTE_COMMITMENT1];
        const field_ct prev_note_commitment2 = prev_public_inputs[InnerProofFields::NOTE_COMMITMENT2];
        const field_ct temp_prev_allow_chain = prev_public_inputs[InnerProofFields::ALLOW_CHAIN];

        const bool_ct temp_is_propagating_prev_output1 =
            (backward_link == prev_note_commitment1) &&
            is_tx_real; // Inclusion of `is_tx_real` prevents `0 == 0` from passing, for padded txs (which have a 0
                        // prev_note_commitment).
        const bool_ct temp_is_propagating_prev_output2 = (backward_link == prev_note_commitment2) && is_tx_real;
        const bool_ct found_link_in_loop = temp_is_propagating_prev_output1 || temp_is_propagating_prev_output2;

        // If we've found a tx which matches this tx's backward_link, then write data to the higher-scoped
        // variables:
        // Note: we don't need to try to prevent multiple matches (and hence multiple writes to the
        // higher-scoped variables) in this loop. Multiple matches would mean there are >1 txs with the same output
        // commitment, which is a bigger problem that will be caught when updating the nullifier tree (duplicate
        // output commitments would share the same input_nullifier).
        // Notice: once found, the below values remain unchanged through future iterations:
        found_link_in_rollup |= found_link_in_loop;
        prev_allow_chain = field_ct::conditional_assign(found_link_in_loop, temp_prev_allow_chain, prev_allow_chain);
        is_propagating_prev_output1 = bool_ct(field_ct::conditional_assign(
            found_link_in_loop, temp_is_propagating_prev_output1, is_propagating_prev_output1));
        is_propagating_prev_output2 = bool_ct(field_ct::conditional_assign(
            found_link_in_loop, temp_is_propagating_prev_output2, is_propagating_prev_output2));
    }

    // start_of_subchain = "no earlier txs in this tx's chain have been included in this rollup"
    const bool_ct start_of_subchain = chaining && !found_link_in_rollup;
    // middle_of_chain = "this tx is not the first tx of its chain to be included in this rollup"
    const bool_ct middle_of_chain = chaining && found_link_in_rollup;

    const bool_ct linked_commitment_exists =
        merkle_tree::check_membership(old_data_root,
                                      linked_commitment_paths[i],
                                      backward_link,
                                      linked_commitment_indices[i].decompose_into_bits(DATA_TREE_DEPTH));

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

        total_tx_fees[k] += tx_fee * static_cast<suint_ct>(matches);
    }

    // Assert this proof matched either 0 or 1 assets
    auto is_valid_asset_id = !is_real || num_matched == 0 || num_matched == 1 || is_account;
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
    auto bridge_call_datas = map(rollup.bridge_call_datas, [&](auto& bid) {
        return suint_ct(witness_ct(&composer, bid), DEFI_BRIDGE_CALL_DATA_BIT_LENGTH, "bridge_call_data");
    });
    const auto recursive_manifest = Composer::create_unrolled_manifest(verification_keys[0]->num_public_inputs);

    const auto num_asset_ids = field_ct(witness_ct(&composer, rollup.num_asset_ids));
    auto asset_ids = map(rollup.asset_ids, [&](auto& aid) { return field_ct(witness_ct(&composer, aid)); });
    // Zero any input bridge_call_datas that are outside scope, and check in scope bridge_call_datas are not zero.
    for (uint32_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        auto in_scope = uint32_ct(i) < num_defi_interactions;
        bridge_call_datas[i] *= in_scope;
        auto valid = !in_scope || bridge_call_datas[i] != 0;
        valid.assert_equal(true, "bridge_call_data out of scope");
    }

    // Input asset_ids that are outside scope are set to 2^{30} (NUM_MAX_ASSETS).
    for (uint32_t i = 0; i < NUM_ASSETS; i++) {
        auto in_scope = uint32_ct(i) < num_asset_ids;
        asset_ids[i] = field_ct::conditional_assign(in_scope, asset_ids[i], field_ct(MAX_NUM_ASSETS));
        auto valid = !in_scope || asset_ids[i] != field_ct(MAX_NUM_ASSETS);
        valid.assert_equal(true, "asset_id out of scope");
    }

    // Loop accumulators.
    auto new_data_values = std::vector<field_ct>();
    auto new_null_indicies = std::vector<field_ct>();
    recursion_output<bn254> recursion_output;
    // Public inputs of the inner txs which will be 'made public' ('propagated' - not to be confused with chained
    // txs propagation) by this rollup circuit:
    std::vector<std::vector<field_ct>> propagated_tx_public_inputs;
    // All public inputs of the inner txs (including public inputs which will not be made public by this rollup
    // circuit):
    std::vector<std::vector<field_ct>> prev_txs_public_inputs;
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
        recursion_output = verify_proof<bn254, recursive_inner_verifier_settings>(&composer,
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
            composer, rollup_id, public_inputs, bridge_call_datas, defi_deposit_sums, num_defi_interactions);

        process_claims(public_inputs, new_defi_root);

        // Ordering matters. This `push_back` must happen after any mutations to `public_inputs` in the
        // `process_defi_deposit()` & `process_claims()` functions, but before `process_chained_txs`.
        propagated_tx_public_inputs.push_back(slice(public_inputs, 0, PropagatedInnerProofFields::NUM_FIELDS));

        process_chained_txs(i,
                            is_real,
                            public_inputs,
                            prev_txs_public_inputs,
                            old_data_root,
                            linked_commitment_paths,
                            linked_commitment_indices);

        // Add this proof's data values to the list.
        new_data_values.push_back(public_inputs[InnerProofFields::NOTE_COMMITMENT1]);
        new_data_values.push_back(public_inputs[InnerProofFields::NOTE_COMMITMENT2]);

        // Add input note nullifiers to the list.
        new_null_indicies.push_back(public_inputs[InnerProofFields::NULLIFIER1]);
        new_null_indicies.push_back(public_inputs[InnerProofFields::NULLIFIER2]);

        // Check this proof's data root exists in the data root tree (unless a padding entry).
        auto data_root = public_inputs[InnerProofFields::MERKLE_ROOT];
        bool_ct data_root_exists =
            data_root != 0 && check_membership(data_roots_root,
                                               data_roots_paths[i],
                                               data_root,
                                               data_root_indicies[i].decompose_into_bits(ROOT_TREE_DEPTH));
        is_real.assert_equal(data_root_exists, format("data_root_for_proof_", i));

        // Accumulate tx fee.
        auto proof_id = public_inputs[InnerProofFields::PROOF_ID];
        auto asset_id = public_inputs[InnerProofFields::TX_FEE_ASSET_ID];
        accumulate_tx_fees(composer, total_tx_fees, proof_id, asset_id, tx_fee, asset_ids, num_asset_ids, is_real);

        prev_txs_public_inputs.push_back(public_inputs);
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
        bridge_call_datas[i].set_public();
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
