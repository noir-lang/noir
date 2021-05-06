#pragma once
#include "value_note.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

template <size_t num_scalar_mul_bits>
point_ct conditionally_hash_and_accumulate(const point_ct& accumulator,
                                           const field_ct& scalar,
                                           const size_t generator_index);
point_ct accumulate(const point_ct& accumulator, const point_ct& p_1);

point_ct encrypt_note(const value_note& plaintext);
point_ct encrypt_partial_note(field_ct const& secret, field_ct const& nonce, point_ct const& owner);

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup