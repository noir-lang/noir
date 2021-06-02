#pragma once
#include "compute_circuit_data.hpp"
#include "../../fixtures/user_context.hpp"

namespace rollup {
namespace proofs {
namespace join_split {

using namespace notes::native::value;

template <typename WorldState> class JoinSplitTxFactory {
  public:
    JoinSplitTxFactory(WorldState& world_state, fixtures::user_context const& user)
        : world_state(world_state)
        , user(user)
    {}

    auto create_join_split_tx(std::vector<uint32_t> in_note_idx,
                              std::vector<uint32_t> in_note_value,
                              std::array<uint32_t, 2> out_note_value,
                              uint256_t public_input = 0,
                              uint256_t public_output = 0,
                              uint256_t tx_fee = 0,
                              uint32_t account_note_idx = 0,
                              uint32_t asset_id = 0,
                              uint32_t nonce = 0)
    {
        auto sender = user.owner.public_key;
        auto receiver = user.owner.public_key;

        // Fake input notes want random note secrets to ensure we never get nullifier conflicts.
        auto random_note_secret = fr::random_element();
        random_note_secret.data[3] = random_note_secret.data[3] & 0x03FFFFFFFFFFFFFFULL;
        random_note_secret = random_note_secret.to_montgomery_form();

        auto num_input_notes = static_cast<uint32_t>(in_note_idx.size());
        std::vector<fr> input_note_secrets(2, user.note_secret);

        if (num_input_notes == 0) {
            in_note_idx.resize(2);
            in_note_value.resize(2);
            in_note_idx[0] = 0;
            in_note_idx[1] = 1;
            input_note_secrets[0] = random_note_secret;
            input_note_secrets[1] = random_note_secret;
        }

        if (num_input_notes == 1) {
            in_note_idx.resize(2);
            in_note_value.resize(2);
            in_note_idx[1] = 1;
            input_note_secrets[1] = random_note_secret;
        }

        value_note input_note1 = { in_note_value[0], asset_id, nonce, sender, input_note_secrets[0] };
        value_note input_note2 = { in_note_value[1], asset_id, nonce, sender, input_note_secrets[1] };
        value_note output_note1 = { out_note_value[0], asset_id, nonce, receiver, user.note_secret };
        value_note output_note2 = { out_note_value[1], asset_id, nonce, sender, user.note_secret };

        join_split_tx tx;
        tx.public_input = public_input + tx_fee;
        tx.public_output = public_output;
        tx.asset_id = asset_id;
        tx.num_input_notes = num_input_notes;
        tx.input_index = { in_note_idx[0], in_note_idx[1] };
        tx.old_data_root = world_state.data_tree.root();
        tx.input_path = { world_state.data_tree.get_hash_path(in_note_idx[0]),
                          world_state.data_tree.get_hash_path(in_note_idx[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.account_index = account_note_idx;
        tx.account_path = world_state.data_tree.get_hash_path(account_note_idx);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = 0;
        tx.nonce = nonce;
        tx.claim_note.deposit_value = 0;
        tx.claim_note.owner = receiver;
        tx.claim_note.owner_nonce = nonce;
        tx.claim_note.defi_interaction_nonce = 0;
        tx.input_owner = fr::zero();
        tx.output_owner = fr::zero();

        return tx;
    }

    auto create_defi_deposit_tx(std::vector<uint32_t> in_note_idx,
                                std::vector<uint32_t> in_note_value,
                                std::array<uint32_t, 2> out_note_value,
                                uint256_t bridge_id,
                                uint32_t asset_id = 0)
    {
        auto tx = create_join_split_tx(in_note_idx, in_note_value, out_note_value, 0, 0, 0, 0, asset_id);
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
