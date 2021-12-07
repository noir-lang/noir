#pragma once
#include <stdlib/types/turbo.hpp>
#include "witness_data.hpp"
#include "commit.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

using namespace plonk::stdlib::types::turbo;

struct value_note {
    point_ct owner;
    suint_ct value;
    field_ct secret;
    suint_ct asset_id;
    suint_ct nonce;
    field_ct input_nullifier;
    field_ct commitment;
    field_ct creator_pubkey;
    bool_ct is_virtual;
    suint_ct virtual_note_nonce;

    value_note(witness_data const& note)
        : owner(note.owner)
        , value(note.value)
        , secret(note.secret)
        , asset_id(note.asset_id)
        , nonce(note.nonce)
        , input_nullifier(note.input_nullifier)
        , commitment(value::commit(note))
        , creator_pubkey(note.creator_pubkey)
    {
        const auto loan_idx = MAX_NUM_ASSETS_BIT_LENGTH - 1; //  bit 29

        // extract the most significant bit of the asset id: bit 29-30
        const auto sliced_asset_id = asset_id.slice(loan_idx + 1, loan_idx);

        // 'virtual' notes defined by asset id msb being +1.
        // A virtual note does not have an ERC20 token equivalent and exists only inside the Aztec network
        // The low 29 bits of the asset id represent the defi interaction nonce of the defi interaction that created the
        // note
        is_virtual = sliced_asset_id[1] == 1;
        virtual_note_nonce = sliced_asset_id[0];
    }

    operator byte_array_ct() const { return byte_array_ct(commitment); }
};

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup