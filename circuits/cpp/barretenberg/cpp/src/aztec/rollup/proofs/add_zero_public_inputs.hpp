#pragma once
#include <stdlib/types/types.hpp>

namespace rollup {
namespace proofs {

using namespace plonk::stdlib::types;

inline void add_zero_public_inputs(Composer& composer, size_t num)
{
    for (size_t i = 0; i < num; ++i) {
        auto zero = field_ct(witness_ct(&composer, 0));
        zero.assert_is_zero();
        zero.set_public();
    }
}

} // namespace proofs
} // namespace rollup
