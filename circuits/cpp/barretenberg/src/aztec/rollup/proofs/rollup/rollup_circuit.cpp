#include "rollup_circuit.hpp"
#include "./rollup_proof_data.hpp"
#include "../../constants.hpp"
#include "../inner_proof_data.hpp"
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

void propagate_inner_proof_public_inputs(std::vector<field_ct> const& public_inputs)
{
    for (size_t i = 0; i < PropagatedInnerProofFields::NUM_FIELDS; ++i) {
        public_inputs[i].set_public();
    }
}

void add_zero_public_input(Composer& composer)
{
    auto zero = field_ct(witness_ct(&composer, 0));
    zero.assert_is_zero();
    zero.set_public();
}

void add_tx_padding_public_inputs(Composer& composer)
{
    for (size_t i = 0; i < PropagatedInnerProofFields::NUM_FIELDS; ++i) {
        add_zero_public_input(composer);
    }
}

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
                          std::vector<field_ct> const& bridge_ids,
                          std::vector<field_ct>& defi_deposit_sums,
                          field_ct const& num_defi_interactions)
{
    field_ct defi_interaction_nonce = (rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK);

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
    auto is_valid_bridge_id = num_matched == 1 || !is_defi_deposit;
    is_valid_bridge_id.assert_equal(true,
                                    format("proof bridge id matched ", uint64_t(num_matched.get_value()), " times."));

    // Compute claim fee which to be added to the claim note.
    const auto tx_fee = public_inputs[InnerProofFields::TX_FEE];
    const auto defi_deposit_fee = tx_fee.slice(TX_FEE_BIT_LENGTH, 1);
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
    // For claim proofs, defi root is output in field named INPUT_OWNER.
    const auto defi_root = public_inputs[InnerProofFields::INPUT_OWNER];

    auto valid = defi_root == new_defi_root || !is_claim;
    valid.assert_equal(true, format("claim proof has unmatched defi root"));
}

/**
 * Accumulate tx fees from each inner proof depending on the type of proof.
 */
void accumulate_tx_fees(Composer& composer,
                        std::vector<field_ct>& total_tx_fees,
                        field_ct const& proof_id,
                        field_ct const& asset_id,
                        field_ct const& tx_fee,
                        std::vector<field_ct> const& asset_ids,
                        field_ct const& num_asset_ids,
                        bool_ct const& is_real)
{
    const auto is_js_tx = proof_id == field_ct(ProofIds::JOIN_SPLIT);
    const auto is_account = proof_id == field_ct(ProofIds::ACCOUNT);
    const auto is_defi_deposit = proof_id == field_ct(ProofIds::DEFI_DEPOSIT);
    const auto is_defi_claim = proof_id == field_ct(ProofIds::DEFI_CLAIM);
    const auto is_defi = (is_defi_deposit || is_defi_claim);

    // asset_id = bridge_id for a defi deposit proof
    const uint8_t input_asset_id_lsb = (DEFI_BRIDGE_ADDRESS_BIT_LENGTH + DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN);
    const uint8_t input_asset_id_msb = input_asset_id_lsb + DEFI_BRIDGE_INPUT_ASSET_ID_LEN - 1;
    const field_ct defi_input_asset_id = asset_id.slice(input_asset_id_msb, input_asset_id_lsb);

    // combined asset id of an inner proof
    const auto input_asset_id = (defi_input_asset_id * is_defi + asset_id * is_js_tx);

    // Accumulate tx_fee for each asset_id. Note that tx_fee = 0 for padding proofs.
    field_ct num_matched(&composer, 0);
    for (uint32_t k = 0; k < NUM_ASSETS; k++) {
        auto is_asset_id_real = uint32_ct(k) < num_asset_ids;

        const auto matches = input_asset_id == asset_ids[k] && is_asset_id_real;
        num_matched += matches;

        total_tx_fees[k] += tx_fee * matches;
    }

    // Assert this proof matched a single asset_id or it must be a padding proof.
    auto is_valid_asset_id = !is_real || num_matched == 1 || is_account;
    is_valid_asset_id.assert_equal(true,
                                   format("proof asset id matched ", uint64_t(num_matched.get_value()), " times."));
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
    const auto data_start_index = field_ct(witness_ct(&composer, rollup.data_start_index));
    const auto old_data_root = field_ct(witness_ct(&composer, rollup.old_data_root));
    const auto new_data_root = field_ct(witness_ct(&composer, rollup.new_data_root));
    const auto old_data_path = create_witness_hash_path(composer, rollup.old_data_path);
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
    auto bridge_ids = map(rollup.bridge_ids, [&](auto& bid) { return field_ct(witness_ct(&composer, bid)); });
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
    std::vector<std::vector<field_ct>> inner_public_inputs;
    auto total_tx_fees = std::vector<field_ct>(NUM_ASSETS, field_ct(witness_ct::create_constant_witness(&composer, 0)));
    std::vector<field_ct> defi_deposit_sums(NUM_BRIDGE_CALLS_PER_BLOCK,
                                            field_ct(witness_ct::create_constant_witness(&composer, 0)));

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
        for (size_t i = 0; i < InnerProofFields::NUM_FIELDS; ++i) {
            public_inputs[i] *= is_real;
        }

        auto tx_fee = process_defi_deposit(
            composer, rollup_id, public_inputs, bridge_ids, defi_deposit_sums, num_defi_interactions);
        process_claims(public_inputs, new_defi_root);

        // Add the proofs data values to the list.
        new_data_values.push_back(public_inputs[InnerProofFields::NOTE_COMMITMENT1]);
        new_data_values.push_back(public_inputs[InnerProofFields::NOTE_COMMITMENT2]);

        // Add nullifiers to the list.
        new_null_indicies.push_back(public_inputs[InnerProofFields::NULLIFIER1]);
        new_null_indicies.push_back(public_inputs[InnerProofFields::NULLIFIER2]);

        // Check this proofs data root exists in the data root tree (unless a padding entry).
        auto data_root = public_inputs[InnerProofFields::MERKLE_ROOT];
        bool_ct valid =
            data_root != 0 &&
            check_membership(data_roots_root, data_roots_paths[i], data_root, byte_array_ct(data_root_indicies[i]));
        is_real.assert_equal(valid, format("data_root_for_proof_", i));

        // Accumulate tx fee.
        auto proof_id = public_inputs[InnerProofFields::PROOF_ID];
        auto asset_id = public_inputs[InnerProofFields::ASSET_ID];
        accumulate_tx_fees(composer, total_tx_fees, proof_id, asset_id, tx_fee, asset_ids, num_asset_ids, is_real);

        inner_public_inputs.push_back(public_inputs);
    }

    new_data_values.resize(rollup_size_pow2_ * 2, fr(0));
    batch_update_membership(new_data_root, old_data_root, old_data_path, new_data_values, data_start_index);

    auto new_null_root =
        check_nullifiers_inserted(composer, new_null_roots, old_null_paths, num_txs, old_null_root, new_null_indicies);

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
    add_zero_public_input(composer); // old_defi_root
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

    /**
     * Compress public inputs
     *
     * Verifier smart contract requires 150 gas per public input processed.
     * Much cheaper to SHA256 hash the public inputs down into a single 32-byte block
     */
    std::vector<field_ct> public_input_hash_inputs;
    for (auto& inner : inner_public_inputs) {
        for (size_t i = 0; i < PropagatedInnerProofFields::NUM_FIELDS; ++i) {
            public_input_hash_inputs.push_back(inner[i]);
        }
    }
    packed_byte_array_ct input_msg = packed_byte_array_ct::from_field_element_vector(public_input_hash_inputs);
    auto hash_output = stdlib::sha256<Composer>(input_msg);
    std::vector<field_ct> inner_inputs_hash = hash_output.to_unverified_byte_slices(16);
    // convert the hash output to a field element (i.e. reduce mod p)
    field_ct hash_output_reduced = inner_inputs_hash[1] + inner_inputs_hash[0] * (uint256_t(1) << 128);
    composer.set_public_input(hash_output_reduced.normalize().witness_index);
    for (auto& inner : inner_public_inputs) {
        propagate_inner_proof_public_inputs(inner);
    }

    for (size_t i = max_num_txs; i < rollup_size_pow2_; ++i) {
        add_tx_padding_public_inputs(composer);
    }

    // Publish pairing coords limbs as public inputs.
    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
