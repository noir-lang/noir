#pragma once
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace notes {

using namespace plonk::stdlib::types::turbo;

constexpr size_t NOTE_VALUE_BIT_LENGTH = 252;

// note encryptions and nullifiers should use different pedersen generators to eliminate partial collisions
struct public_note {
    point_ct ciphertext;
};

struct private_note {
    point_ct owner;
    // note value must be 252 bits or smaller - we assume this is checked elsewhere
    field_ct value;
    // this secret must be 250 bits or smaller - it cannot be taken from the entire field_ct range
    field_ct secret;
    // this asset_id value must be 32 bits or smaller
    field_ct asset_id;
};

public_note encrypt_note(const private_note& plaintext);

field_ct compute_nullifier(const field_ct& account_private_key,
                           const public_note& ciphertext,
                           const field_ct& tree_index,
                           const bool_ct& is_real);
// template <size_t num_bits> note_triple fixed_base_scalar_mul(const field_ct& in, const size_t generator_index);
// extern template note_triple fixed_base_scalar_mul<32>(const field_ct& in, const size_t generator_index);
// extern template note_triple fixed_base_scalar_mul<250>(const field_ct& in, const size_t generator_index);

} // namespace notes
} // namespace proofs
} // namespace rollup