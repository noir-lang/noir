#include "../composers/composers.hpp"
#include "uint.hpp"

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

/**
 * @brief Return a uint which, if range constrained, would be proven to be the sum of `self` and `other`.
 *
 * @details Shorthand: write `a` for the uint `self`, and `b` for the uint `other`.
 *
 * The function gives part of a circuit allowing a prover to demonstrate knowledge of the remainder under division of
 * the integer (a_val + const_a) + (b_val + const_b) = a_val + b_val + (const_a + const_b) by 2^width. The desired
 * remainder is unchanged under subtraction of multiple of 2^width from the constant part const_a + const_b.
 * Therefore, for efficiency (to minimize the number of possible quotient values), we may (and do) replace the sum
 * const_a + const_b by c = (const_a + const_b) % 2**width.
 *
 * The function operator+ imposes the constraints
 *      a_val + b_val - 2^width * ov - rem + c == 0
 *   and
 *      ov in {0, 1, 2},
 * and returns rem.
 *
 * Note that, if `ov` were not constrained, there is the possibility of returning a malicious value of `rem`. The point
 * is that, if the constraints hold, then the constraint
 *      a_val + b_val - 2^width * (ov + x) - (rem - x*2^width) + c == 0
 * holds for any field element x. Suppose that rem is constrained to width-many bits for the widths we allow. Then,
 * if ov is in {0, 1, 2}, and also ov + x is in {0, 1, 2}, then rem - x*2^width will not be constrained to width-many
 * bits unless x = 0. Now suppose, instead, that ov is unconstrianed. If y is width-bit integer less than the field
 * modulus, then setting x = (rem - y)/2^width gives an equation proving that y is sum mod 2^width.
 *
 * @warning These constraints do not show that rem is actually the desired remainder, but if the prover also
 * normalizes rem (i.e., constrains it to width-many bits), then rem is shown to be the desired remainder.
 */

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator+(const uint& other) const
{
    ASSERT(context == other.context || (context != nullptr && other.context == nullptr) ||
           (context == nullptr && other.context != nullptr));
    Composer* ctx = (context == nullptr) ? other.context : context;

    if (is_constant() && other.is_constant()) {
        return uint<Composer, Native>(context, (additive_constant + other.additive_constant) & MASK);
    }

    if (is_constant() && !other.is_constant()) {
        uint<Composer, Native> result(other);
        result.additive_constant = (additive_constant + other.additive_constant) & MASK;
        result.witness_status = WitnessStatus::NOT_NORMALIZED;
        return result;
    }

    if (!is_constant() && other.is_constant()) {
        uint<Composer, Native> result(*this);
        result.additive_constant = (additive_constant + other.additive_constant) & MASK;
        result.witness_status = WitnessStatus::NOT_NORMALIZED;
        return result;
    }

    // Neither summand is constant.

    const uint256_t lhs = ctx->get_variable(witness_index);
    const uint256_t rhs = ctx->get_variable(other.witness_index);
    const uint256_t constants = (additive_constant + other.additive_constant) & MASK;
    const uint256_t sum = lhs + rhs + constants;
    const uint256_t overflow = sum >> width;
    const uint256_t remainder = sum & MASK;

    /**
     * Constraints:
     *   witness + other_witness - remainder - 2^width . overflow + constants == 0
     * and
     *   overflow lies in {0, 1, 2}
     * The sum L = witness + other_witness + constants is at most 3.(2^width-1). In the maximal case,
     * we have L = (2^w - 3) + 2*2^width, showing that a minimal range of values for the overflow
     * is {0, 1, 2}.
     **/

    const waffle::add_quad gate{
        .a = witness_index,
        .b = other.witness_index,
        .c = ctx->add_variable(remainder),
        .d = ctx->add_variable(overflow),
        .a_scaling = fr::one(),
        .b_scaling = fr::one(),
        .c_scaling = fr::neg_one(),
        .d_scaling = -fr(CIRCUIT_UINT_MAX_PLUS_ONE),
        .const_scaling = constants,
    };

    ctx->create_balanced_add_gate(gate);

    uint<Composer, Native> result(ctx);
    result.witness_index = gate.c;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;

    return result;
}

/**
 * @brief Return a uint which, if range constrained, would be proven to be the difference of `self` and `other`.
 *
 * @details Shorthand: write `a` for the uint `self`, and `b` for the uint `other`.
 *
 * The function gives part of a circuit allowing a prover to demonstrate knowledge of the remainder under division of
 * the integer (a_val + const_a) - (b_val + const_b) = a_val + b_val + (const_a - const_b) by 2^width. We apply two
 * transformations to this problem.
 *
 * 1) The desired remainder is unchanged under subtraction of multiples of 2^width from the constant part
 * const_a - const_b. Therefore, for efficiency (to minimize the number of possible quotient values), we replace
 * const_a - const_b by c = (const_a - const_b) % 2**width.
 *
 * 2) Set d' = a_val - b_val + c. Then the integer d' lies in the range [-2^{width} + 1, 2*2^{width} - 2], so the
 * integer d = d' + 2^width lies in [1, 3*2^width -2]. Therefore, there there is a unique equation of the form
 *    d = 2^width * ov + rem,  (ov in [0, 1, 2], rem in [0, 2^width - 1]).
 * Then a_val - b_val + (c + 2^width) = d = 2^width * ov + rem.
 *
 * The function operator- imposes the constraints
 *     a_val - b_val - 2^width * ov - rem + (c + 2^width) == 0
 *   and
 *     ov in {0, 1, 2},
 * and returns rem. These constraints do not show that rem is actually the desired remainder, but if the prover also
 * normalizes rem (i.e., constrains it to width-many bits), then rem is shown to be the desired remainder.
 */
template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator-(const uint& other) const
{
    // Assert contexts are equal unless one is null (e.g., from witness or uint without context)
    ASSERT(context == other.context || (context != nullptr && other.context == nullptr) ||
           (context == nullptr && other.context != nullptr));
    // If context is null, replace by other context.
    Composer* ctx = (context == nullptr) ? other.context : context;

    if (is_constant() && other.is_constant()) {
        return uint<Composer, Native>(context, (additive_constant - other.additive_constant) & MASK);
    }

    if (!is_constant() && witness_status == WitnessStatus::NOT_NORMALIZED) {
        weak_normalize();
    }
    if (!other.is_constant() && other.witness_status == WitnessStatus::NOT_NORMALIZED) {
        other.weak_normalize();
    }

    const uint32_t lhs_idx = is_constant() ? ctx->zero_idx : witness_index;
    const uint32_t rhs_idx = other.is_constant() ? ctx->zero_idx : other.witness_index;

    const uint256_t lhs = ctx->get_variable(lhs_idx);                                     // a
    const uint256_t rhs = ctx->get_variable(rhs_idx);                                     // b
    const uint256_t constant_term = (additive_constant - other.additive_constant) & MASK; // c

    const uint256_t difference = CIRCUIT_UINT_MAX_PLUS_ONE + lhs - rhs + constant_term; // d
    const uint256_t overflow = difference >> width;
    const uint256_t remainder = difference & MASK;

    // constraints:
    //   witness - other_witness - remainder - 2**width . overflow + (2**width + constant_term) == 0
    // and
    //   overflow in {0, 1, 2}
    const waffle::add_quad gate{
        .a = lhs_idx,
        .b = rhs_idx,
        .c = ctx->add_variable(remainder),
        .d = ctx->add_variable(overflow),
        .a_scaling = fr::one(),
        .b_scaling = fr::neg_one(),
        .c_scaling = fr::neg_one(),
        .d_scaling = -fr(CIRCUIT_UINT_MAX_PLUS_ONE),
        .const_scaling = CIRCUIT_UINT_MAX_PLUS_ONE + constant_term,
    };

    ctx->create_balanced_add_gate(gate);

    uint<Composer, Native> result(ctx);
    result.witness_index = gate.c;
    result.witness_status =
        (is_constant() || other.is_constant()) ? WitnessStatus::NOT_NORMALIZED : WitnessStatus::WEAK_NORMALIZED;

    return result;
}

/**
 * @brief Multiply two uint_ct's, trucating overflowing bits.
 */

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator*(const uint& other) const
{
    // Assert contexts are equal unless one us null (e.g., from witness or uint without context)
    ASSERT(context == other.context || (context != nullptr && other.context == nullptr) ||
           (context == nullptr && other.context != nullptr));

    Composer* ctx = (context == nullptr) ? other.context : context;

    if (is_constant() && other.is_constant()) {
        return uint<Composer, Native>(context, (additive_constant * other.additive_constant) & MASK);
    }

    if (is_constant() && !other.is_constant()) {
        return other * (*this);
    }

    /**
     * Notation: a = this,  const_a = this.additive_constant;
     *           b = other, const_b = other.additive_constant.
     * We are computing (a + const_a) * (b + const_b) modulo 2**width. We could record that we have
     * computed the long division of
     *      ab + b*const_a + a*const_b + const_a*const_b,
     * by 2**width and return the remainder. However, as in operator+, we trust ourselves to  correctly
     * reduce products of constants, so we instead record that we have computed the long division of
     *      ab + b*const_a + a*const_b + (const_a*const_b % 2**width).
     */

    const uint32_t rhs_idx = other.is_constant() ? ctx->zero_idx : other.witness_index;

    const uint256_t lhs = ctx->get_variable(witness_index); // a
    const uint256_t rhs = ctx->get_variable(rhs_idx);       // b

    const uint256_t constant_term =
        (additive_constant * other.additive_constant) & MASK; // (const _a*const_b % 2**width).
    const uint256_t product = (lhs * rhs) + (lhs * other.additive_constant) + (rhs * additive_constant) +
                              constant_term;     // the expression we will divide
    const uint256_t overflow = product >> width; // (product - remainder)/width
    const uint256_t remainder = product & MASK;  // remainder

    /**
     * constraint:
     *      ab + a const_b + b const_a - r - (2**width) overflow + (const_a const_b % 2**width) == 0
     */

    const waffle::mul_quad gate{
        .a = witness_index, // a
        .b = rhs_idx,       // b
        .c = ctx->add_variable(remainder),
        .d = ctx->add_variable(overflow),
        .mul_scaling = fr::one(),
        .a_scaling = other.additive_constant,
        .b_scaling = additive_constant,
        .c_scaling = fr::neg_one(),
        .d_scaling = -fr(CIRCUIT_UINT_MAX_PLUS_ONE), // 2**width
        .const_scaling = constant_term,
    };

    ctx->create_big_mul_gate(gate);
    constrain_accumulators(context, gate.d, width + 2, "arithmetic: uint mul overflow too large.");

    uint<Composer, Native> result(ctx);

    // Manually normalize the result. We do this here, and not for operator+, because
    // it is much easier to overflow the 252-bit modulus using multiplications. It is
    // left to the circuit writer keep track of overflows when using addition.
    result.accumulators = constrain_accumulators(ctx, gate.c, width, "arithmetic: uint mul remainder too large.");
    result.witness_index = result.accumulators[num_accumulators() - 1];
    result.witness_status = WitnessStatus::OK;

    return result;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator/(const uint& other) const
{
    return divmod(other).first;
}

template <typename Composer, typename Native>
uint<Composer, Native> uint<Composer, Native>::operator%(const uint& other) const
{
    return divmod(other).second;
}

/**
 *  @brief Return the pair of uints ((a / b), (a % b))
 *
 *  @details We impose the following constraints:
 *      - a and b are integers contrained to width-many bits (no new constraints if this is already established);
 *      - a = b.q + r
 *      - 0 <= r < b.
 * As integers, 0 <= r < b is equivalent to 0 <= r and 0 < b - r. Since the symbol r is in use, let q be the 254-bit
 * prime sometimes called r, i.e., the order of BN254. Let S be the subset of F_q consisting of all of the values range
 * constrained to width-many bits and their negatives. If width is a power of 2 at most 128, then S is the disjoint
 * union of [0, 2**width-1] (the positive elements) and [r-2**width, r-1] (the negations). Since we only allow widths up
 * to 64, we can use range constraints to impose non-negativity. Range constraining r, we have 0 <= r. Since b is range
 * constrained, range constraining b - r - 1 gives r < b.
 **/
template <typename Composer, typename Native>
std::pair<uint<Composer, Native>, uint<Composer, Native>> uint<Composer, Native>::divmod(const uint& other) const
{

    // Assert contexts are equal unless one us null (e.g., from witness or uint without context)
    ASSERT(context == other.context || (context != nullptr && other.context == nullptr) ||
           (context == nullptr && other.context != nullptr));

    Composer* ctx = (context == nullptr) ? other.context : context;

    // we need to guarantee that these values are 32 bits
    if (!is_constant() && witness_status != WitnessStatus::OK) {
        normalize();
    }
    if (!other.is_constant() && other.witness_status != WitnessStatus::OK) {
        other.normalize();
    }

    // We want to force the divisor b to be non-zero, as this is an error state
    if (other.is_constant() && other.get_value() == 0) {
        // ASSERT(other.get_value() != 0);
        // TODO: find some way of enabling the above assert that does not break
        //       stdlib_uint.test_divide_by_zero_fails.
        // impose failing constraint.
        field_t<Composer> one = field_t<Composer>::from_witness_index(context, 1);
        field_t<Composer> zero = field_t<Composer>::from_witness_index(context, 0);
        one / zero;
    } else if (!other.is_constant()) {
        const bool_t<Composer> is_divisor_zero = field_t<Composer>(other).is_zero();
        field_t<Composer>(is_divisor_zero).assert_equal(0); // here 0 is a circuit constant
    }

    // handle case of two constants and case of identical witness indices
    if (is_constant() && other.is_constant()) {
        const uint<Composer, Native> remainder(ctx, additive_constant % other.additive_constant);
        const uint<Composer, Native> quotient(ctx, additive_constant / other.additive_constant);
        return std::make_pair(quotient, remainder);
    } else if (witness_index == other.witness_index) {
        const uint<Composer, Native> remainder(context, 0);
        const uint<Composer, Native> quotient(context, 1);
        return std::make_pair(quotient, remainder);
    }

    const uint32_t dividend_idx = is_constant() ? ctx->zero_idx : witness_index;
    const uint32_t divisor_idx = other.is_constant() ? ctx->zero_idx : other.witness_index;

    const uint256_t dividend = get_unbounded_value();
    const uint256_t divisor = other.get_unbounded_value();

    // when divisor is zero, (q, r) is set to (0, 0), which only holds if dividend is zero.
    const uint256_t q = dividend / divisor;
    const uint256_t r = dividend % divisor;

    const uint32_t quotient_idx = ctx->add_variable(q);
    const uint32_t remainder_idx = ctx->add_variable(r);

    // constraint: qb + const_b q + 0 b - a + r - const_a == 0
    // i.e., a + const_a = q(b + const_b) + r
    const waffle::mul_quad division_gate{ .a = quotient_idx,  // q
                                          .b = divisor_idx,   // b
                                          .c = dividend_idx,  // a
                                          .d = remainder_idx, // r
                                          .mul_scaling = fr::one(),
                                          .a_scaling = other.additive_constant,
                                          .b_scaling = fr::zero(),
                                          .c_scaling = fr::neg_one(),
                                          .d_scaling = fr::one(),
                                          .const_scaling = -fr(additive_constant) };
    ctx->create_big_mul_gate(division_gate);

    // set delta = (b + const_b - r - 1)
    const uint256_t delta = divisor - r - 1;
    const uint32_t delta_idx = ctx->add_variable(delta);

    // constraint: b - r - delta + const_b - 1 == 0
    const waffle::add_triple delta_gate{ .a = divisor_idx,
                                         .b = remainder_idx,
                                         .c = delta_idx,
                                         .a_scaling = fr::one(),
                                         .b_scaling = fr::neg_one(),
                                         .c_scaling = fr::neg_one(),
                                         .const_scaling = other.additive_constant + fr::neg_one() };

    ctx->create_add_gate(delta_gate);

    // validate delta is in the correct range
    ctx->decompose_into_base4_accumulators(delta_idx, width, "arithmetic: divmod delta range constraint fails.");

    // normalize witness quotient and remainder
    // minimal bit range for quotient: from 0 (in case a = b-1) to width (when b = 1).
    uint<Composer, Native> quotient(ctx);
    quotient.accumulators = ctx->decompose_into_base4_accumulators(
        quotient_idx, width, "arithmetic: divmod quotient range constraint fails.");
    quotient.witness_index = quotient.accumulators[(width >> 1) - 1];
    quotient.witness_status = WitnessStatus::OK;

    // constrain remainder to lie in [0, 2^width-1]
    uint<Composer, Native> remainder(ctx);
    remainder.accumulators = ctx->decompose_into_base4_accumulators(
        remainder_idx, width, "arithmetic: divmod remaidner range constraint fails.");
    remainder.witness_index = remainder.accumulators[(width >> 1) - 1];
    remainder.witness_status = WitnessStatus::OK;

    return std::make_pair(quotient, remainder);
}

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