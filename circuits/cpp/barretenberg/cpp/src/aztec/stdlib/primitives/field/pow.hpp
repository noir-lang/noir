#pragma once
#include <common/assert.hpp>

#include "./field.hpp"
#include "../uint/uint.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer>
static field_t<Composer> pow(const field_t<Composer>& base, const uint32<Composer>& exponent)
{
    using field_pt = field_t<Composer>;

    auto* ctx = base.get_context() ? base.get_context() : exponent.get_context();

    field_pt accumulator(ctx, 1);
    field_pt mul_coefficient = base - 1;
    for (size_t i = 0; i < 32; ++i) {
        accumulator *= accumulator;
        const auto bit = exponent.at(31 - i);
        accumulator *= (mul_coefficient * bit + 1);
    }
    accumulator = accumulator.normalize();
    return accumulator;
}

} // namespace stdlib
} // namespace plonk
