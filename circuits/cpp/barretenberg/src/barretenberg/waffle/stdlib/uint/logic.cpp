#include "./uint.hpp"

#include "../../composer/mimc_composer.hpp"
#include "../../composer/standard_composer.hpp"
#include "../../composer/turbo_composer.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator&(const uint& other) const
{
    return logic_operator(other, LogicOp::AND);
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator^(const uint& other) const
{
    return logic_operator(other, LogicOp::XOR);
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator|(const uint& other) const
{
    return (*this + other) - (*this & other);
}

template <typename Composer, typename Native> uint<Composer, Native> uint<Composer, Native>::operator~() const
{
    if (!is_constant() && witness_status != WitnessStatus::NOT_NORMALIZED) {
        weak_normalize();
    }
    return uint(context, MASK) - *this;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator>>(const size_t shift) const
{
    if (shift >= width) {
        return uint(context, 0);
    }
    if (is_constant()) {
        return uint(context, additive_constant >> shift);
    }

    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    if (shift == 0) {
        return *this;
    }

    /**
     * bit shifts...
     *
     * We represent uints using a set of accumulating base-4 sums,
     * which adds complexity to bit shifting.
     *
     * Right shifts by even values are trivial - for a shift of 'x',
     * we return accumulator[(x - width - 1) / 2]
     *
     * Shifts by odd values are harder as we only have quads to work with.
     *
     * To recap accumulators. Our uint A can be described via a sum of its quads (a_0, ..., a_{width - 1})
     * (we use w as shorthand for 'width)
     *
     *      w - 1
     *      ===
     *      \          i
     * A =  /    a  . 4
     *      ===   i
     *     i = 0
     *
     * Our range constraint will represent A via its accumulating sums (A_0, ..., A_{w-1}), where
     *
     *         i
     *        ===
     *        \                             j
     * A   =  /    a                     . 4
     *  i     ===   ((w - 2/ 2) - i + j)
     *       j = 0
     *
     *
     * To compute (A >> x), we want the following value:
     *
     *    (w - x - 2) / 2
     *      ===
     *      \                 j
     * R =  /    a         . 4
     *      ===   (x + j)
     *     j = 0
     *
     *
     * From this, we can see that if x is even, R = A
     *                                               w - x - 1
     *
     * If x is odd, then we want to obtain the following:
     *
     *           (w - x - 2) / 2
     *             ===
     *             \                 j
     * R = b   2 . /    a         . 4
     *      x      ===   (x + j)
     *           j = 0
     *
     * Where b   is the most significant bit of A            - 4. A
     *        x                                  (x + 2) / 2       (x / 2)
     *
     * We have a special selector configuration in our arithmetic widget,
     * that will extract 6.b  from two accumulators for us.
     *                      x
     * The factor of 6 is for efficiency reasons,
     * we need to scale our other gate coefficients by 6 to accomodate this
     **/

    if ((shift & 1) == 0) {
        uint result(context);
        result.witness_index = accumulators[static_cast<size_t>(((width >> 1) - 1 - (shift >> 1)))];
        result.witness_status = WitnessStatus::WEAK_NORMALIZED;
        return result;
    }

    const uint256_t output = get_value() >> shift;

    // get accumulator index
    const size_t x = ((width >> 1) - 1 - (shift >> 1));

    // this >> shift = 2 * a[x - 1] + high bit of (a[x] - 4 * a[x - 1])
    // our add-with-bit-extract gate will pull out the high bit of ^^
    // if we place a[x] in column 3 and a[x - 1] in column 4
    // (but it actually extracts 6 * high_bit for efficiency reasons)
    // so we need to scale everything else accordingly
    const uint32_t right_index = accumulators[x];
    const uint32_t left_index = shift == 31 ? context->zero_idx : accumulators[static_cast<size_t>(x - 1)];

    const waffle::add_quad gate{
        context->zero_idx, context->add_variable(output),
        right_index,       left_index,
        fr::zero(),        -fr(6),
        fr::zero(),        fr(12),
        fr::zero(),
    };

    context->create_big_add_gate_with_bit_extraction(gate);

    uint result(context);
    result.witness_index = gate.b;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;

    return result;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator<<(const size_t shift) const
{
    if (shift >= width) {
        return uint(context, 0);
    }

    if (is_constant()) {
        return uint(context, (additive_constant << shift) & MASK);
    }

    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    if (shift == 0) {
        return *this;
    }

    if ((shift & 1) == 0) {
        const size_t x = (shift >> 1);
        const uint32_t right_idx = accumulators[x - 1];
        const uint32_t base_idx = witness_index;

        const uint256_t base_shift_factor = uint256_t(1) << (x * 2);
        const uint256_t right_shift_factor = uint256_t(1) << (width);

        const uint256_t output = (get_value() << shift) & MASK;

        const waffle::add_triple gate{
            base_idx,      right_idx, context->add_variable(output), base_shift_factor, -fr(right_shift_factor),
            fr::neg_one(), fr::zero()
        };

        context->create_add_gate(gate);

        uint result(context);
        result.witness_index = gate.c;
        result.witness_status = WitnessStatus::WEAK_NORMALIZED;
        return result;
    }

    const uint256_t output = (get_value() << shift) & MASK;

    // get accumulator index
    const size_t x = (shift >> 1);

    const uint32_t right_index = shift == 1 ? context->zero_idx : accumulators[static_cast<size_t>(x - 1)];
    const uint32_t left_index = accumulators[x];
    const uint32_t base_index = witness_index;

    const uint256_t base_shift_factor = ((uint256_t(1) << (x * 2 + 1))) * uint256_t(6);
    const uint256_t b_hi_shift_factor = CIRCUIT_UINT_MAX_PLUS_ONE;
    const uint256_t right_shift_factor = CIRCUIT_UINT_MAX_PLUS_ONE * uint256_t(12);

    fr q_1 = uint256_t(6);
    fr q_2 = base_shift_factor;
    fr q_3 = right_shift_factor;

    fr denominator = b_hi_shift_factor;
    denominator.self_neg();
    denominator = denominator.invert();

    q_1 *= denominator;
    q_2 *= denominator;
    q_3 *= denominator;

    const waffle::add_quad gate{
        context->add_variable(output), base_index, left_index, right_index, -q_1, q_2, fr::zero(), -q_3, fr::zero(),
    };

    context->create_big_add_gate_with_bit_extraction(gate);

    uint result(context);
    result.witness_index = gate.a;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;

    return result;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::ror(const size_t target_rotation) const
{
    const size_t rotation = target_rotation & (width - 1);

    const auto rotate = [](const uint256_t input, const uint64_t rot) {
        uint256_t r0 = (input >> rot);
        uint256_t r1 = (input << (width - rot)) & MASK;
        return (rot > 0) ? (r0 + r1) : input;
    };

    if (is_constant()) {
        return uint(context, rotate(additive_constant, rotation));
    }

    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    if (rotation == 0) {
        return *this;
    }

    const uint256_t output = rotate(get_value(), rotation);

    if ((rotation & 1) == 0) {
        const size_t x = (rotation >> 1);
        const size_t pivot = ((width >> 1) - 1 - x);
        const uint32_t left_idx = accumulators[pivot];
        const uint32_t base_idx = witness_index;

        const uint256_t t0 = (1ULL << (x * 2));
        const uint256_t t1 = (1ULL << ((width >> 1) - x) * 2);
        const uint256_t t2 = t0 * t1;

        const fr left_shift_factor = fr::one() - t2;
        const fr base_shift_factor = t1;

        const waffle::add_triple gate{ base_idx,          left_idx,          context->add_variable(output),
                                       base_shift_factor, left_shift_factor, fr::neg_one(),
                                       fr::zero() };

        context->create_add_gate(gate);

        uint result(context);
        result.witness_index = gate.c;
        result.witness_status = WitnessStatus::WEAK_NORMALIZED;
        return result;
    }

    const size_t x = (rotation >> 1) + 1;
    const size_t pivot = ((width >> 1) - 1 - x);

    const uint32_t pivot_idx = rotation == 31 ? context->zero_idx : accumulators[static_cast<size_t>(pivot)];
    const uint32_t next_pivot_idx = accumulators[static_cast<size_t>(pivot + 1)];
    const uint32_t base_idx = witness_index;

    const uint256_t out_scale_factor = uint256_t(6);
    const uint256_t base_scale_factor =
        (uint256_t(1) << ((uint256_t(width >> 1) - uint256_t(x)) << uint256_t(1)) + uint256_t(1)) * uint256_t(6);
    const uint256_t pivot_scale_factor = (uint256_t(1) << (uint256_t(width) + uint256_t(1))) * uint256_t(6);
    const uint256_t b_hi_scale_factor = (uint256_t(1) << uint256_t(width));

    fr q_1 = -fr(out_scale_factor);
    fr q_2 = base_scale_factor;
    constexpr fr twelve = fr{ 12, 0, 0, 0 }.to_montgomery_form();
    fr q_3 = twelve - pivot_scale_factor;

    fr denominator = fr::one() - b_hi_scale_factor;
    denominator = denominator.invert();

    q_1 *= denominator;
    q_2 *= denominator;
    q_3 *= denominator;

    const waffle::add_quad gate{
        context->add_variable(output), base_idx, next_pivot_idx, pivot_idx, q_1, q_2, fr::zero(), q_3, fr::zero(),
    };

    context->create_big_add_gate_with_bit_extraction(gate);

    uint result(context);
    result.witness_index = gate.a;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;

    return result;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::rol(const size_t target_rotation) const
{
    return ror(width - (target_rotation & (width - 1)));
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::logic_operator(const uint& other, const LogicOp op_type) const
{
    Composer* ctx = (context == nullptr) ? other.context : context;

    // we need to ensure that we can decompose our integers into (width / 2) quads
    // we don't need to completely normalize, however, as our quaternary decomposition will do that by default
    if (!is_constant() && witness_status == WitnessStatus::NOT_NORMALIZED) {
        weak_normalize();
    }
    if (!other.is_constant() && other.witness_status == WitnessStatus::NOT_NORMALIZED) {
        other.weak_normalize();
    }

    const uint256_t lhs = get_value();
    const uint256_t rhs = other.get_value();
    uint256_t out = 0;

    switch (op_type) {
    case AND: {
        out = lhs & rhs;
        break;
    }
    case XOR: {
        out = lhs ^ rhs;
        break;
    }
    default: {
    }
    }

    if (is_constant() && other.is_constant()) {
        return uint<Composer, Native>(ctx, out);
    }

    const uint32_t lhs_idx = is_constant() ? ctx->add_variable(lhs) : witness_index;
    const uint32_t rhs_idx = other.is_constant() ? ctx->add_variable(rhs) : other.witness_index;

    waffle::accumulator_triple logic_accumulators;

    switch (op_type) {
    case AND: {
        logic_accumulators = ctx->create_and_constraint(lhs_idx, rhs_idx, width);
        break;
    }
    case XOR: {
        logic_accumulators = ctx->create_xor_constraint(lhs_idx, rhs_idx, width);
        break;
    }
    default: {
    }
    }

    if (is_constant()) {
        const uint32_t constant_idx = ctx->put_constant_variable(additive_constant);
        ctx->assert_equal(lhs_idx, constant_idx);
    } else {
        accumulators = logic_accumulators.left;
        witness_index = accumulators[((width >> 1) - 1)];
        witness_status = WitnessStatus::OK;
    }

    if (other.is_constant()) {
        const uint32_t constant_idx = ctx->put_constant_variable(other.additive_constant);
        ctx->assert_equal(rhs_idx, constant_idx);
    } else {
        other.accumulators = logic_accumulators.right;
        other.witness_index = other.accumulators[(width >> 1) - 1];
        witness_status = WitnessStatus::OK;
    }

    uint<Composer, Native> result(ctx);
    result.accumulators = logic_accumulators.out;
    result.witness_index = result.accumulators[(width >> 1) - 1];
    result.witness_status = WitnessStatus::OK;
    return result;
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