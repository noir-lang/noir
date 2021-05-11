#pragma once
#include <stdlib/types/turbo.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include "../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

// compute a pedersen hash of `scalar` and add the resulting point into `accumulator`, iff scalar != 0
template <size_t num_scalar_mul_bits>
inline point_ct conditionally_hash_and_accumulate(const point_ct& accumulator,
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

inline point_ct accumulate(const point_ct& accumulator, const point_ct& p_1)
{
    field_ct lambda = (p_1.y - accumulator.y) / (p_1.x - accumulator.x);
    field_ct x_2 = (lambda * lambda) - (p_1.x + accumulator.x);
    field_ct y_2 = lambda * (accumulator.x - x_2) - accumulator.y;
    return { x_2, y_2 };
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup