#pragma once
#include "compute_circuit_data.hpp"
#include "../../fixtures/user_context.hpp"
#include "../notes/native/index.hpp"

namespace rollup {
namespace proofs {
namespace join_split {

using namespace notes::native;

template <typename WorldState> class JoinSplitTxFactory {
  public:
    JoinSplitTxFactory(WorldState& world_state, fixtures::user_context const& user)
        : world_state(world_state)
        , user(user)
    {}

    auto create_join_split_tx(std::vector<uint32_t> in_idx,
                              std::vector<uint32_t> in_value,
                              std::array<uint32_t, 2> out_value,
                              uint256_t public_input = 0,
                              uint256_t public_output = 0,
                              uint32_t account_note_idx = 0,
                              uint32_t asset_id = 0,
                              uint32_t nonce = 0,
                              uint32_t virtual_asset_id = 0)
    {
        auto num_inputs = in_idx.size();
        auto sender = user.owner.public_key;
        auto receiver = user.owner.public_key;

        auto asset_id2 = (virtual_asset_id >> (MAX_NUM_ASSETS_BIT_LENGTH - 1)) == 1 ? virtual_asset_id : asset_id;
        value::value_note input_note1 = { 0, asset_id, nonce, sender, fr::random_element(), 0, fr::random_element() };
        value::value_note input_note2 = { 0, asset_id2, nonce, sender, fr::random_element(), 0, fr::random_element() };

        switch (num_inputs) {
        case 0:
            in_idx = { 0, 1 };
            break;
        case 1:
            in_idx.resize(2);
            in_idx[1] = in_idx[0] + 1; // Not used, can't be the same as in_idx[0].
            input_note1 = {
                in_value[0], asset_id, nonce, sender, user.note_secret, 0, world_state.input_nullifiers[in_idx[0]]
            };
            input_note2 = { 0, asset_id, nonce, sender, fr::random_element(), 0, fr::random_element() };
            break;
        case 2:
            input_note1 = {
                in_value[0], asset_id, nonce, sender, user.note_secret, 0, world_state.input_nullifiers[in_idx[0]]
            };
            input_note2 = {
                in_value[1], asset_id2, nonce, sender, user.note_secret, 0, world_state.input_nullifiers[in_idx[1]]
            };
            break;
        }

        value::value_note output_note1 = { out_value[0], asset_id, nonce, receiver, user.note_secret, 0, fr(0) };
        value::value_note output_note2 = { out_value[1], asset_id, nonce, sender, user.note_secret, 0, fr(0) };
        notes::native::claim::claim_note_tx_data claim_note = { 0, 0, user.note_secret, fr(0) };

        auto get_proof_id = [&]() -> uint32_t {
            if (claim_note.deposit_value > 0) {
                return ProofIds::DEFI_DEPOSIT;
            }
            if (public_input > 0) {
                return ProofIds::DEPOSIT;
            }
            if (public_output > 0) {
                return ProofIds::WITHDRAW;
            }
            return ProofIds::SEND;
        };

        join_split_tx tx;
        tx.proof_id = get_proof_id();
        if (tx.proof_id == ProofIds::DEPOSIT) {
            tx.public_value = public_input;
        }
        if (tx.proof_id == ProofIds::WITHDRAW) {
            tx.public_value = public_output;
        }
        tx.public_owner = tx.public_value ? fr::random_element() : fr::zero();
        tx.asset_id = asset_id;
        tx.num_input_notes = static_cast<uint32_t>(num_inputs);
        tx.input_index = { in_idx[0], in_idx[1] };
        tx.old_data_root = world_state.data_tree.root();
        tx.input_path = { world_state.data_tree.get_hash_path(in_idx[0]),
                          world_state.data_tree.get_hash_path(in_idx[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.account_index = account_note_idx;
        tx.account_path = world_state.data_tree.get_hash_path(account_note_idx);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = 0;
        tx.nonce = nonce;
        tx.claim_note = claim_note;
        tx.propagated_input_index = 0;
        tx.backward_link = fr::zero();
        tx.allow_chain = 0;

        return tx;
    }

    /**
     * Performs any final stage fixed processing for the tx data.
     * Computes the nullifiers for the input notes, and sets the results as the input nullifiers on the output notes.
     * Computes and sets the signature.
     */
    void finalise_and_sign_tx(join_split_tx& tx,
                              fixtures::grumpkin_key_pair const& signer,
                              numeric::random::Engine* rand_engine = nullptr)
    {
        auto num_inputs = tx.num_input_notes;
        auto input_nullifier1 = compute_nullifier(tx.input_note[0].commit(), user.owner.private_key, num_inputs > 0);
        auto input_nullifier2 = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, num_inputs > 1);
        tx.output_note[0].input_nullifier = input_nullifier1;
        tx.output_note[1].input_nullifier = input_nullifier2;
        tx.claim_note.input_nullifier = tx.proof_id == ProofIds::DEFI_DEPOSIT ? input_nullifier1 : 0;
        tx.signature = sign_join_split_tx(tx, signer, rand_engine);
    }

    auto create_defi_deposit_tx(std::vector<uint32_t> in_note_idx,
                                std::vector<uint32_t> in_note_value,
                                std::array<uint32_t, 2> out_note_value,
                                uint256_t bridge_id,
                                uint32_t asset_id = 0,
                                uint32_t virtual_asset_id = 0)
    {
        auto tx =
            create_join_split_tx(in_note_idx, in_note_value, out_note_value, 0, 0, 0, asset_id, 0, virtual_asset_id);
        tx.proof_id = ProofIds::DEFI_DEPOSIT;
        tx.claim_note.bridge_id = bridge_id;
        tx.claim_note.deposit_value = tx.output_note[0].value;
        tx.claim_note.note_secret = user.note_secret;
        tx.output_note[0].value = 0;
        return tx;
    }

  private:
    WorldState& world_state;
    fixtures::user_context const& user;
};

} // namespace join_split
} // namespace proofs
} // namespace rollup
