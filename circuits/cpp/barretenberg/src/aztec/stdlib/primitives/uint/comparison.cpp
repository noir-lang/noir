#include "../composers/composers.hpp"
#include "uint.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename Composer, typename Native>
bool_t<Composer> uint<Composer, Native>::operator>(const uint& other) const
{
    Composer* ctx = (context == nullptr) ? other.context : context;

    field_t<Composer> a(*this);
    field_t<Composer> b(other);
    bool result_witness = uint256_t(a.get_value()) > uint256_t(b.get_value());

    if (is_constant() && other.is_constant()) {
        return bool_t<Composer>(ctx, result_witness);
    }

    const bool_t<Composer> result = witness_t<Composer>(ctx, result_witness);

    /**
     * if (a > b), then (a - b - 1) will be in the range [0, 2**{width}]
     * if !(a > b), then (b - a) will be in the range [0, 2**{width}]
     * i.e. (a - b - 1)result + (b - a)(1 - result) should be positive
     **/
    const auto diff = a - b;
    const auto comparison_check =
        diff.madd(field_t<Composer>(result) * 2 - field_t<Composer>(1), -field_t<Composer>(result));
    comparison_check.create_range_constraint(width);

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
    const field_t<Composer> lhs = static_cast<field_t<Composer>>(*this);
    const field_t<Composer> rhs = static_cast<field_t<Composer>>(other);

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
template class uint<waffle::PlookupComposer, uint8_t>;
template class uint<waffle::PlookupComposer, uint16_t>;
template class uint<waffle::PlookupComposer, uint32_t>;
template class uint<waffle::PlookupComposer, uint64_t>;

template class uint<waffle::TurboComposer, uint8_t>;
template class uint<waffle::TurboComposer, uint16_t>;
template class uint<waffle::TurboComposer, uint32_t>;
template class uint<waffle::TurboComposer, uint64_t>;

template class uint<waffle::StandardComposer, uint8_t>;
template class uint<waffle::StandardComposer, uint16_t>;
template class uint<waffle::StandardComposer, uint32_t>;
template class uint<waffle::StandardComposer, uint64_t>;

} // namespace stdlib
} // namespace plonk