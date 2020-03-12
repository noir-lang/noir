#include "uint.hpp"
#include "../composers/composers.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename Composer, typename Native>
bool_t<Composer> uint<Composer, Native>::operator>(const uint& other) const
{
    Composer* ctx = (context == nullptr) ? other.context : context;

    // we need to gaurantee that these values are 32 bits
    if (!is_constant() && witness_status != WitnessStatus::OK) {
        normalize();
    }
    if (!other.is_constant() && other.witness_status != WitnessStatus::OK) {
        other.normalize();
    }

    /**
     * if (a > b), then (a - b - 1) will be in the range [0, 2**{width}]
     * if !(a > b), then (b - a) will be in the range [0, 2**{width}]
     * if (a > b) = c and (a - b) = d, then this means that the following identity should always hold:
     *
     *          (d - 1).c - d.(1 - c) = 0
     *
     **/
    const uint256_t lhs = get_value();
    const uint256_t rhs = other.get_value();

    if (is_constant() && other.is_constant()) {
        return bool_t<Composer>(ctx, lhs > rhs);
    }

    const fr a = lhs;
    const fr b = rhs;
    const fr diff = a - b;

    const uint32_t lhs_idx = is_constant() ? ctx->zero_idx : witness_index;
    const uint32_t rhs_idx = other.is_constant() ? ctx->zero_idx : other.witness_index;
    const uint32_t diff_idx = ctx->add_variable(diff);

    const waffle::add_triple gate_a{ lhs_idx,
                                     rhs_idx,
                                     diff_idx,
                                     fr::one(),
                                     fr::neg_one(),
                                     fr::neg_one(),
                                     (additive_constant - other.additive_constant) };

    ctx->create_add_gate(gate_a);

    const uint256_t delta = lhs > rhs ? lhs - rhs - 1 : rhs - lhs;

    const bool_t<Composer> result = witness_t(ctx, lhs > rhs);

    const waffle::mul_quad gate_b{ diff_idx,
                                   result.witness_index,
                                   ctx->add_variable(delta),
                                   ctx->zero_idx,
                                   -fr(2),
                                   fr::one(),
                                   fr::one(),
                                   fr::one(),
                                   fr::zero(),
                                   fr::zero() };
    ctx->create_big_mul_gate(gate_b);

    return result;
}

template <typename Composer, typename Native>
bool_t<Composer> uint<Composer, Native>::operator<(const uint& other) const
{
    return other > *this;
}

template <typename Composer, typename Native>
bool_t<Composer> uint<Composer, Native>::operator>=(const uint& other) const
{
    return (!(other > *this)).normalize();
}

template <typename Composer, typename Native>
bool_t<Composer> uint<Composer, Native>::operator<=(const uint& other) const
{
    return (!(*this > other)).normalize();
}

template <typename Composer, typename Native>
bool_t<Composer> uint<Composer, Native>::operator==(const uint& other) const
{
    // casting to a field type will ensure that lhs / rhs are both normalized
    const field_t<Composer> lhs = *this;
    const field_t<Composer> rhs = other;

    return (lhs == rhs).normalize();
}

template <typename Composer, typename Native>
bool_t<Composer> uint<Composer, Native>::operator!=(const uint& other) const
{
    return (!(*this == other)).normalize();
}

template <typename Composer, typename Native> bool_t<Composer> uint<Composer, Native>::operator!() const
{
    return (field_t<Composer>(*this).is_zero()).normalize();
}
template class uint<waffle::TurboComposer, uint8_t>;
template class uint<waffle::TurboComposer, uint16_t>;
template class uint<waffle::TurboComposer, uint32_t>;
template class uint<waffle::TurboComposer, uint64_t>;

template class uint<waffle::StandardComposer, uint8_t>;
template class uint<waffle::StandardComposer, uint16_t>;
template class uint<waffle::StandardComposer, uint32_t>;
template class uint<waffle::StandardComposer, uint64_t>;

template class uint<waffle::MiMCComposer, uint8_t>;
template class uint<waffle::MiMCComposer, uint16_t>;
template class uint<waffle::MiMCComposer, uint32_t>;
template class uint<waffle::MiMCComposer, uint64_t>;
} // namespace stdlib
} // namespace plonk