#pragma once
#include "user_context.hpp"
#include "../world_state/world_state.hpp"
#include "../proofs/notes/native/index.hpp"
#include "../proofs/join_split/index.hpp"
#include "../proofs/account/index.hpp"
#include "../proofs/claim/index.hpp"
#include <stdlib/merkle_tree/index.hpp>

namespace rollup {
namespace fixtures {

using namespace plonk::stdlib::merkle_tree;
using namespace ::rollup::proofs;
using namespace ::rollup::proofs::notes;
using WorldState = world_state::WorldState<MemoryStore>;

class TestContext {
  public:
    TestContext(join_split::circuit_data const& js_cd,
                account::circuit_data const& account_cd,
                claim::circuit_data const& claim_cd)
        : rand_engine(&numeric::random::get_debug_engine(true))
        , user(fixtures::create_user_context(rand_engine))
        , js_tx_factory(world_state, user)
        , account_tx_factory(world_state, user)
        , claim_tx_factory(world_state, user)
        , js_cd(js_cd)
        , account_cd(account_cd)
        , claim_cd(claim_cd)
        , extra_key_pairs{
            fixtures::create_key_pair(rand_engine), fixtures::create_key_pair(rand_engine),
            fixtures::create_key_pair(rand_engine), fixtures::create_key_pair(rand_engine),
            fixtures::create_key_pair(rand_engine),
        }
    {}

    void append_value_notes(std::vector<uint32_t> const& values, uint32_t asset_id = 0)
    {
        for (auto v : values) {
            // Use the insertion index (data_tree.size()) as the input_nullifier.
            // This ensures consistent commitments in tests which is important when leveraging fixtures.
            native::value::value_note note = {
                v, asset_id, 0, user.owner.public_key, user.note_secret, 0, world_state.data_tree.size()
            };
            world_state.append_data_note(note);
        }
    }

    void append_account_notes()
    {
        native::account::account_note note1 = {
            .alias_hash = user.alias_hash,
            .owner_key = user.owner.public_key,
            .signing_key = user.signing_keys[0].public_key,
        };
        native::account::account_note note2 = {
            .alias_hash = user.alias_hash,
            .owner_key = user.owner.public_key,
            .signing_key = user.signing_keys[1].public_key,
        };
        world_state.append_data_note(note1);
        world_state.append_data_note(note2);
    }

    void nullify_account_alias_hash(fr const& account_alias_hash)
    {
        world_state.nullify(native::account::compute_account_alias_hash_nullifier(account_alias_hash));
    }

    void nullify_account_public_key(grumpkin::g1::affine_element const& account_public_key)
    {
        world_state.nullify(native::account::compute_account_public_key_nullifier(account_public_key));
    }

    std::vector<uint8_t> create_join_split_proof(std::vector<uint32_t> in_note_idx,
                                                 std::vector<uint32_t> in_note_value,
                                                 std::array<uint32_t, 2> out_note_value,
                                                 uint256_t public_input = 0,
                                                 uint256_t public_output = 0,
                                                 uint32_t account_note_idx = 0,
                                                 uint32_t asset_id = 0,
                                                 bool account_required = false)
    {
        auto tx = js_tx_factory.create_join_split_tx(in_note_idx,
                                                     in_note_value,
                                                     out_note_value,
                                                     public_input,
                                                     public_output,
                                                     account_note_idx,
                                                     asset_id,
                                                     account_required);
        auto signer = account_required ? user.signing_keys[0] : user.owner;
        js_tx_factory.finalise_and_sign_tx(tx, signer);
        return join_split::create_proof(tx, js_cd);
    }

    std::vector<uint8_t> create_defi_proof(std::vector<uint32_t> in_note_indices,
                                           std::vector<uint32_t> in_note_values,
                                           std::array<uint32_t, 2> out_note_values,
                                           uint256_t bridge_call_data,
                                           uint32_t asset_id = 0,
                                           bool account_required = false,
                                           uint32_t virtual_asset_id = 0)
    {

        auto tx = js_tx_factory.create_defi_deposit_tx(
            in_note_indices, in_note_values, out_note_values, bridge_call_data, asset_id, virtual_asset_id);
        auto signer = account_required ? user.signing_keys[0] : user.owner;
        js_tx_factory.finalise_and_sign_tx(tx, signer);
        return join_split::create_proof(tx, js_cd);
    }

    std::vector<uint8_t> create_new_account_proof(uint32_t account_note_idx = 0)
    {
        auto tx = account_tx_factory.create_new_account_tx(account_note_idx);
        return account::create_proof(tx, user.owner, account_cd);
    }

    std::vector<uint8_t> create_add_signing_keys_to_account_proof(uint32_t account_note_idx = 0)
    {
        grumpkin::g1::affine_element new_signing_keys[2] = { extra_key_pairs[0].public_key,
                                                             extra_key_pairs[1].public_key };
        auto tx = account_tx_factory.create_add_signing_keys_to_account_tx(new_signing_keys, account_note_idx);
        return account::create_proof(tx, user.signing_keys[0], account_cd);
    }

    std::vector<uint8_t> create_migrate_account_proof(uint32_t account_note_idx = 0)
    {
        grumpkin::g1::affine_element new_owner_key = extra_key_pairs[0].public_key;
        grumpkin::g1::affine_element new_signing_keys[2] = { extra_key_pairs[1].public_key,
                                                             extra_key_pairs[2].public_key };
        auto tx = account_tx_factory.create_migrate_account_tx(new_owner_key, new_signing_keys, account_note_idx);
        return account::create_proof(tx, user.signing_keys[0], account_cd);
    }

    auto create_claim_tx(uint256_t bridge_call_data,
                         uint256_t deposit_value,
                         uint32_t claim_note_index,
                         uint32_t defi_note_index,
                         uint256_t fee)
    {
        auto& defi_note = defi_interactions[defi_note_index];
        auto partial_state =
            notes::native::value::create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0);
        notes::native::claim::claim_note claim_note = {
            deposit_value, bridge_call_data, defi_note.interaction_nonce,
            fee,           partial_state,    world_state.input_nullifiers[claim_note_index]
        };
        return claim_tx_factory.create_claim_tx(
            world_state.defi_tree.root(), claim_note_index, defi_note_index, claim_note, defi_note);
    }

    std::vector<uint8_t> create_claim_proof(uint256_t bridge_call_data,
                                            uint256_t deposit_value,
                                            uint32_t claim_note_index,
                                            uint32_t defi_note_index,
                                            uint256_t fee)
    {
        auto tx = create_claim_tx(bridge_call_data, deposit_value, claim_note_index, defi_note_index, fee);
        return claim::create_proof(tx, claim_cd);
    }

    /**
     * Updates the next slot in the root tree with the latest data root.
     * Inserts the given defi interaction notes from the previous rollup into the defi tree.
     * @param dins_ - defi interaction NOTES (not 'nonce')
     */
    uint32_t start_next_root_rollup(std::vector<native::defi_interaction::note> const& dins_ = {})
    {
        uint32_t rollup_id = static_cast<uint32_t>(world_state.root_tree.size());
        // defi notes go into this rollup, but the nonces were 'generated' in the previous rollup
        uint32_t initial_din_insertion_index = rollup_id * NUM_INTERACTION_RESULTS_PER_BLOCK;
        uint32_t initial_interaction_nonce = (rollup_id - 1) * NUM_INTERACTION_RESULTS_PER_BLOCK;
        world_state.update_root_tree_with_data_root();

        auto dins = dins_;
        defi_interactions.resize(initial_din_insertion_index + dins.size());
        for (size_t i = 0; i < dins.size(); ++i) {
            auto din_insertion_index = initial_din_insertion_index + i;
            auto interaction_nonce = initial_interaction_nonce + i;
            dins[i].interaction_nonce = static_cast<uint32_t>(interaction_nonce);
            defi_interactions[din_insertion_index] = dins[i];
        }

        world_state.add_defi_notes(dins, initial_din_insertion_index);
        return initial_din_insertion_index;
    }

    numeric::random::Engine* rand_engine;
    WorldState world_state;
    fixtures::user_context user;
    join_split::JoinSplitTxFactory<WorldState> js_tx_factory;
    account::AccountTxFactory<WorldState> account_tx_factory;
    claim::ClaimTxFactory<WorldState> claim_tx_factory;
    join_split::circuit_data const& js_cd;
    account::circuit_data const& account_cd;
    claim::circuit_data const& claim_cd;
    std::vector<native::defi_interaction::note> defi_interactions;
    std::array<fixtures::grumpkin_key_pair, 5> extra_key_pairs;
};

} // namespace fixtures
} // namespace rollup