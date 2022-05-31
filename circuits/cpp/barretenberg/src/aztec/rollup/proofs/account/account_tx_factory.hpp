#pragma once
#include "compute_circuit_data.hpp"
#include "../../fixtures/user_context.hpp"

namespace rollup {
namespace proofs {
namespace account {

template <typename WorldState> class AccountTxFactory {
  public:
    AccountTxFactory(WorldState& world_state, fixtures::user_context const& user)
        : world_state(world_state)
        , user(user)
    {}

    auto create_new_account_tx(uint32_t account_note_idx = 0)
    {
        account_tx tx;
        tx.merkle_root = world_state.data_tree.root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = user.owner.public_key;
        tx.new_signing_pub_key_1 = user.signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = user.signing_keys[1].public_key;
        tx.alias_hash = user.alias_hash;
        tx.create = true;
        tx.migrate = false;
        tx.account_note_index = account_note_idx;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_note_path = world_state.data_tree.get_hash_path(account_note_idx);
        return tx;
    }

    auto create_migrate_account_tx(grumpkin::g1::affine_element& new_owner_key,
                                   grumpkin::g1::affine_element new_signing_keys[2],
                                   uint32_t account_note_idx = 0)
    {
        account_tx tx;
        tx.merkle_root = world_state.data_tree.root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = new_owner_key;
        tx.new_signing_pub_key_1 = new_signing_keys[0];
        tx.new_signing_pub_key_2 = new_signing_keys[1];
        tx.alias_hash = user.alias_hash;
        tx.create = false;
        tx.migrate = true;
        tx.account_note_index = account_note_idx;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_note_path = world_state.data_tree.get_hash_path(account_note_idx);
        return tx;
    }

    auto create_add_signing_keys_to_account_tx(grumpkin::g1::affine_element new_signing_keys[2],
                                               uint32_t account_note_idx = 0)
    {
        account_tx tx;
        tx.merkle_root = world_state.data_tree.root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = user.owner.public_key;
        tx.new_signing_pub_key_1 = new_signing_keys[0];
        tx.new_signing_pub_key_2 = new_signing_keys[1];
        tx.alias_hash = user.alias_hash;
        tx.create = false;
        tx.migrate = false;
        tx.account_note_index = account_note_idx;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_note_path = world_state.data_tree.get_hash_path(account_note_idx);
        return tx;
    }

  private:
    WorldState& world_state;
    fixtures::user_context const& user;
};

} // namespace account
} // namespace proofs
} // namespace rollup
