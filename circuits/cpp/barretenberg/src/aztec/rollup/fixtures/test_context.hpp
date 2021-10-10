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
    {}

    void append_value_notes(std::vector<uint32_t> const& values, uint32_t asset_id = 0)
    {
        for (auto v : values) {
            native::value::value_note note = { v, asset_id, 0, user.owner.public_key, user.note_secret, 0 };
            world_state.append_data_note(note);
        }
    }

    void append_account_notes()
    {
        auto account_alias_id = fixtures::generate_account_alias_id(user.alias_hash, 1);
        native::account::account_note note1 = { account_alias_id,
                                                user.owner.public_key,
                                                user.signing_keys[0].public_key };
        native::account::account_note note2 = { account_alias_id,
                                                user.owner.public_key,
                                                user.signing_keys[1].public_key };
        world_state.append_data_note(note1);
        world_state.append_data_note(note2);
    }

    void nullify_account_alias_id(fr const& account_alias_id)
    {
        world_state.nullify(native::account::compute_account_alias_id_nullifier(account_alias_id));
    }

    std::vector<uint8_t> create_join_split_proof(std::vector<uint32_t> in_note_idx,
                                                 std::vector<uint32_t> in_note_value,
                                                 std::array<uint32_t, 2> out_note_value,
                                                 uint256_t public_input = 0,
                                                 uint256_t public_output = 0,
                                                 uint256_t tx_fee = 7,
                                                 uint32_t account_note_idx = 0,
                                                 uint32_t asset_id = 0,
                                                 uint32_t nonce = 0)
    {
        auto tx = js_tx_factory.create_join_split_tx(in_note_idx,
                                                     in_note_value,
                                                     out_note_value,
                                                     public_input,
                                                     public_output,
                                                     tx_fee,
                                                     account_note_idx,
                                                     asset_id,
                                                     nonce);
        auto signer = nonce ? user.signing_keys[0] : user.owner;
        return join_split::create_proof(tx, signer, js_cd);
    }

    std::vector<uint8_t> create_defi_proof(std::vector<uint32_t> in_note_idx,
                                           std::vector<uint32_t> in_note_value,
                                           std::array<uint32_t, 2> out_note_value,
                                           uint256_t bridge_id,
                                           uint32_t asset_id = 0,
                                           uint32_t nonce = 0)
    {

        auto tx = js_tx_factory.create_defi_deposit_tx(in_note_idx, in_note_value, out_note_value, bridge_id, asset_id);
        auto signer = nonce ? user.signing_keys[0] : user.owner;
        return join_split::create_proof(tx, signer, js_cd);
    }

    std::vector<uint8_t> create_account_proof(uint32_t nonce = 0, uint32_t account_note_idx = 0)
    {
        auto tx = account_tx_factory.create_tx(nonce, account_note_idx);
        auto signer = nonce ? user.signing_keys[0] : user.owner;
        return account::create_proof(tx, signer, account_cd);
    }

    auto create_claim_tx(uint256_t bridge_id, uint256_t deposit_value, uint32_t claim_note_index, uint256_t fee)
    {
        uint32_t interaction_nonce = 0;
        // Assume this claim note was created against the most recent matching bridge id interaction.
        for (size_t i = defi_interactions.size(); i > 0; --i) {
            if (defi_interactions[i - 1].bridge_id == bridge_id) {
                interaction_nonce = static_cast<uint32_t>(i - 1);
                break;
            }
        }

        auto partial_state =
            notes::native::value::create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0);
        notes::native::claim::claim_note claim_note = {
            deposit_value, bridge_id, interaction_nonce, fee, partial_state
        };
        return claim_tx_factory.create_claim_tx(
            world_state.defi_tree.root(), claim_note_index, claim_note, defi_interactions[interaction_nonce]);
    }

    std::vector<uint8_t> create_claim_proof(uint256_t bridge_id,
                                            uint256_t deposit_value,
                                            uint32_t claim_note_index,
                                            uint256_t fee)
    {
        auto tx = create_claim_tx(bridge_id, deposit_value, claim_note_index, fee);
        return claim::create_proof(tx, claim_cd);
    }

    /*
     * Updates the next slot in the root tree with the latest data root.
     * Inserts the given defi interaction notes from the previous rollup into the defi tree.
     */
    void start_next_root_rollup(std::vector<native::defi_interaction::note> const& dins_ = {})
    {
        uint32_t rollup_id = static_cast<uint32_t>(world_state.root_tree.size());
        uint32_t din_insertion_index = (rollup_id - 1) * NUM_INTERACTION_RESULTS_PER_BLOCK;
        world_state.update_root_tree_with_data_root();

        auto dins = dins_;
        defi_interactions.resize(din_insertion_index + dins.size());
        for (size_t i = 0; i < dins.size(); ++i) {
            auto nonce = din_insertion_index + i;
            dins[i].interaction_nonce = static_cast<uint32_t>(nonce);
            defi_interactions[nonce] = dins[i];
        }

        world_state.add_defi_notes(dins);
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
};

} // namespace fixtures
} // namespace rollup