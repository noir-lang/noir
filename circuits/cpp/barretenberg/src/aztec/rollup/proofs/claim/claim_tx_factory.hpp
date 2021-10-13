#pragma once
#include "get_circuit_data.hpp"
#include "claim_tx.hpp"
#include "../../fixtures/user_context.hpp"

namespace rollup {
namespace proofs {
namespace claim {

using namespace notes::native::value;

template <typename WorldState> class ClaimTxFactory {
  public:
    ClaimTxFactory(WorldState& world_state, fixtures::user_context const& user)
        : world_state(world_state)
        , user(user)
    {}

    auto create_claim_tx(barretenberg::fr const& defi_root,
                         uint32_t claim_note_index,
                         notes::native::claim::claim_note const& claim_note,
                         notes::native::defi_interaction::note const& defi_interaction_note)
    {
        claim_tx tx;
        tx.data_root = world_state.data_tree.root();
        tx.defi_root = defi_root;
        tx.claim_note_index = claim_note_index;
        tx.claim_note_path = world_state.data_tree.get_hash_path(claim_note_index);
        tx.claim_note = claim_note;
        tx.defi_interaction_note_path = world_state.defi_tree.get_hash_path(defi_interaction_note.interaction_nonce);
        tx.defi_interaction_note = defi_interaction_note;
        tx.defi_interaction_note_dummy_nullifier_nonce = fr::random_element();
        tx.output_value_a = claim_note.deposit_value * defi_interaction_note.total_output_a_value /
                            defi_interaction_note.total_input_value;
        tx.output_value_b = claim_note.deposit_value * defi_interaction_note.total_output_b_value /
                            defi_interaction_note.total_input_value;
        return tx;
    }

  private:
    WorldState& world_state;
    fixtures::user_context const& user;
};

} // namespace claim
} // namespace proofs
} // namespace rollup
