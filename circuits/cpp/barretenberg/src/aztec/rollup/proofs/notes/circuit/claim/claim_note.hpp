#pragma once
#include <stdlib/types/turbo.hpp>
#include "../pedersen_note.hpp"
#include "bridge_id.hpp"
#include "witness_data.hpp"
#include "create_partial_value_note.hpp"
#include "encrypt.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

struct claim_note {
    field_ct deposit_value;
    field_ct bridge_id;
    field_ct defi_interaction_nonce;
    point_ct partial_state;
    point_ct encrypted;

    claim_note(witness_data const& data, point_ct const& owner, field_ct const& nonce)
    {
        deposit_value = data.deposit_value;
        bridge_id = data.bridge_id.to_field();
        defi_interaction_nonce = data.defi_interaction_nonce;
        partial_state = create_partial_value_note(data.note_secret, nonce, owner);
        encrypted = encrypt(data.deposit_value, bridge_id, data.defi_interaction_nonce, partial_state);
    }

    operator byte_array_ct() const { return byte_array_ct(encrypted.x).write(encrypted.y); }
};

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup