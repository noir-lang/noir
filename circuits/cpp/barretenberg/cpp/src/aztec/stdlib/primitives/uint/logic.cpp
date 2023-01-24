#include "../composers/composers.hpp"
#include "uint.hpp"

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

/**
 * @brief Right shift a uint.
 *
 * @details Note that the result is only weakly normalized.
 */
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
     * We represent uints using a set of accumulating base-4 sums, which adds complexity to bit shifting.
     *
     * Right shifts by even values are trivial because of how our accumulators are defined. Shifts by odd
     * values are harder as we only have quads to work with.
     *
     * To recap accumulators. Our uint A can be described via a sum of its quads (a_0, ..., a_{width - 1})
     * (we use w as shorthand for 'width / 2' and assume the width is even.)
     *
     *        w - 1
     *        ===
     *        \          i
     *   A =  /    a  . 4
     *        ===   i
     *       i = 0
     *
     * Our range constraint will represent A via its accumulating sums (A_0, ..., A_{w-1}), where
     *
     *           i                                    |       ~NOTE~       :           i
     *          ===                                   | this is equivalent :         ===
     *          \                             j       |   to the formula   :         \                  i - j
     *   A   =  /    a                     . 4        |      given in      :  a   =  /    a         .  4
     *    i     ===   (w - 1 - i + j)                 | turbo_composer.cpp :    i     ===   (15 - j)
     *         j = 0                                  |   when width = 32  :        j = 0
     *
     *
     * Write x = 2y + z with z in {0,1}. Let
     *
     *               w - 1 - y
     *                 ===
     *                 \                 j
     *   R = A/4**y =  /    a         . 4
     *                 ===   (y + j)
     *                j = 0
     *
     *
     * We can see that if x is even, (A>>x) = A              .
     *                                          w - 1 - (x/2)
     *
     * If x is odd, then first over-shift to calculate A>>(x+1), then correct by doubling and adding the
     * needed low bit. The first shift discards, in order, the quads a_0, ..., a_{(x+1)/2 - 1}, so the needed
     * low bit is the high bit of a_y. Defining A_{-1} = 0, we have a_{w-1-i} = A_{i} - 4.A_{i-1}, i >= 0.
     * Therefore,
     *
     *   (A>>x) = b   + 2 . A
     *             x         w - 1 - ((x+1)/2).
     *
     * where b   is the most significant bit of A            - 4. A
     *        x                                  w - ((x+1)/2)        w - 1 - ((x+1)/2)
     *
     *
     * We have a special selector configuration in our arithmetic widget that extracts 6.b_x from given the two
     * relevant accumulators. The factor of 6 is for efficiency reasons.  We need to scale our other gate
     * coefficients by 6 to accomodate this.
     **/

    if ((shift & 1) == 0) {
        uint result(context);
        result.witness_index = accumulators[static_cast<size_t>(((width >> 1) - 1 - (shift >> 1)))];
        result.witness_status = WitnessStatus::WEAK_NORMALIZED;
        return result;
    }

    const uint256_t output = get_value() >> shift;

    // get accumulator index
    const size_t idx = ((width >> 1) - 1 - (shift >> 1)); // w - 1 - ((x-1)/2)) = w - (1 + ((x-1)/2))) = x - ((x+1)/2)

    // this >> shift = 2 * a[idx - 1] + high bit of (a[idx] - 4 * a[idx - 1])
    // our add-with-bit-extract gate will pull out the high bit of ^^
    // if we place a[idx] in column 3 and a[idx - 1] in column 4
    // (but it actually extracts 6 * high_bit for efficiency reasons)
    // so we need to scale everything else accordingly
    const uint32_t right_index = accumulators[idx]; // idx is w - ((x+1)/2) above
    const uint32_t left_index = shift == width - 1 ? context->zero_idx : accumulators[static_cast<size_t>(idx - 1)];

    // Constraint: -6.(self >> shift) + 12.a[idx-1] + 6.high bit of (a[idx] - 4.a[idx-1]) = 0.
    const waffle::add_quad gate{
        .a = context->zero_idx,
        .b = context->add_variable(output),
        .c = right_index,
        .d = left_index,
        .a_scaling = fr::zero(),
        .b_scaling = -fr(6),
        .c_scaling = fr::zero(),
        .d_scaling = fr(12),
        .const_scaling = fr::zero(),
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

    // even case
    if ((shift & 1) == 0) {
        /**
         * Shift by an even number of bits s = 2x, keeping the lowest width-many bits.
         * Assume width is even and let w = width / 2. Let s = shift. Then we can express A << s
         * in terms of the accumulator decomposition of A by writing
         *                     w-1                         x-1
         *                     ===                         ===
         *                     \             j             \              k
         *  A << s = A.4**x =  /     a      4   +    4**w  /     a       4
         *                     ===     j-x                 ===     w-x+k
         *                    j = x                       k = 0
         *
         *                   = (value * 2**shift) % 2**32 + 2**width . A_{x-1}
         */
        const size_t x = (shift >> 1); // A << shift is A .4^x
        const uint32_t base_idx = witness_index;
        const uint32_t right_idx = accumulators[x - 1]; // A_{x-1}

        const uint256_t base_shift_factor = uint256_t(1) << (x * 2);  // 2**shift
        const uint256_t right_shift_factor = uint256_t(1) << (width); // 2**width

        const uint256_t output = (get_value() << shift) & MASK; // (value * 2**shift) % 2**32

        // constraint: A.2**shift - A_{x-1}.2**width + (value * 2**shift) % 2**32 == 0
        const waffle::add_triple gate{ .a = base_idx,
                                       .b = right_idx,
                                       .c = context->add_variable(output),
                                       .a_scaling = base_shift_factor,
                                       .b_scaling = -fr(right_shift_factor),
                                       .c_scaling = fr::neg_one(),
                                       .const_scaling = fr::zero() };

        context->create_add_gate(gate);

        uint result(context);
        result.witness_index = gate.c;
        result.witness_status = WitnessStatus::WEAK_NORMALIZED;
        return result;
    }

    // odd case
    /**
     * Now s = 2x + 1. As above, we compute 2**s and subtract part outside of the lowest
     * width-many bits. Again let w = width / 2 and s = shift.
     * Illustration:
     * A = [ w-1+x wi-2+x ... wi+2 wi+1 | wi | wi-1  ... 2x+2 2x+1 ... 0 ]
     *          \   /          \     /     \     /         \   /
     *           a_{w-1}       a_{w-x}    a_{w-1-x}         a_0
     *
     * Then bits width+1 through width-1+x of the shift A*2**s are given by
     *                                            w+x-1
     *                                             ===
     *                                             \             j
     * 2**{width+1}.A_{x-1} = 2*4**{w}A_{x-1} = 2  /     a      4
     *                                             ===     j-x       .
     *                                            j = w
     * Only the bit at position equal width is unaccounted-for. This bit is the
     * high bit of a_{w-1-x} = A_x - 4.A_{x-1}, which can be accessed through our addition gate
     * with bit extraction.  Altogether, we would like to impose that
     *         s        width+1           width
     * out  = 2 . A - 2         . A    - 2     (high bit of A  - 4A    )
     *                             x-1         (             x     x-1 )
     *
     * Since our gate with bit extraction adds the term 6 * (high bit of w_0 - 4.w_4), we scale
     * the entire equation by -6/2**width.
     */
    const uint256_t output = (get_value() << shift) & MASK;

    // get accumulator index
    const size_t x = (shift >> 1);

    const uint32_t right_index = shift == 1 ? context->zero_idx : accumulators[static_cast<size_t>(x - 1)];
    const uint32_t left_index = accumulators[x];
    const uint32_t base_index = witness_index;

    const uint256_t base_shift_factor = ((uint256_t(1) << (x * 2 + 1))) * uint256_t(6); // 6 . 2**shift
    const uint256_t b_hi_shift_factor = CIRCUIT_UINT_MAX_PLUS_ONE;                      // 2**width
    const uint256_t right_shift_factor = CIRCUIT_UINT_MAX_PLUS_ONE * uint256_t(12);     // 12 . 2**width

    fr q_1 = uint256_t(6);
    fr q_2 = base_shift_factor;
    fr q_3 = right_shift_factor;

    fr denominator = b_hi_shift_factor;
    denominator.self_neg();
    denominator = denominator.invert();

    q_1 *= denominator; // - 6/2**width
    q_2 *= denominator; // - 6/2**width . 2**shift
    q_3 *= denominator; // -12/2**width . 2**width

    /**
     * constraint:
     * -(-6/2**width) output + (-6/2**width) . 2**s . A + 0 A_x - (-12/2**width) 2**width A_{x-1}
     *                       + 6 * high bit of A_x - 4 A_{x-1}                                   ,
     * where A = witness and shift = s = 2x + 1 and A_{-1} = 0.
     */
    const waffle::add_quad gate{ .a = context->add_variable(output),
                                 .b = base_index,
                                 .c = left_index,
                                 .d = right_index,
                                 .a_scaling = -q_1,
                                 .b_scaling = q_2,
                                 .c_scaling = fr::zero(),
                                 .d_scaling = -q_3,
                                 .const_scaling = fr::zero() };

    context->create_big_add_gate_with_bit_extraction(gate);

    uint result(context);
    result.witness_index = gate.a;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;

    return result;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::ror(const size_t target_rotation) const
{
    // reduce rotation modulo width (width is always assumed a power of 2)
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

    // case of rotation by an even number of bits
    if ((rotation & 1) == 0) {
        /**
         * Rotate A = \sum_{i=0}^{width-1} a_i.4^i rightward by an even number r = 2x of bits,
         * yielding output A'. Assume width is even and let w = width / 2. Since r is even, rotating
         * A is equivalent to rotate half as many of the quads a_i.
         * Illustration in terms of quads:
         *       A  = [a_{w-1}  ... a_{x}  a_{x-1} ... a_{0}]
         *   ~~> A' = [a_{x-1}  ... a_{0}  a_{w-1} ... a_{x}]
         * We get the higest x quads of A' from the lowest x quads of 4**{w-x} A, and use an appropriate
         * accumulator to add in the lowest w-x quads to A' and to remove the highest w-x quads from
         * 4**{w-x} A. Since the accumulator A_i encodes the highest i+1 bits of A, we arrive at the formula
         *   A' = 4**{w-x} A + (1 - 4**w) A_{w-x}
         */
        const size_t x = (rotation >> 1);              // x = r/2 with r even
        const size_t pivot = ((width >> 1) - 1 - x);   // pivot = w - 1 - x
        const uint32_t left_idx = accumulators[pivot]; // A_pivot
        const uint32_t base_idx = witness_index;       // A

        const uint256_t t0 = (1ULL << (x * 2));                // 4^x
        const uint256_t t1 = (1ULL << ((width >> 1) - x) * 2); // 4^{w-x}
        const uint256_t t2 = t0 * t1;                          // 4^w

        const fr left_shift_factor = fr::one() - t2;
        const fr base_shift_factor = t1;

        // constraint: 4^{w-x} A + (1 - 4^w) A_pivot - out == 0
        const waffle::add_triple gate{ .a = base_idx,
                                       .b = left_idx,
                                       .c = context->add_variable(output),
                                       .a_scaling = base_shift_factor,
                                       .b_scaling = left_shift_factor,
                                       .c_scaling = fr::neg_one(),
                                       .const_scaling = fr::zero() };

        context->create_add_gate(gate);

        uint result(context);
        result.witness_index = gate.c;
        result.witness_status = WitnessStatus::WEAK_NORMALIZED;
        return result;
    }

    // case of rotation by an odd number of bits
    /**
     * Now rotate A by an even number r = 2(x-1)+1 of bits, yielding output A'.
     * Let a_j = a_{j,0} + 2a_{j, 1} be the bit decomposition of a_j, j = 0, ... , w-1.
     * Illustration in terms of bits:
     *       A  = [ a_{w-1,1} ... a_{x,0} a_{x-1,1} |  a_{x-2,0} ... a_{0,1} | a_{0,  0}  ]
     *   ~~> A' = [ a_{x-1,0} ... a_{0,1} a_{0,  0} |  a_{w-1,1} ... a_{x.0} | a_{x-1,1}  ]
     *            [--------from 2 * 4^{w-x} A-------|-----from A_{w-x-1}-----|--extracted-]
     * As in the even case, the high bits of A' come from a shift of A, while an accumulator
     * contributes the lower bits while removing bits of the shift of A beyond width.
     * This handles all but two bits: the 0th bit of A', and one dangling bit coming from the
     * shift of A. These are handled using the bit extraction gate. Recall that the accumulators
     * A_i satisfy a_{w-1-i} = A_{i} - 4.A_{i-1}, i >= 0, when we set A_{-1} = 0. We see that
     * to extract a_{x-1, 1}, we should set i = w - x.
     */
    const size_t x = (rotation >> 1) + 1;        // x = (r-1)/2 + 1 with r odd
    const size_t pivot = ((width >> 1) - 1 - x); // pivot = w - 1 - x

    const uint32_t pivot_idx =
        rotation == width - 1 ? context->zero_idx : accumulators[static_cast<size_t>(pivot)]; // A_{w-x-1}
    const uint32_t next_pivot_idx = accumulators[static_cast<size_t>(pivot + 1)];             // A_{w-x}
    const uint32_t base_idx = witness_index;                                                  // A

    const uint256_t out_scale_factor = uint256_t(6);
    const uint256_t base_scale_factor =
        (uint256_t(1) << ((uint256_t(width >> 1) - uint256_t(x)) << uint256_t(1)) + uint256_t(1)) *
        uint256_t(6); // 6*2*4^{w-x}
    const uint256_t pivot_scale_factor = (uint256_t(1) << (uint256_t(width) + uint256_t(1))) * uint256_t(6); // 6*2*4^w
    const uint256_t b_hi_scale_factor = (uint256_t(1) << uint256_t(width));                                  // 4^w

    fr q_1 = -fr(out_scale_factor);
    fr q_2 = base_scale_factor;
    constexpr fr twelve = fr{ 12, 0, 0, 0 }.to_montgomery_form();
    fr q_3 = twelve - pivot_scale_factor;

    fr denominator = fr::one() - b_hi_scale_factor;
    denominator = denominator.invert(); // 1/(1-4^w)

    q_1 *= denominator; // -6/(1 - 4^w)
    q_2 *= denominator; //  (6*2*4^{w-x})/(1 - 4^w)
    q_3 *= denominator; //  (12 - 6*2*4^{w})/(1 - 4^w)

    /**
     * constraint:
     *                            w-x                                        w            /                      \
     *     6             6 . 2 . 4                            12 - 6 . 2 . 4             | 6 * the highest bit of |
     * - ------  out  +  ------------ A  +  0 . A          +  ----------------- A      + | A        - 4 A         |
     *        w                  w               pivot + 1               w       pivot   |  pivot+1      pivot    |
     *   1 - 4              1 - 4                                   1 - 4                 \                      /
     * or, simplified,
     *   -out + (2 * 4^{w-x}) A + (2 - 2 * 4^{w}) A_{w-x-1} + (1 - 4^w) a_{x-1, 1} == 0.
     *
     */
    const waffle::add_quad gate{ .a = context->add_variable(output),
                                 .b = base_idx,
                                 .c = next_pivot_idx,
                                 .d = pivot_idx,
                                 .a_scaling = q_1,
                                 .b_scaling = q_2,
                                 .c_scaling = fr::zero(),
                                 .d_scaling = q_3,
                                 .const_scaling = fr::zero() };

    context->create_big_add_gate_with_bit_extraction(gate);

    uint result(context);
    result.witness_index = gate.a;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;

    return result;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::rol(const size_t target_rotation) const
{
    // Rotating right by r in [0, w - 1] is the same as rotating left by w - r
    // In general, set r = target_rotation mod width = target_rotation & (width - 1),
    // width being assumed a power of 2.
    return ror(width - (target_rotation & (width - 1)));
}

/**
 * @brief Implement AND and XOR.
 */
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
        // returns a constant uint.
        return uint<Composer, Native>(ctx, out);
    }

    // // PLOOKUP implementation is not being audited.
    // if constexpr (Composer::type == waffle::PLOOKUP) {
    //     std::array<std::vector<field_t<Composer>>, 3> sequence;
    //     if (op_type == XOR) {
    //         sequence = plookup::read_sequence_from_table(
    //             waffle::PlookupMultiTableId::UINT32_XOR, field_t<Composer>(*this), field_t<Composer>(other), true);
    //     } else {
    //         sequence = plookup::read_sequence_from_table(
    //             waffle::PlookupMultiTableId::UINT32_AND, field_t<Composer>(*this), field_t<Composer>(other), true);
    //     }
    //     uint<Composer, Native> result(ctx);
    //     for (size_t i = 0; i < num_accumulators(); ++i) {
    //         result.accumulators.emplace_back(sequence[2][num_accumulators() - 1 - i].witness_index);
    //     }
    //     result.witness_index = result.accumulators[num_accumulators() - 1];
    //     result.witness_status = WitnessStatus::OK;
    //     return result;
    // }

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
        field_t<Composer>::from_witness_index(ctx, lhs_idx)
            .assert_equal(additive_constant, "uint logic operator assert equal fail");
    } else {
        accumulators = logic_accumulators.left;
        witness_index = accumulators[num_accumulators() - 1];
        witness_status = WitnessStatus::OK;
    }

    if (other.is_constant()) {
        field_t<Composer>::from_witness_index(ctx, rhs_idx)
            .assert_equal(other.additive_constant, "uint logic operator assert equal fail");
    } else {
        other.accumulators = logic_accumulators.right;
        other.witness_index = other.accumulators[num_accumulators() - 1];
        witness_status = WitnessStatus::OK;
    }

    uint<Composer, Native> result(ctx);
    result.accumulators = logic_accumulators.out;
    result.witness_index = result.accumulators[num_accumulators() - 1];
    result.witness_status = WitnessStatus::OK;
    return result;
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