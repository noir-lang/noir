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
// typedef std::pair<claim_note, point_ct> claim_note_pair;

struct claim_note {
    field_ct deposit_value;

    field_ct bridge_id;

    // populated in rollup ciruit; in deposit circuit this defaults to 0
    field_ct defi_interaction_nonce;

    point_ct partial_state;
};

struct claim_note_tx_data_ct {
    field_ct deposit_value;
    bridge_id bridge_id;
    field_ct note_secret;
    field_ct defi_interaction_nonce;
};

inline claim_note_tx_data_ct create_claim_note_witness(Composer& composer, native::claim_note_tx_data const& note)
{
    // TODO: we must verify if the partial_state comes from the correct note owner
    field_ct witness_deposit_value = witness_ct(&composer, note.deposit_value);
    bridge_id bridge_id = create_bridge_id_witness(composer, note.bridge_id);
    field_ct note_secret = witness_ct(&composer, note.note_secret);
    field_ct defi_interaction_nonce = witness_ct(&composer, note.defi_interaction_nonce);

    composer.create_range_constraint(witness_deposit_value.witness_index, NOTE_VALUE_BIT_LENGTH);
    composer.create_range_constraint(defi_interaction_nonce.witness_index, 32);

    return { witness_deposit_value, bridge_id, note_secret, defi_interaction_nonce };
}

inline claim_note compute_claim_note(claim_note_tx_data_ct const& note, field_ct const& nonce, point_ct const& owner)
{
    point_ct partial_state = encrypt_partial_note(note.note_secret, nonce, owner);
    return { note.deposit_value, note.bridge_id.to_field(), note.defi_interaction_nonce, partial_state };
}

inline point_ct encrypt_note(const claim_note& plaintext)
{
    point_ct accumulator =
        group_ct::fixed_base_scalar_mul<254>(plaintext.bridge_id, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_BRIDGE_ID);

    accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(
        accumulator, plaintext.deposit_value, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_VALUE);

    accumulator = accumulate(accumulator,
                             pedersen::compress_to_point(plaintext.partial_state.x,
                                                         plaintext.partial_state.y,
                                                         GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_PARTIAL_STATE));
    accumulator = conditionally_hash_and_accumulate<32>(
        accumulator, plaintext.defi_interaction_nonce, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_DEFI_INTERACTION_NONCE);
    return accumulator;
}

// inline claim_note_pair create_note_pair(Composer& composer,
//                                         native::claim_note_tx_data const& note,
//                                         field_ct const& nonce,
//                                         point_ct const& owner)
// {
//     auto note_witness = create_claim_note_witness(composer, note, nonce, owner);
//     auto enc_note = encrypt_note(note_witness);
//     return { note_witness, enc_note };
// }

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup