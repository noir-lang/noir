#include "join_split_circuit.hpp"
#include "../../constants.hpp"
#include "../notes/circuit/value/compute_nullifier.hpp"
#include "../notes/circuit/value/value_note.hpp"
#include "../notes/circuit/account/account_note.hpp"
#include "../notes/circuit/claim/claim_note.hpp"
#include "verify_signature.hpp"
#include <stdlib/merkle_tree/membership.hpp>
#include <stdlib/primitives/field/pow.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk;
using namespace notes::circuit;
using namespace plonk::stdlib::merkle_tree;

/**
 * Check that the input note data, follows the given hash paths, to the publically given merkle root.
 * The note does not need to exist in the tree if it's not real, or if it's consumed (i.e. propagated = input).
 * Return the nullifier for the input note. If the input note is consumed, the nullifier becomes 0.
 */
field_ct process_input_note(field_ct const& account_private_key,
                            field_ct const& merkle_root,
                            merkle_tree::hash_path const& hash_path,
                            suint_ct const& index,
                            value::value_note const& note,
                            bool_ct is_propagated,
                            bool_ct is_note_in_use)
{
    const bool_ct valid_value = note.value == 0 || is_note_in_use;
    valid_value.assert_equal(true, "padding note non zero");

    const bool_ct exists = merkle_tree::check_membership(
        merkle_root, hash_path, note.commitment, index.value.decompose_into_bits(DATA_TREE_DEPTH));
    const bool_ct valid = exists || is_propagated || !is_note_in_use;
    valid.assert_equal(true, "input note not a member");

    return compute_nullifier(note.commitment, account_private_key, is_note_in_use);
}

join_split_outputs join_split_circuit_component(join_split_inputs const& inputs)
{
    const bool_ct is_deposit = inputs.proof_id == field_ct(ProofIds::DEPOSIT);
    const bool_ct is_withdraw = inputs.proof_id == field_ct(ProofIds::WITHDRAW);
    const bool_ct is_send = inputs.proof_id == field_ct(ProofIds::SEND);
    const bool_ct is_defi_deposit = inputs.proof_id == field_ct(ProofIds::DEFI_DEPOSIT);
    const bool_ct not_defi_deposit = !is_defi_deposit;
    const bool_ct is_public_tx = is_deposit || is_withdraw;

    const auto public_asset_id = inputs.asset_id * is_public_tx;
    const auto public_input = inputs.public_value * is_deposit;
    const auto public_output = inputs.public_value * is_withdraw;

    const auto input_note_1 = value::value_note(inputs.input_note1);
    const auto input_note_2 = value::value_note(inputs.input_note2);
    const auto output_note_1 = value::value_note(inputs.output_note1);
    const auto output_note_2 = value::value_note(inputs.output_note2);

    const auto partial_claim_note = claim::partial_claim_note(
        inputs.partial_claim_note, inputs.input_note1.owner, inputs.input_note1.account_required);

    const auto output_note_1_commitment =
        field_ct::conditional_assign(is_defi_deposit, partial_claim_note.partial_commitment, output_note_1.commitment);

    const auto defi_deposit_value = inputs.partial_claim_note.deposit_value * is_defi_deposit;
    const auto bridge_id = partial_claim_note.bridge_id * is_defi_deposit;

    const bool_ct no_input_notes = inputs.num_input_notes == 0;
    const bool_ct one_input_note = inputs.num_input_notes == 1;
    const bool_ct two_input_notes = inputs.num_input_notes == 2;
    (no_input_notes || one_input_note || two_input_notes).assert_equal(true, "invalid num_input_notes");
    const bool_ct input_note_1_in_use = one_input_note || two_input_notes;
    const bool_ct& input_note_2_in_use = two_input_notes;

    const bool_ct equal_input_asset_ids = input_note_1.asset_id == input_note_2.asset_id;
    const bool_ct different_input_asset_ids = !equal_input_asset_ids;

    const auto& input_note_1_value = input_note_1.value;
    auto input_note_2_value = input_note_2.value;
    const auto output_note_1_value = output_note_1.value * not_defi_deposit;
    const auto& output_note_2_value = output_note_2.value;

    // Check public value and owner are not zero for deposit and withdraw, otherwise they must be zero.
    (is_public_tx == inputs.public_value.is_zero()).assert_equal(false, "public value invalid");
    (is_public_tx == inputs.public_owner.is_zero()).assert_equal(false, "public owner invalid");

    // Constrain the proof id.
    inputs.proof_id.assert_is_in_set({ field_ct(ProofIds::DEPOSIT),
                                       field_ct(ProofIds::WITHDRAW),
                                       field_ct(ProofIds::SEND),
                                       field_ct(ProofIds::DEFI_DEPOSIT) },
                                     "invalid proof id");

    // Check we're not joining the same input note.
    input_note_1.commitment.assert_not_equal(input_note_2.commitment, "joining same note");

    no_input_notes.must_imply(is_deposit, "can only deposit");

    const bool_ct asset_ids_match = inputs.asset_id == input_note_1.asset_id &&
                                    inputs.asset_id == output_note_1.asset_id &&
                                    inputs.asset_id == output_note_2.asset_id;
    (asset_ids_match).assert_equal(true, "asset ids don't match");

    (input_note_2_in_use && (is_deposit || is_send || is_withdraw))
        .must_imply(equal_input_asset_ids, "input asset ids must match unless defi-depositing");

    // defi-deposit checks
    {
        // Prevent deposits of 0 into the defi bridge (simply because it's illogical).
        // This check is mirrored in the claim_circuit, so if you ever remove this check, you _must_ remove it
        // from there as well, to prevent user funds from becoming 'unclaimable'.
        is_defi_deposit.must_imply(defi_deposit_value != 0, "Expected a nonzero defi_deposit_value for a defi-deposit");

        (is_defi_deposit && input_note_2_in_use && different_input_asset_ids)
            .must_imply(defi_deposit_value == input_note_2_value, "all of input note 2 must be defi-deposited");

        // Check the bridge_id's data mirrors the input notes' data:
        const auto& bridge_id_data = inputs.partial_claim_note.bridge_id_data;

        is_defi_deposit.must_imply(
            bridge_id_data.input_asset_id_a == input_note_1.asset_id,
            "Expected bridge_id_data.input_asset_id_a == input_note_1.asset_id for a defi-deposit");

        // Note: the opposite of this check isn't true: input_note_2_in_use does not necessarily imply the second input
        // to the bridge will be in-use, since both input notes could be of the same asset_id (and hence will be joined
        // into the bridge's first input).
        (bridge_id_data.config.second_input_in_use)
            .must_imply(input_note_2_in_use,
                        "Expected input_note_2_in_use, given bridge_id_data.config.second_input_in_use");

        (input_note_2_in_use && different_input_asset_ids)
            .must_imply(bridge_id_data.config.second_input_in_use,
                        "Expected bridge_id_data.config.second_input_in_use, given input_note_2_in_use && "
                        "different_input_asset_ids");

        (bridge_id_data.config.second_input_in_use)
            .must_imply(bridge_id_data.input_asset_id_b == input_note_2.asset_id,
                        "Expected bridge_id_data.input_asset_id_b == input_note_2.asset_id, given "
                        "bridge_id_data.config.second_input_in_use");
    }

    // Transaction chaining.
    bool_ct note1_propagated = inputs.backward_link == input_note_1.commitment;
    bool_ct note2_propagated = inputs.backward_link == input_note_2.commitment;
    {
        // Ensure backward_link isn't some nonzero value which is unrelated to either input:
        bool_ct backward_link_in_use = inputs.backward_link != 0;
        backward_link_in_use.must_imply(note1_propagated || note2_propagated, "backward_link unrelated to inputs");

        // allow_chain must be in {0, 1, 2, 3}
        bool_ct allow_chain_1_and_2 = inputs.allow_chain == 3;
        bool_ct allow_chain_1 = inputs.allow_chain == 1 || allow_chain_1_and_2;
        bool_ct allow_chain_2 = inputs.allow_chain == 2 || allow_chain_1_and_2;

        (inputs.allow_chain == 0 || allow_chain_1 || allow_chain_2).assert_equal(true, "allow_chain out of range");

        // When allowing chaining, ensure propagation is to one's self (and not to some other user).
        point_ct self = input_note_1.owner;
        allow_chain_1.must_imply(output_note_1.owner == self, "inter-user chaining disallowed");
        allow_chain_2.must_imply(output_note_2.owner == self, "inter-user chaining disallowed");

        // Prevent chaining from a partial claim note.
        is_defi_deposit.must_imply(!allow_chain_1, "cannot chain from a partial claim note");
    }

    // For defi deposits with two input notes, don't consider second note's value in the input/output value
    // balancing equations below:
    input_note_2_value *= !(is_defi_deposit && input_note_2_in_use && different_input_asset_ids);

    // Derive tx_fee.
    const suint_ct total_in_value = public_input + input_note_1_value + input_note_2_value;
    const suint_ct total_out_value = public_output + output_note_1_value + output_note_2_value + defi_deposit_value;

    const suint_ct tx_fee =
        total_in_value.subtract(total_out_value, TX_FEE_BIT_LENGTH, "total_in_value < total_out_value");
    /**
     * Note: the above subtraction (which disallows underflow) implicitly checks that input_note_1_value >=
     * input_note_2_value in the case of a defi_deposit with two different input note asset_ids.
     * For a defi deposit with two different input note asset_ids:
     *    public_input == public_output == 0
     *    in2_value == defi_deposit_value
     *    in2_value is ignored (zeroed) from the above balancing equation.
     *    out1_value is ignored (zeroed) for defi deposits
     * So in such cases:
     *    tx_fee = (in1_value + in2_value) - (out1_value + out2_value + defi_deposit_value)
     * => tx_fee = (in1_value + 0) - (0 + out2_value + in2_value)
     * => in1_value = tx_fee + out2_value + in2_value
     * => in1_value >= in2_value
     */

    // Verify input notes have the same account public key and account_required.
    input_note_1.owner.assert_equal(input_note_2.owner, "input note owners don't match");
    input_note_1.account_required.assert_equal(input_note_2.account_required,
                                               "input note account_required don't match");

    // And thus check both input notes have the correct public key (derived from the private key) and
    // account_required.
    inputs.account_private_key.assert_is_not_zero("account private key is zero");
    auto account_public_key = group_ct::fixed_base_scalar_mul_g1<254>(inputs.account_private_key);
    account_public_key.assert_equal(input_note_1.owner, "account_private_key incorrect");
    inputs.account_required.assert_equal(input_note_1.account_required, "account_required incorrect");

    // Verify output notes creator_pubkey is either account_public_key.x or 0.
    output_note_1.creator_pubkey.assert_is_in_set({ field_ct(0), account_public_key.x },
                                                  "output note 1 creator_pubkey mismatch");
    output_note_2.creator_pubkey.assert_is_in_set({ field_ct(0), account_public_key.x },
                                                  "output note 2 creator_pubkey mismatch");

    // Signer is the account public key if account_required is false, else it's the given signing key.
    const point_ct signer =
        point_ct::conditional_assign(inputs.account_required, inputs.signing_pub_key, account_public_key);

    // Verify that the signing key account note exists if account_required == true.
    {
        const auto account_alias_hash = inputs.alias_hash;
        const auto account_note_data = account::account_note(account_alias_hash.value, account_public_key, signer);
        const bool_ct signing_key_exists =
            merkle_tree::check_membership(inputs.merkle_root,
                                          inputs.account_note_path,
                                          account_note_data.commitment,
                                          inputs.account_note_index.value.decompose_into_bits(DATA_TREE_DEPTH));
        (signing_key_exists || !inputs.account_required).assert_equal(true, "account check_membership failed");
    }

    // Verify each input note exists in the tree, and compute nullifiers.
    const field_ct nullifier1 = process_input_note(inputs.account_private_key,
                                                   inputs.merkle_root,
                                                   inputs.input_path1,
                                                   inputs.input_note1_index,
                                                   input_note_1,
                                                   note1_propagated,
                                                   input_note_1_in_use);

    const field_ct nullifier2 = process_input_note(inputs.account_private_key,
                                                   inputs.merkle_root,
                                                   inputs.input_path2,
                                                   inputs.input_note2_index,
                                                   input_note_2,
                                                   note2_propagated,
                                                   input_note_2_in_use);

    // Assert that input nullifiers in the output note commitments equal the input notes' nullifiers.
    output_note_1.input_nullifier.assert_equal(nullifier1, "output note 1 has incorrect input nullifier");
    output_note_2.input_nullifier.assert_equal(nullifier2, "output note 2 has incorrect input nullifier");
    partial_claim_note.input_nullifier.assert_equal(nullifier1 * is_defi_deposit,
                                                    "partial claim note has incorrect input nullifier");

    // Q: Why do we need to verify a signature in the circuit? Why doesn't it suffice to simply compute the public key
    // from its secret key, as a way of showing willingness to spend the notes?
    // A: By passing a signature to the circuit, the 'signing private key' doesn't need to be passed to the proof
    // construction software. This is useful for multisigs, offline signing, etc., so that the proof construction
    // software (or machine) doesn't have access to the signing private key.
    verify_signature(inputs.public_value.value,
                     inputs.public_owner,
                     public_asset_id.value,
                     output_note_1_commitment,
                     output_note_2.commitment,
                     nullifier1,
                     nullifier2,
                     signer,
                     inputs.backward_link,
                     inputs.allow_chain,
                     inputs.signature);

    return { nullifier1, nullifier2, output_note_1_commitment, output_note_2.commitment, public_asset_id,
             tx_fee,     bridge_id,  defi_deposit_value };
}

void join_split_circuit(Composer& composer, join_split_tx const& tx)
{
    join_split_inputs inputs = {
        .proof_id = witness_ct(&composer, tx.proof_id),
        .public_value = suint_ct(witness_ct(&composer, tx.public_value), NOTE_VALUE_BIT_LENGTH, "public_value"),
        .public_owner = witness_ct(&composer, tx.public_owner),
        .asset_id = suint_ct(witness_ct(&composer, tx.asset_id), ASSET_ID_BIT_LENGTH, "asset_id"),
        .num_input_notes = witness_ct(&composer, tx.num_input_notes),
        .input_note1_index = suint_ct(witness_ct(&composer, tx.input_index[0]), DATA_TREE_DEPTH, "input_index0"),
        .input_note2_index = suint_ct(witness_ct(&composer, tx.input_index[1]), DATA_TREE_DEPTH, "input_index1"),
        .input_note1 = value::witness_data(composer, tx.input_note[0]),
        .input_note2 = value::witness_data(composer, tx.input_note[1]),
        .output_note1 = value::witness_data(composer, tx.output_note[0]),
        .output_note2 = value::witness_data(composer, tx.output_note[1]),
        // Construction of partial_claim_note_witness_data includes construction of bridge_id, which contains many
        // constraints on the bridge_id's format and the bit_config's format:
        .partial_claim_note = claim::partial_claim_note_witness_data(composer, tx.partial_claim_note),
        .signing_pub_key = { .x = witness_ct(&composer, tx.signing_pub_key.x),
                             .y = witness_ct(&composer, tx.signing_pub_key.y) },
        .signature = stdlib::schnorr::convert_signature(&composer, tx.signature),
        .merkle_root = witness_ct(&composer, tx.old_data_root),
        .input_path1 = merkle_tree::create_witness_hash_path(composer, tx.input_path[0]),
        .input_path2 = merkle_tree::create_witness_hash_path(composer, tx.input_path[1]),
        .account_note_index =
            suint_ct(witness_ct(&composer, tx.account_note_index), DATA_TREE_DEPTH, "account_note_index"),
        .account_note_path = merkle_tree::create_witness_hash_path(composer, tx.account_note_path),
        .account_private_key = witness_ct(&composer, static_cast<fr>(tx.account_private_key)),
        .alias_hash = suint_ct(witness_ct(&composer, tx.alias_hash), ALIAS_HASH_BIT_LENGTH, "alias_hash"),
        .account_required = bool_ct(witness_ct(&composer, tx.account_required)),
        .backward_link = witness_ct(&composer, tx.backward_link),
        .allow_chain = witness_ct(&composer, tx.allow_chain),
    };
    auto outputs = join_split_circuit_component(inputs);
    const field_ct defi_root = witness_ct(&composer, 0);
    defi_root.assert_is_zero();

    // The following make up the public inputs to the circuit.
    inputs.proof_id.set_public();
    outputs.output_note1.set_public();
    outputs.output_note2.set_public();
    outputs.nullifier1.set_public();
    outputs.nullifier2.set_public();
    inputs.public_value.set_public();
    inputs.public_owner.set_public();
    outputs.public_asset_id.set_public();

    // Any public witnesses exposed from here on, will not be exposed by the rollup, and thus will
    // not be part of the calldata on chain, and will also not be part of tx id generation, or be signed over.
    inputs.merkle_root.set_public();
    outputs.tx_fee.set_public();
    inputs.asset_id.set_public();
    outputs.bridge_id.set_public();
    outputs.defi_deposit_value.set_public();
    defi_root.set_public();
    inputs.backward_link.set_public();
    inputs.allow_chain.set_public();
}

} // namespace join_split
} // namespace proofs
} // namespace rollup
