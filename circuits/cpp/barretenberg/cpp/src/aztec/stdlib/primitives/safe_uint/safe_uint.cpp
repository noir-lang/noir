#include "safe_uint.hpp"
#include "../bool/bool.hpp"
#include "../composers/composers.hpp"
#include "../../../rollup/constants.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext>

safe_uint_t<ComposerContext> safe_uint_t<ComposerContext>::operator+(const safe_uint_t& other) const
{
    return safe_uint_t((value + other.value), current_max + other.current_max, IS_UNSAFE);
}

template <typename ComposerContext>
safe_uint_t<ComposerContext> safe_uint_t<ComposerContext>::operator*(const safe_uint_t& other) const
{

    uint512_t new_max = uint512_t(current_max) * uint512_t(other.current_max);
    ASSERT(new_max.hi == 0);
    return safe_uint_t((value * other.value), new_max.lo, IS_UNSAFE);
}

template <typename ComposerContext> safe_uint_t<ComposerContext> safe_uint_t<ComposerContext>::normalize() const
{
    auto norm_value = value.normalize();
    return safe_uint_t(norm_value, current_max, IS_UNSAFE);
}

template <typename ComposerContext> void safe_uint_t<ComposerContext>::assert_is_zero(std::string const& msg) const
{
    value.assert_is_zero(msg);
}

template <typename ComposerContext> void safe_uint_t<ComposerContext>::assert_is_not_zero(std::string const& msg) const
{
    value.assert_is_not_zero(msg);
}

template <typename ComposerContext> bool_t<ComposerContext> safe_uint_t<ComposerContext>::is_zero() const
{
    return value.is_zero();
}

template <typename ComposerContext> barretenberg::fr safe_uint_t<ComposerContext>::get_value() const
{
    return value.get_value();
}

template <typename ComposerContext>
bool_t<ComposerContext> safe_uint_t<ComposerContext>::operator==(const safe_uint_t& other) const
{
    return value == other.value;
}

template <typename ComposerContext>
bool_t<ComposerContext> safe_uint_t<ComposerContext>::operator!=(const safe_uint_t& other) const
{
    return !operator==(other);
}
template <typename ComposerContext>
std::array<safe_uint_t<ComposerContext>, 3> safe_uint_t<ComposerContext>::slice(const uint8_t msb,
                                                                                const uint8_t lsb) const
{
    ASSERT(msb >= lsb);
    ASSERT(static_cast<size_t>(msb) <= rollup::MAX_NO_WRAP_INTEGER_BIT_LENGTH);
    const safe_uint_t lhs = *this;
    ComposerContext* ctx = lhs.get_context();

    const uint256_t value = uint256_t(get_value());
    // This should be caught by the proof itself, but the circuit creator will have now way of knowing where the issue
    // is
    ASSERT(value < (static_cast<uint256_t>(1) << rollup::MAX_NO_WRAP_INTEGER_BIT_LENGTH));
    const auto msb_plus_one = uint32_t(msb) + 1;
    const auto hi_mask = ((uint256_t(1) << (256 - uint32_t(msb))) - 1);
    const auto hi = (value >> msb_plus_one) & hi_mask;

    const auto lo_mask = (uint256_t(1) << lsb) - 1;
    const auto lo = value & lo_mask;

    const auto slice_mask = ((uint256_t(1) << (uint32_t(msb - lsb) + 1)) - 1);
    const auto slice = (value >> lsb) & slice_mask;
    safe_uint_t lo_wit, slice_wit, hi_wit;
    if (this->value.is_constant()) {
        hi_wit = safe_uint_t(hi);
        lo_wit = safe_uint_t(lo);
        slice_wit = safe_uint_t(slice);

    } else {
        hi_wit = safe_uint_t(witness_t(ctx, hi), rollup::MAX_NO_WRAP_INTEGER_BIT_LENGTH - uint32_t(msb), "hi_wit");
        lo_wit = safe_uint_t(witness_t(ctx, lo), lsb, "lo_wit");
        slice_wit = safe_uint_t(witness_t(ctx, slice), msb_plus_one - lsb, "slice_wit");
    }
    assert_equal(((hi_wit * safe_uint_t(uint256_t(1) << msb_plus_one)) + lo_wit +
                  (slice_wit * safe_uint_t(uint256_t(1) << lsb))));

    std::array<safe_uint_t, 3> result = { lo_wit, slice_wit, hi_wit };
    return result;
}

INSTANTIATE_STDLIB_TYPE(safe_uint_t);

} // namespace stdlib
} // namespace plonk
