#include "encrypt_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>
#include "../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

// compute a pedersen hash of `scalar` and add the resulting point into `accumulator`, iff scalar != 0
template <size_t num_scalar_mul_bits>
point_ct conditionally_hash_and_accumulate(const point_ct& accumulator,
                                           const field_ct& scalar,
                                           const size_t generator_index)
{
    point_ct p_1 = group_ct::fixed_base_scalar_mul<num_scalar_mul_bits>(scalar, generator_index);

    bool_ct is_zero = scalar.is_zero();

    // If scalar = 0 we want to return accumulator, as g^{0} = 1
    // If scalar != 0, we want to return accumulator + p_1
    field_ct lambda = (accumulator.y - p_1.y) / (accumulator.x - p_1.x);
    field_ct x_2 = (lambda * lambda) - (accumulator.x + p_1.x);
    field_ct y_2 = lambda * (p_1.x - x_2) - p_1.y;

    x_2 = (accumulator.x - x_2) * field_ct(is_zero) + x_2;
    y_2 = (accumulator.y - y_2) * field_ct(is_zero) + y_2;
    return { x_2, y_2 };
}

point_ct accumulate(const point_ct& accumulator, const point_ct& p_1)
{
    field_ct lambda = (p_1.y - accumulator.y) / (p_1.x - accumulator.x);
    field_ct x_2 = (lambda * lambda) - (p_1.x + accumulator.x);
    field_ct y_2 = lambda * (accumulator.x - x_2) - accumulator.y;
    return { x_2, y_2 };
}

/**
 * Compute a pedersen hash of the plaintext:
 * [output] = plaintext.value * [g0] + plaintext.secret * [g1] + plaintext.asset_id * [g2] + plaintext.owner.x * [g3] +
 * plaintext.owner.y * [g4]
 **/
point_ct encrypt_note(const value_note& plaintext)
{
    point_ct accumulator = group_ct::fixed_base_scalar_mul<250>(plaintext.secret, TX_NOTE_HASH_INDEX + 1);

    accumulator =
        conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(accumulator, plaintext.value, TX_NOTE_HASH_INDEX);
    accumulator = conditionally_hash_and_accumulate<32>(accumulator, plaintext.asset_id, TX_NOTE_HASH_INDEX + 2);
    accumulator = accumulate(accumulator,
                             pedersen::compress_to_point(plaintext.owner.x, plaintext.owner.y, TX_NOTE_HASH_INDEX + 3));

    return accumulator;
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup
