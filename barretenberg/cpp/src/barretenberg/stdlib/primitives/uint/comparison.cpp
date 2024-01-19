#include "../circuit_builders/circuit_builders.hpp"
#include "uint.hpp"

using namespace bb;

namespace bb::stdlib {

template <typename Builder, typename Native>
/**
 * @brief Determine whether this > other.
 *
 * @details This allows a prover to demonstrate that they have correctly classified a, b
 * as satisfying either a > b or a <= b.
 *
 */
bool_t<Builder> uint<Builder, Native>::operator>(const uint& other) const
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
     * The field_t operator on uints normalizes its input, so a and be have
     * been constrained to the width of Native. That is, both a and b
     * lie in the closed interval [0, 2*{width} - 1]. Now,
     *    (a > b) <==>  (a - b - 1) is in the range [0, 2**{width} - 2]
     * and
     *   !(a > b) <==>  (b - a)     is in the range [0, 2**{width} - 1].
     * Consider
     *   comparison_check = (a - b - 1)result + (b - a)(1 - result).
     * If comparison_check is in the range [0, 2**{width} - 1] and result is boolean,
     * then we are left with three possibilities:
     *   (1) a - b - 1 = 2**{width} - 1
     *   (2) a > b
     *   (3) !(a > b)
     * The bool_t operator on witnesses applies the relevant constraint to result, so we are
     * left to eliminate possibility (1). The difference a - b is calculated relative to the
     * circuit modulus r. The number D of distinct Fr elements that can be written this
     * way is at most M = 2*(2**{width}-1) + 1 = 2**{width+1} - 1, and in fact, D = M if
     * r > M. Since our r has 252 bits, it suffices to ensure that 2**252 >= M.
     * Altogether, as long as 252 > width, 2**width cannot be written as the additive inverse
     * of a of  that width.
     **/

    const auto diff = a - b;
    // diff.result - result + diff.result - diff = diff.(2.result - 1) - result
    const auto comparison_check =
        diff.madd(field_t<Builder>(result) * 2 - field_t<Builder>(1), -field_t<Builder>(result));

    ctx->decompose_into_base4_accumulators(
        comparison_check.witness_index, width, "comparison: uint comparison range constraint fails.");

    return result;
}

template <typename Builder, typename Native> bool_t<Builder> uint<Builder, Native>::operator<(const uint& other) const
{
    return other > *this;
}

template <typename Builder, typename Native> bool_t<Builder> uint<Builder, Native>::operator>=(const uint& other) const
{
    return (!(other > *this)).normalize();
}

template <typename Builder, typename Native> bool_t<Builder> uint<Builder, Native>::operator<=(const uint& other) const
{
    return (!(*this > other)).normalize();
}

template <typename Builder, typename Native> bool_t<Builder> uint<Builder, Native>::operator==(const uint& other) const
{
    // casting to a field type will ensure that lhs / rhs are both normalized
    const field_t<Builder> lhs = static_cast<field_t<Builder>>(*this);
    const field_t<Builder> rhs = static_cast<field_t<Builder>>(other);

    return (lhs == rhs).normalize();
}

template <typename Builder, typename Native> bool_t<Builder> uint<Builder, Native>::operator!=(const uint& other) const
{
    return (!(*this == other)).normalize();
}

template <typename Builder, typename Native> bool_t<Builder> uint<Builder, Native>::operator!() const
{
    // return true if this is zero, otherwise return false.
    return (field_t<Builder>(*this).is_zero()).normalize();
}

template class uint<bb::StandardCircuitBuilder, uint8_t>;
template class uint<bb::StandardCircuitBuilder, uint16_t>;
template class uint<bb::StandardCircuitBuilder, uint32_t>;
template class uint<bb::StandardCircuitBuilder, uint64_t>;
} // namespace bb::stdlib