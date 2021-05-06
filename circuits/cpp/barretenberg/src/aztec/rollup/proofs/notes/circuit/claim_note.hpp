#pragma once
#include <stdlib/types/turbo.hpp>
#include "../native/claim_note.hpp"
#include "../constants.hpp"
#include "bridge_id.hpp"
#include "encrypt_note.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace plonk::stdlib::types::turbo;

class claim_note {
  public:
    field_ct deposit_value;
    bridge_id bridge_id;
    field_ct packed_bridge_id;
    field_ct note_secret;
    field_ct defi_interaction_nonce;

    claim_note(Composer& composer, native::claim_note_tx_data const& note)
        : deposit_value(witness_ct(&composer, note.deposit_value))
        , bridge_id(create_bridge_id_witness(composer, note.bridge_id))
        , packed_bridge_id(bridge_id.to_field())
        , note_secret(witness_ct(&composer, note.note_secret))
        , defi_interaction_nonce(witness_ct(&composer, note.defi_interaction_nonce))
    {
        composer.create_range_constraint(deposit_value.witness_index, NOTE_VALUE_BIT_LENGTH);
        composer.create_range_constraint(defi_interaction_nonce.witness_index, 32);
    }

    point_ct encrypt(field_ct const& nonce, point_ct const& owner) const
    {
        point_ct partial_state = encrypt_partial_note(note_secret, nonce, owner);

        point_ct accumulator =
            group_ct::fixed_base_scalar_mul<254>(packed_bridge_id, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_BRIDGE_ID);

        accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
            accumulator, deposit_value, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_VALUE);

        accumulator =
            accumulate(accumulator,
                       pedersen::compress_to_point(
                           partial_state.x, partial_state.y, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_PARTIAL_STATE));
        accumulator = conditionally_hash_and_accumulate<32>(
            accumulator, defi_interaction_nonce, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_DEFI_INTERACTION_NONCE);
        return accumulator;
    }
};

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup