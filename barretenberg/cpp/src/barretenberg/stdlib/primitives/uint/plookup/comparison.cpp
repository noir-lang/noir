#include "../../circuit_builders/circuit_builders.hpp"
#include "uint.hpp"

using namespace barretenberg;

namespace proof_system::plonk {
namespace stdlib {

template <typename Builder, typename Native>
bool_t<Builder> uint_plookup<Builder, Native>::operator>(const uint_plookup& other) const
{
    Builder* ctx = (context == nullptr) ? other.context : context;

    field_t<Builder> a(*this);
    field_t<Builder> b(other);
    bool result_witness = uint256_t(a.get_value()) > uint256_t(b.get_value());

    if (is_constant() && other.is_constant()) {
        return bool_t<Builder>(ctx, result_witness);
    }

    const bool_t<Builder> result = witness_t<Builder>(ctx, result_witness);

    /**
     * if (a > b), then (a - b - 1) will be in the range [0, 2**{width}]
     * if !(a > b), then (b - a) will be in the range [0, 2**{width}]
     * i.e. (a - b - 1)result + (b - a)(1 - result) should be positive
     **/
    const auto diff = a - b;
    const auto comparison_check =
        diff.madd(field_t<Builder>(result) * 2 - field_t<Builder>(1), -field_t<Builder>(result));

    comparison_check.create_range_constraint(width);

    return result;
}

template <typename Builder, typename Native>
bool_t<Builder> uint_plookup<Builder, Native>::operator<(const uint_plookup& other) const
{
    return other > *this;
}

template <typename Builder, typename Native>
bool_t<Builder> uint_plookup<Builder, Native>::operator>=(const uint_plookup& other) const
{
    return (!(other > *this)).normalize();
}

template <typename Builder, typename Native>
bool_t<Builder> uint_plookup<Builder, Native>::operator<=(const uint_plookup& other) const
{
    return (!(*this > other)).normalize();
}

template <typename Builder, typename Native>
bool_t<Builder> uint_plookup<Builder, Native>::operator==(const uint_plookup& other) const
{
    // casting to a field type will ensure that lhs / rhs are both normalized
    const field_t<Builder> lhs = static_cast<field_t<Builder>>(*this);
    const field_t<Builder> rhs = static_cast<field_t<Builder>>(other);

    return (lhs == rhs).normalize();
}

template <typename Builder, typename Native>
bool_t<Builder> uint_plookup<Builder, Native>::operator!=(const uint_plookup& other) const
{
    return (!(*this == other)).normalize();
}

template <typename Builder, typename Native> bool_t<Builder> uint_plookup<Builder, Native>::operator!() const
{
    return (field_t<Builder>(*this).is_zero()).normalize();
}

INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint8_t);
INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint16_t);
INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint32_t);
INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint64_t);
} // namespace stdlib
} // namespace proof_system::plonk