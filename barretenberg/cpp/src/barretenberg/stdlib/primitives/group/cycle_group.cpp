#include "../field/field.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

#include "./cycle_group.hpp"
#include "barretenberg/proof_system/plookup_tables/types.hpp"
#include "barretenberg/stdlib/primitives/plookup/plookup.hpp"
namespace bb::stdlib {

template <typename Composer>
cycle_group<Composer>::cycle_group(Composer* _context)
    : x(0)
    , y(0)
    , _is_infinity(true)
    , _is_constant(true)
    , context(_context)
{}

/**
 * @brief Construct a new cycle group<Composer>::cycle group object
 *
 * @param _x
 * @param _y
 * @param is_infinity
 */
template <typename Composer>
cycle_group<Composer>::cycle_group(field_t _x, field_t _y, bool_t is_infinity)
    : x(_x.normalize())
    , y(_y.normalize())
    , _is_infinity(is_infinity)
    , _is_constant(_x.is_constant() && _y.is_constant() && is_infinity.is_constant())
{
    if (_x.get_context() != nullptr) {
        context = _x.get_context();
    } else if (_y.get_context() != nullptr) {
        context = _y.get_context();
    } else {
        context = is_infinity.get_context();
    }
}

/**
 * @brief Construct a new cycle group<Composer>::cycle group object
 *
 * @details is_infinity is a circuit constant. We EXPLICITLY require that whether this point is infinity/not infinity is
 * known at circuit-construction time *and* we know this point is on the curve. These checks are not constrained.
 * Use from_witness if these conditions are not met.
 * Examples of when conditions are met: point is a derived from a point that is on the curve + not at infinity.
 * e.g. output of a doubling operation
 * @tparam Composer
 * @param _x
 * @param _y
 * @param is_infinity
 */
template <typename Composer>
cycle_group<Composer>::cycle_group(const FF& _x, const FF& _y, bool is_infinity)
    : x(_x)
    , y(_y)
    , _is_infinity(is_infinity)
    , _is_constant(true)
    , context(nullptr)
{
    ASSERT(get_value().on_curve());
}

/**
 * @brief Construct a cycle_group object out of an AffineElement object
 *
 * @note This produces a circuit-constant object i.e. known at compile-time, no constraints.
 *       If `_in` is not fixed for a given circuit, use `from_witness` instead
 *
 * @tparam Composer
 * @param _in
 */
template <typename Composer>
cycle_group<Composer>::cycle_group(const AffineElement& _in)
    : x(_in.x)
    , y(_in.y)
    , _is_infinity(_in.is_point_at_infinity())
    , _is_constant(true)
    , context(nullptr)
{}

/**
 * @brief Converts an AffineElement into a circuit witness.
 *
 * @details Somewhat expensive as we do an on-curve check and `_is_infinity` is a witness and not a constant.
 *          If an element is being converted where it is known the element is on the curve and/or cannot be point at
 *          infinity, it is best to use other methods (e.g. direct conversion of field_t coordinates)
 *
 * @tparam Composer
 * @param _context
 * @param _in
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::from_witness(Composer* _context, const AffineElement& _in)
{
    cycle_group result(_context);
    result.x = field_t(witness_t(_context, _in.x));
    result.y = field_t(witness_t(_context, _in.y));
    result._is_infinity = bool_t(witness_t(_context, _in.is_point_at_infinity()));
    result._is_constant = false;
    result.validate_is_on_curve();
    return result;
}

/**
 * @brief Converts a native AffineElement into a witness, but constrains the witness values to be known constants.
 *
 * @details When performing group operations where one operand is a witness and one is a constant,
 * it can be more efficient to convert the constant element into a witness. This is because we have custom gates
 * that evaluate additions in one constraint, but only if both operands are witnesses.
 *
 * @tparam Composer
 * @param _context
 * @param _in
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::from_constant_witness(Composer* _context, const AffineElement& _in)
{
    cycle_group result(_context);
    result.x = field_t(witness_t(_context, _in.x));
    result.y = field_t(witness_t(_context, _in.y));
    result.x.assert_equal(_in.x);
    result.y.assert_equal(_in.y);
    // point at infinity is circuit constant
    result._is_infinity = _in.is_point_at_infinity();
    result._is_constant = false;
    return result;
}

template <typename Composer> Composer* cycle_group<Composer>::get_context(const cycle_group& other) const
{
    if (get_context() != nullptr) {
        return get_context();
    }
    return other.get_context();
}

template <typename Composer> typename cycle_group<Composer>::AffineElement cycle_group<Composer>::get_value() const
{
    AffineElement result(x.get_value(), y.get_value());
    if (is_point_at_infinity().get_value()) {
        result.self_set_infinity();
    }
    return result;
}

/**
 * @brief On-curve check.
 *
 * @tparam Composer
 */
template <typename Composer> void cycle_group<Composer>::validate_is_on_curve() const
{
    // This class is for short Weierstrass curves only!
    static_assert(Group::curve_a == 0);
    auto xx = x * x;
    auto xxx = xx * x;
    auto res = y.madd(y, -xxx - Group::curve_b);
    res *= is_point_at_infinity();
    res.assert_is_zero();
}

/**
 * @brief Evaluates a doubling. Does not use Ultra double gate
 *
 * @tparam Composer
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::dbl() const
    requires IsNotUltraArithmetic<Composer>
{
    auto modified_y = field_t::conditional_assign(is_point_at_infinity(), 1, y);
    auto lambda = (x * x * 3) / (modified_y + modified_y);
    auto x3 = lambda.madd(lambda, -x - x);
    auto y3 = lambda.madd(x - x3, -modified_y);
    return cycle_group(x3, y3, is_point_at_infinity());
}

/**
 * @brief Evaluates a doubling. Uses Ultra double gate
 *
 * @tparam Composer
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::dbl() const
    requires IsUltraArithmetic<Composer>
{
    // ensure we use a value of y that is not zero. (only happens if point at infinity)
    // this costs 0 gates if `is_infinity` is a circuit constant
    auto modified_y = field_t::conditional_assign(is_point_at_infinity(), 1, y).normalize();
    auto x1 = x.get_value();
    auto y1 = modified_y.get_value();

    // N.B. the formula to derive the witness value for x3 mirrors the formula in elliptic_relation.hpp
    // Specifically, we derive x^4 via the Short Weierstrass curve formula `y^2 = x^3 + b`
    // i.e. x^4 = x * (y^2 - b)
    // We must follow this pattern exactly to support the edge-case where the input is the point at infinity.
    auto y_pow_2 = y1.sqr();
    auto x_pow_4 = x1 * (y_pow_2 - Group::curve_b);
    auto lambda_squared = (x_pow_4 * 9) / (y_pow_2 * 4);
    auto lambda = (x1 * x1 * 3) / (y1 + y1);
    auto x3 = lambda_squared - x1 - x1;
    auto y3 = lambda * (x1 - x3) - y1;
    if (is_constant()) {
        return cycle_group(x3, y3, is_point_at_infinity().get_value());
    }
    cycle_group result(witness_t(context, x3), witness_t(context, y3), is_point_at_infinity());
    context->create_ecc_dbl_gate(bb::ecc_dbl_gate_<FF>{
        .x1 = x.get_witness_index(),
        .y1 = modified_y.normalize().get_witness_index(),
        .x3 = result.x.get_witness_index(),
        .y3 = result.y.get_witness_index(),
    });
    return result;
}

/**
 * @brief Will evaluate ECC point addition over `*this` and `other`.
 *        Incomplete addition formula edge cases are *NOT* checked!
 *        Only use this method if you know the x-coordinates of the operands cannot collide
 *        Standard version that does not use ecc group gate
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::unconditional_add(const cycle_group& other) const
    requires IsNotUltraArithmetic<Composer>
{
    auto x_diff = other.x - x;
    auto y_diff = other.y - y;
    // unconditional add so do not check divisor is zero
    // (this also makes it much easier to test failure cases as this does not segfault!)
    auto lambda = y_diff.divide_no_zero_check(x_diff);
    auto x3 = lambda.madd(lambda, -other.x - x);
    auto y3 = lambda.madd(x - x3, -y);
    cycle_group result(x3, y3, false);
    return result;
}

/**
 * @brief Will evaluate ECC point addition over `*this` and `other`.
 *        Incomplete addition formula edge cases are *NOT* checked!
 *        Only use this method if you know the x-coordinates of the operands cannot collide
 *        Ultra version that uses ecc group gate
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::unconditional_add(const cycle_group& other) const
    requires IsUltraArithmetic<Composer>
{
    auto context = get_context(other);

    const bool lhs_constant = is_constant();
    const bool rhs_constant = other.is_constant();
    if (lhs_constant && !rhs_constant) {
        auto lhs = cycle_group::from_constant_witness(context, get_value());
        return lhs.unconditional_add(other);
    }
    if (!lhs_constant && rhs_constant) {
        auto rhs = cycle_group::from_constant_witness(context, other.get_value());
        return unconditional_add(rhs);
    }

    const auto p1 = get_value();
    const auto p2 = other.get_value();
    AffineElement p3(Element(p1) + Element(p2));
    if (lhs_constant && rhs_constant) {
        return cycle_group(p3);
    }
    field_t r_x(witness_t(context, p3.x));
    field_t r_y(witness_t(context, p3.y));
    cycle_group result(r_x, r_y, false);

    bb::ecc_add_gate_<FF> add_gate{
        .x1 = x.get_witness_index(),
        .y1 = y.get_witness_index(),
        .x2 = other.x.get_witness_index(),
        .y2 = other.y.get_witness_index(),
        .x3 = result.x.get_witness_index(),
        .y3 = result.y.get_witness_index(),
        .sign_coefficient = 1,
    };
    context->create_ecc_add_gate(add_gate);

    return result;
}

/**
 * @brief will evaluate ECC point subtraction over `*this` and `other`.
 *        Incomplete addition formula edge cases are *NOT* checked!
 *        Only use this method if you know the x-coordinates of the operands cannot collide
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::unconditional_subtract(const cycle_group& other) const
{
    if constexpr (!IS_ULTRA) {
        return unconditional_add(-other);
    } else {
        auto context = get_context(other);

        const bool lhs_constant = is_constant();
        const bool rhs_constant = other.is_constant();

        if (lhs_constant && !rhs_constant) {
            auto lhs = cycle_group<Composer>::from_constant_witness(context, get_value());
            return lhs.unconditional_subtract(other);
        }
        if (!lhs_constant && rhs_constant) {
            auto rhs = cycle_group<Composer>::from_constant_witness(context, other.get_value());
            return unconditional_subtract(rhs);
        }
        auto p1 = get_value();
        auto p2 = other.get_value();
        AffineElement p3(Element(p1) - Element(p2));
        if (lhs_constant && rhs_constant) {
            return cycle_group(p3);
        }
        field_t r_x(witness_t(context, p3.x));
        field_t r_y(witness_t(context, p3.y));
        cycle_group result(r_x, r_y, false);

        bb::ecc_add_gate_<FF> add_gate{
            .x1 = x.get_witness_index(),
            .y1 = y.get_witness_index(),
            .x2 = other.x.get_witness_index(),
            .y2 = other.y.get_witness_index(),
            .x3 = result.x.get_witness_index(),
            .y3 = result.y.get_witness_index(),
            .sign_coefficient = -1,
        };
        context->create_ecc_add_gate(add_gate);

        return result;
    }
}

/**
 * @brief Will evaluate ECC point addition over `*this` and `other`.
 *        Uses incomplete addition formula
 *        If incomplete addition formula edge cases are triggered (x-coordinates of operands collide),
 *        the constraints produced by this method will be unsatisfiable.
 *        Useful when an honest prover will not produce a point collision with overwhelming probability,
 *        but a cheating prover will be able to.
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::checked_unconditional_add(const cycle_group& other) const
{
    field_t x_delta = x - other.x;
    x_delta.assert_is_not_zero("cycle_group::checked_unconditional_add, x-coordinate collision");
    return unconditional_add(other);
}

/**
 * @brief Will evaluate ECC point subtraction over `*this` and `other`.
 *        Uses incomplete addition formula
 *        If incomplete addition formula edge cases are triggered (x-coordinates of operands collide),
 *        the constraints produced by this method will be unsatisfiable.
 *        Useful when an honest prover will not produce a point collision with overwhelming probability,
 *        but a cheating prover will be able to.
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::checked_unconditional_subtract(const cycle_group& other) const
{
    field_t x_delta = x - other.x;
    x_delta.assert_is_not_zero("cycle_group::checked_unconditional_subtract, x-coordinate collision");
    return unconditional_subtract(other);
}

/**
 * @brief Will evaluate ECC point addition over `*this` and `other`.
 *        This method uses complete addition i.e. is compatible with edge cases.
 *        Method is expensive due to needing to evaluate both an addition, a doubling,
 *        plus conditional logic to handle points at infinity.
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer> cycle_group<Composer> cycle_group<Composer>::operator+(const cycle_group& other) const
{
    Composer* context = get_context(other);
    const bool_t x_coordinates_match = (x == other.x);
    const bool_t y_coordinates_match = (y == other.y);
    const bool_t double_predicate = (x_coordinates_match && y_coordinates_match);
    const bool_t infinity_predicate = (x_coordinates_match && !y_coordinates_match);

    auto x1 = x;
    auto y1 = y;
    auto x2 = other.x;
    auto y2 = other.y;
    // if x_coordinates match, lambda triggers a divide by zero error.
    // Adding in `x_coordinates_match` ensures that lambda will always be well-formed
    auto x_diff = x2.add_two(-x1, x_coordinates_match);
    auto lambda = (y2 - y1) / x_diff;
    auto x3 = lambda.madd(lambda, -(x2 + x1));
    auto y3 = lambda.madd(x1 - x3, -y1);
    cycle_group add_result(x3, y3, x_coordinates_match);

    auto dbl_result = dbl();

    // dbl if x_match, y_match
    // infinity if x_match, !y_match
    cycle_group result(context);
    result.x = field_t::conditional_assign(double_predicate, dbl_result.x, add_result.x);
    result.y = field_t::conditional_assign(double_predicate, dbl_result.y, add_result.y);

    const bool_t lhs_infinity = is_point_at_infinity();
    const bool_t rhs_infinity = other.is_point_at_infinity();
    // if lhs infinity, return rhs
    result.x = field_t::conditional_assign(lhs_infinity, other.x, result.x);
    result.y = field_t::conditional_assign(lhs_infinity, other.y, result.y);

    // if rhs infinity, return lhs
    result.x = field_t::conditional_assign(rhs_infinity, x, result.x);
    result.y = field_t::conditional_assign(rhs_infinity, y, result.y);

    // is result point at infinity?
    // yes = infinity_predicate && !lhs_infinity && !rhs_infinity
    // yes = lhs_infinity && rhs_infinity
    // n.b. can likely optimize this
    bool_t result_is_infinity = infinity_predicate && (!lhs_infinity && !rhs_infinity);
    result_is_infinity = result_is_infinity || (lhs_infinity && rhs_infinity);
    result.set_point_at_infinity(result_is_infinity);
    return result;
}

/**
 * @brief Will evaluate ECC point subtraction over `*this` and `other`.
 *        This method uses complete addition i.e. is compatible with edge cases.
 *        Method is expensive due to needing to evaluate both an addition, a doubling,
 *        plus conditional logic to handle points at infinity.
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer> cycle_group<Composer> cycle_group<Composer>::operator-(const cycle_group& other) const
{
    Composer* context = get_context(other);
    const bool_t x_coordinates_match = (x == other.x);
    const bool_t y_coordinates_match = (y == other.y);
    const bool_t double_predicate = (x_coordinates_match && !y_coordinates_match).normalize();
    const bool_t infinity_predicate = (x_coordinates_match && y_coordinates_match).normalize();

    auto x1 = x;
    auto y1 = y;
    auto x2 = other.x;
    auto y2 = other.y;
    auto x_diff = x2.add_two(-x1, x_coordinates_match);
    auto lambda = (-y2 - y1) / x_diff;
    auto x3 = lambda.madd(lambda, -(x2 + x1));
    auto y3 = lambda.madd(x1 - x3, -y1);
    cycle_group add_result(x3, y3, x_coordinates_match);

    auto dbl_result = dbl();

    // dbl if x_match, !y_match
    // infinity if x_match, y_match
    cycle_group result(context);
    result.x = field_t::conditional_assign(double_predicate, dbl_result.x, add_result.x);
    result.y = field_t::conditional_assign(double_predicate, dbl_result.y, add_result.y);

    const bool_t lhs_infinity = is_point_at_infinity();
    const bool_t rhs_infinity = other.is_point_at_infinity();
    // if lhs infinity, return -rhs
    result.x = field_t::conditional_assign(lhs_infinity, other.x, result.x);
    result.y = field_t::conditional_assign(lhs_infinity, (-other.y).normalize(), result.y);

    // if rhs infinity, return lhs
    result.x = field_t::conditional_assign(rhs_infinity, x, result.x);
    result.y = field_t::conditional_assign(rhs_infinity, y, result.y);

    // is result point at infinity?
    // yes = infinity_predicate && !lhs_infinity && !rhs_infinity
    // yes = lhs_infinity && rhs_infinity
    // n.b. can likely optimize this
    bool_t result_is_infinity = infinity_predicate && (!lhs_infinity && !rhs_infinity);
    result_is_infinity = result_is_infinity || (lhs_infinity && rhs_infinity);
    result.set_point_at_infinity(result_is_infinity);

    return result;
}

/**
 * @brief Negates a point
 *
 * @tparam Composer
 * @param other
 * @return cycle_group<Composer>
 */
template <typename Composer> cycle_group<Composer> cycle_group<Composer>::operator-() const
{
    cycle_group result(*this);
    result.y = -y;
    return result;
}

template <typename Composer> cycle_group<Composer>& cycle_group<Composer>::operator+=(const cycle_group& other)
{
    *this = *this + other;
    return *this;
}

template <typename Composer> cycle_group<Composer>& cycle_group<Composer>::operator-=(const cycle_group& other)
{
    *this = *this - other;
    return *this;
}

template <typename Composer>
cycle_group<Composer>::cycle_scalar::cycle_scalar(const field_t& _lo, const field_t& _hi)
    : lo(_lo)
    , hi(_hi)
{}

template <typename Composer> cycle_group<Composer>::cycle_scalar::cycle_scalar(const field_t& _in)
{
    const uint256_t value(_in.get_value());
    const uint256_t lo_v = value.slice(0, LO_BITS);
    const uint256_t hi_v = value.slice(LO_BITS, HI_BITS);
    constexpr uint256_t shift = uint256_t(1) << LO_BITS;
    if (_in.is_constant()) {
        lo = lo_v;
        hi = hi_v;
    } else {
        lo = witness_t(_in.get_context(), lo_v);
        hi = witness_t(_in.get_context(), hi_v);
        (lo + hi * shift).assert_equal(_in);
    }
}

template <typename Composer> cycle_group<Composer>::cycle_scalar::cycle_scalar(const ScalarField& _in)
{
    const uint256_t value(_in);
    const uint256_t lo_v = value.slice(0, LO_BITS);
    const uint256_t hi_v = value.slice(LO_BITS, HI_BITS);
    lo = lo_v;
    hi = hi_v;
}

template <typename Composer>
typename cycle_group<Composer>::cycle_scalar cycle_group<Composer>::cycle_scalar::from_witness(Composer* context,
                                                                                               const ScalarField& value)
{
    const uint256_t value_u256(value);
    const uint256_t lo_v = value_u256.slice(0, LO_BITS);
    const uint256_t hi_v = value_u256.slice(LO_BITS, HI_BITS);
    field_t lo = witness_t(context, lo_v);
    field_t hi = witness_t(context, hi_v);
    return cycle_scalar(lo, hi);
}

/**
 * @brief Use when we want to multiply a group element by a string of bits of known size.
 *        N.B. using this constructor method will make our scalar multiplication methods not perform primality tests.
 *
 * @tparam Composer
 * @param context
 * @param value
 * @param num_bits
 * @return cycle_group<Composer>::cycle_scalar
 */
template <typename Composer>
typename cycle_group<Composer>::cycle_scalar cycle_group<Composer>::cycle_scalar::from_witness_bitstring(
    Composer* context, const uint256_t& bitstring, const size_t num_bits)
{
    ASSERT(bitstring.get_msb() < num_bits);
    const uint256_t lo_v = bitstring.slice(0, LO_BITS);
    const uint256_t hi_v = bitstring.slice(LO_BITS, HI_BITS);
    field_t lo = witness_t(context, lo_v);
    field_t hi = witness_t(context, hi_v);
    cycle_scalar result{ lo, hi, num_bits, true, false };
    return result;
}

/**
 * @brief Use when we want to multiply a group element by a string of bits of known size.
 *        N.B. using this constructor method will make our scalar multiplication methods not perform primality tests.
 *
 * @tparam Composer
 * @param context
 * @param value
 * @param num_bits
 * @return cycle_group<Composer>::cycle_scalar
 */
template <typename Composer>
typename cycle_group<Composer>::cycle_scalar cycle_group<Composer>::cycle_scalar::create_from_bn254_scalar(
    const field_t& in, const bool skip_primality_test)
{
    const uint256_t value_u256(in.get_value());
    const uint256_t lo_v = value_u256.slice(0, LO_BITS);
    const uint256_t hi_v = value_u256.slice(LO_BITS, HI_BITS);
    if (in.is_constant()) {
        cycle_scalar result{ field_t(lo_v), field_t(hi_v), NUM_BITS, false, true };
        return result;
    }
    field_t lo = witness_t(in.get_context(), lo_v);
    field_t hi = witness_t(in.get_context(), hi_v);
    lo.add_two(hi * (uint256_t(1) << LO_BITS), -in).assert_equal(0);
    cycle_scalar result{ lo, hi, NUM_BITS, skip_primality_test, true };
    return result;
}

template <typename Composer> bool cycle_group<Composer>::cycle_scalar::is_constant() const
{
    return (lo.is_constant() && hi.is_constant());
}

/**
 * @brief Checks that a cycle_scalar value is smaller than a prime field modulus when evaluated over the INTEGERS
 * N.B. The prime we check can be either the SNARK curve group order or the circuit's embedded curve group order
 * (i.e. BN254 or Grumpkin)
 * For a canonical scalar mul, we check against the embedded curve (i.e. the curve
 * cycle_group implements).
 * HOWEVER: for Pedersen hashes and Pedersen commitments, the hashed/committed data will be
 * native circuit field elements i.e. for a BN254 snark, cycle_group = Grumpkin and we will be committing/hashing
 * BN254::ScalarField values *NOT* Grumpkin::ScalarFIeld values.
 * TLDR: whether the input scalar has to be < BN254::ScalarField or < Grumpkin::ScalarField is context-dependent.
 *
 * @tparam Composer
 */
template <typename Composer> void cycle_group<Composer>::cycle_scalar::validate_scalar_is_in_field() const
{
    if (!is_constant() && !skip_primality_test()) {
        // Check that scalar.hi * 2^LO_BITS + scalar.lo < cycle_group_modulus when evaluated over the integers
        const uint256_t cycle_group_modulus =
            use_bn254_scalar_field_for_primality_test() ? FF::modulus : ScalarField::modulus;
        const uint256_t r_lo = cycle_group_modulus.slice(0, cycle_scalar::LO_BITS);
        const uint256_t r_hi = cycle_group_modulus.slice(cycle_scalar::LO_BITS, cycle_scalar::HI_BITS);

        bool need_borrow = uint256_t(lo.get_value()) > r_lo;
        field_t borrow = lo.is_constant() ? need_borrow : field_t::from_witness(get_context(), need_borrow);

        // directly call `create_new_range_constraint` to avoid creating an arithmetic gate
        if (!lo.is_constant()) {
            if constexpr (IS_ULTRA) {
                get_context()->create_new_range_constraint(borrow.get_witness_index(), 1, "borrow");
            } else {
                borrow.assert_equal(borrow * borrow);
            }
        }
        // Hi range check = r_hi - y_hi - borrow
        // Lo range check = r_lo - y_lo + borrow * 2^{126}
        field_t hi_diff = (-hi + r_hi) - borrow;
        field_t lo_diff = (-lo + r_lo) + (borrow * (uint256_t(1) << cycle_scalar::LO_BITS));

        hi_diff.create_range_constraint(cycle_scalar::HI_BITS);
        lo_diff.create_range_constraint(cycle_scalar::LO_BITS);
    }
}

template <typename Composer>
typename cycle_group<Composer>::ScalarField cycle_group<Composer>::cycle_scalar::get_value() const
{
    uint256_t lo_v(lo.get_value());
    uint256_t hi_v(hi.get_value());
    return ScalarField(lo_v + (hi_v << LO_BITS));
}

/**
 * @brief Construct a new cycle group<Composer>::straus scalar slice::straus scalar slice object
 *
 * @details As part of slicing algoirthm, we also perform a primality test on the inut scalar.
 *
 * TODO(@zac-williamson) make the primality test configurable.
 * We may want to validate the input < BN254::Fr OR input < Grumpkin::Fr depending on context!
 *
 * @tparam Composer
 * @param context
 * @param scalar
 * @param table_bits
 */
template <typename Composer>
cycle_group<Composer>::straus_scalar_slice::straus_scalar_slice(Composer* context,
                                                                const cycle_scalar& scalar,
                                                                const size_t table_bits)
    : _table_bits(table_bits)
{
    // convert an input cycle_scalar object into a vector of slices, each containing `table_bits` bits.
    // this also performs an implicit range check on the input slices
    const auto slice_scalar = [&](const field_t& scalar, const size_t num_bits) {
        std::vector<field_t> result;
        if (num_bits == 0) {
            return result;
        }
        if (scalar.is_constant()) {
            const size_t num_slices = (num_bits + table_bits - 1) / table_bits;
            const uint64_t table_mask = (1ULL << table_bits) - 1ULL;
            uint256_t raw_value = scalar.get_value();
            for (size_t i = 0; i < num_slices; ++i) {
                uint64_t slice_v = static_cast<uint64_t>(raw_value.data[0]) & table_mask;
                result.push_back(field_t(slice_v));
                raw_value = raw_value >> table_bits;
            }
            return result;
        }
        if constexpr (IS_ULTRA) {
            const auto slice_indices =
                context->decompose_into_default_range(scalar.normalize().get_witness_index(),
                                                      num_bits,
                                                      table_bits,
                                                      "straus_scalar_slice decompose_into_default_range");
            for (auto& idx : slice_indices) {
                result.emplace_back(field_t::from_witness_index(context, idx));
            }
        } else {
            uint256_t raw_value = scalar.get_value();
            const uint64_t table_mask = (1ULL << table_bits) - 1ULL;
            const size_t num_slices = (num_bits + table_bits - 1) / table_bits;
            for (size_t i = 0; i < num_slices; ++i) {
                uint64_t slice_v = static_cast<uint64_t>(raw_value.data[0]) & table_mask;
                field_t slice(witness_t(context, slice_v));

                context->create_range_constraint(
                    slice.get_witness_index(), table_bits, "straus_scalar_slice create_range_constraint");

                result.emplace_back(slice);
                raw_value = raw_value >> table_bits;
            }
            std::vector<field_t> linear_elements;
            FF scaling_factor = 1;
            for (size_t i = 0; i < num_slices; ++i) {
                linear_elements.emplace_back(result[i] * scaling_factor);
                scaling_factor += scaling_factor;
            }
            field_t::accumulate(linear_elements).assert_equal(scalar);
        }
        return result;
    };

    const size_t lo_bits = scalar.num_bits() > cycle_scalar::LO_BITS ? cycle_scalar::LO_BITS : scalar.num_bits();
    const size_t hi_bits = scalar.num_bits() > cycle_scalar::LO_BITS ? scalar.num_bits() - cycle_scalar::LO_BITS : 0;
    auto hi_slices = slice_scalar(scalar.hi, hi_bits);
    auto lo_slices = slice_scalar(scalar.lo, lo_bits);

    std::copy(lo_slices.begin(), lo_slices.end(), std::back_inserter(slices));
    std::copy(hi_slices.begin(), hi_slices.end(), std::back_inserter(slices));
}

/**
 * @brief Return a bit-slice associated with round `index`.
 *
 * @details In Straus algorithm, `index` is a known parameter, so no need for expensive lookup tables
 *
 * @tparam Composer
 * @param index
 * @return field_t<Composer>
 */
template <typename Composer>
std::optional<field_t<Composer>> cycle_group<Composer>::straus_scalar_slice::read(size_t index)
{
    if (index >= slices.size()) {
        return std::nullopt;
    }
    return slices[index];
}

/**
 * @brief Construct a new cycle group<Composer>::straus lookup table::straus lookup table object
 *
 * @details Constructs a `table_bits` lookup table.
 *
 * If Composer is not ULTRA, `table_bits = 1`
 * If Composer is ULTRA, ROM table is used as lookup table
 *
 * @tparam Composer
 * @param context
 * @param base_point
 * @param offset_generator
 * @param table_bits
 */
template <typename Composer>
cycle_group<Composer>::straus_lookup_table::straus_lookup_table(Composer* context,
                                                                const cycle_group& base_point,
                                                                const cycle_group& offset_generator,
                                                                size_t table_bits)
    : _table_bits(table_bits)
    , _context(context)
{
    const size_t table_size = 1UL << table_bits;
    point_table.resize(table_size);
    point_table[0] = offset_generator;

    // We want to support the case where input points are points at infinity.
    // If base point is at infinity, we want every point in the table to just be `generator_point`.
    // We achieve this via the following:
    // 1: We create a "work_point" that is base_point if not at infinity, otherwise is just 1
    // 2: When computing the point table, we use "work_point" in additions instead of the "base_point" (to prevent
    //    x-coordinate collisions in honest case) 3: When assigning to the point table, we conditionally assign either
    //    the output of the point addition (if not at infinity) or the generator point (if at infinity)
    // Note: if `base_point.is_point_at_infinity()` is constant, these conditional assigns produce zero gate overhead
    cycle_group fallback_point(Group::affine_one);
    field_t modded_x = field_t::conditional_assign(base_point.is_point_at_infinity(), fallback_point.x, base_point.x);
    field_t modded_y = field_t::conditional_assign(base_point.is_point_at_infinity(), fallback_point.y, base_point.y);
    cycle_group modded_base_point(modded_x, modded_y, false);
    for (size_t i = 1; i < table_size; ++i) {
        auto add_output = point_table[i - 1].checked_unconditional_add(modded_base_point);
        field_t x = field_t::conditional_assign(base_point.is_point_at_infinity(), offset_generator.x, add_output.x);
        field_t y = field_t::conditional_assign(base_point.is_point_at_infinity(), offset_generator.y, add_output.y);
        point_table[i] = cycle_group(x, y, false);
    }
    if constexpr (IS_ULTRA) {
        rom_id = context->create_ROM_array(table_size);
        for (size_t i = 0; i < table_size; ++i) {
            if (point_table[i].is_constant()) {
                auto element = point_table[i].get_value();
                point_table[i] = cycle_group::from_constant_witness(_context, element);
                point_table[i].x.assert_equal(element.x);
                point_table[i].y.assert_equal(element.y);
            }
            context->set_ROM_element_pair(
                rom_id,
                i,
                std::array<uint32_t, 2>{ point_table[i].x.get_witness_index(), point_table[i].y.get_witness_index() });
        }
    } else {
        ASSERT(table_bits == 1);
    }
}

/**
 * @brief Given an `_index` witness, return `straus_lookup_table[index]`
 *
 * @tparam Composer
 * @param _index
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::straus_lookup_table::read(const field_t& _index)
{
    if constexpr (IS_ULTRA) {
        field_t index(_index);
        if (index.is_constant()) {
            index = witness_t(_context, _index.get_value());
            index.assert_equal(_index.get_value());
        }
        auto output_indices = _context->read_ROM_array_pair(rom_id, index.get_witness_index());
        field_t x = field_t::from_witness_index(_context, output_indices[0]);
        field_t y = field_t::from_witness_index(_context, output_indices[1]);
        return cycle_group(x, y, false);
    }
    field_t x = _index * (point_table[1].x - point_table[0].x) + point_table[0].x;
    field_t y = _index * (point_table[1].y - point_table[0].y) + point_table[0].y;
    return cycle_group(x, y, false);
}

/**
 * @brief Internal algorithm to perform a variable-base batch mul.
 *
 * @note Explicit assumption that all base_points are witnesses and not constants!
 *       Constant points must be filtered out by `batch_mul` before calling this.
 *
 * @details batch mul performed via the Straus multiscalar multiplication algorithm
 *          (optimal for MSMs where num points <128-ish).
 *          If Composer is not ULTRA, number of bits per Straus round = 1,
 *          which reduces to the basic double-and-add algorithm
 *
 * @details If `unconditional_add = true`, we use `::unconditional_add` instead of `::checked_unconditional_add`.
 *          Use with caution! Only should be `true` if we're doing an ULTRA fixed-base MSM so we know the points cannot
 *          collide with the offset generators.
 *
 * @note ULTRA Composer will call `_variable_base_batch_mul_internal` to evaluate fixed-base MSMs over points that do
 *       not exist in our precomputed plookup tables. This is a comprimise between maximising circuit efficiency and
 *       minimizing the blowup size of our precomputed table polynomials. variable-base mul uses small ROM lookup tables
 *       which are witness-defined and not part of the plookup protocol.
 * @tparam Composer
 * @param scalars
 * @param base_points
 * @param offset_generators
 * @param unconditional_add
 * @return cycle_group<Composer>::batch_mul_internal_output
 */
template <typename Composer>
typename cycle_group<Composer>::batch_mul_internal_output cycle_group<Composer>::_variable_base_batch_mul_internal(
    const std::span<cycle_scalar> scalars,
    const std::span<cycle_group> base_points,
    const std::span<AffineElement const> offset_generators,
    const bool unconditional_add)
{
    ASSERT(scalars.size() == base_points.size());
    Composer* context = nullptr;
    for (auto& scalar : scalars) {
        if (scalar.lo.get_context() != nullptr) {
            context = scalar.get_context();
            break;
        }
    }
    for (auto& point : base_points) {
        if (point.get_context() != nullptr) {
            context = point.get_context();
            break;
        }
    }

    size_t num_bits = 0;
    for (auto& s : scalars) {
        num_bits = std::max(num_bits, s.num_bits());
    }
    size_t num_rounds = (num_bits + TABLE_BITS - 1) / TABLE_BITS;

    const size_t num_points = scalars.size();

    std::vector<straus_scalar_slice> scalar_slices;
    std::vector<straus_lookup_table> point_tables;
    for (size_t i = 0; i < num_points; ++i) {
        scalar_slices.emplace_back(straus_scalar_slice(context, scalars[i], TABLE_BITS));
        point_tables.emplace_back(straus_lookup_table(context, base_points[i], offset_generators[i + 1], TABLE_BITS));
    }

    Element offset_generator_accumulator = offset_generators[0];
    cycle_group accumulator = offset_generators[0];

    // populate the set of points we are going to add into our accumulator, *before* we do any ECC operations
    // this way we are able to fuse mutliple ecc add / ecc double operations and reduce total gate count.
    // (ecc add/ecc double gates normally cost 2 UltraPlonk gates. However if we chain add->add, add->double,
    // double->add, double->double, they only cost one)
    std::vector<cycle_group> points_to_add;
    for (size_t i = 0; i < num_rounds; ++i) {
        for (size_t j = 0; j < num_points; ++j) {
            const std::optional<field_t> scalar_slice = scalar_slices[j].read(num_rounds - i - 1);
            // if we are doing a batch mul over scalars of different bit-lengths, we may not have any scalar bits for a
            // given round and a given scalar
            if (scalar_slice.has_value()) {
                const cycle_group point = point_tables[j].read(scalar_slice.value());
                points_to_add.emplace_back(point);
            }
        }
    }
    std::vector<std::tuple<field_t, field_t>> x_coordinate_checks;
    size_t point_counter = 0;
    for (size_t i = 0; i < num_rounds; ++i) {
        if (i != 0) {
            for (size_t j = 0; j < TABLE_BITS; ++j) {
                // offset_generator_accuulator is a regular Element, so dbl() won't add constraints
                accumulator = accumulator.dbl();
                offset_generator_accumulator = offset_generator_accumulator.dbl();
            }
        }

        for (size_t j = 0; j < num_points; ++j) {
            const std::optional<field_t> scalar_slice = scalar_slices[j].read(num_rounds - i - 1);
            // if we are doing a batch mul over scalars of different bit-lengths, we may not have a bit slice for a
            // given round and a given scalar
            if (scalar_slice.has_value()) {
                const auto& point = points_to_add[point_counter++];
                if (!unconditional_add) {
                    x_coordinate_checks.push_back({ accumulator.x, point.x });
                }
                accumulator = accumulator.unconditional_add(point);
                offset_generator_accumulator = offset_generator_accumulator + Element(offset_generators[j + 1]);
            }
        }
    }

    for (auto& [x1, x2] : x_coordinate_checks) {
        auto x_diff = x2 - x1;
        x_diff.assert_is_not_zero("_variable_base_batch_mul_internal x-coordinate collision");
    }
    /**
     * offset_generator_accumulator represents the sum of all the offset generator terms present in `accumulator`.
     * We don't subtract off yet, as we may be able to combine `offset_generator_accumulator` with other constant terms
     * in `batch_mul` before performing the subtraction.
     */
    return { accumulator, AffineElement(offset_generator_accumulator) };
}

/**
 * @brief Internal algorithm to perform a fixed-base batch mul for ULTRA Composer
 *
 * @details Uses plookup tables which contain lookups for precomputed multiples of the input base points.
 *          Means we can avoid all point doublings and reduce one scalar mul to ~29 lookups + 29 ecc addition gates
 *
 * @tparam Composer
 * @param scalars
 * @param base_points
 * @param
 * @return cycle_group<Composer>::batch_mul_internal_output
 */
template <typename Composer>
typename cycle_group<Composer>::batch_mul_internal_output cycle_group<Composer>::_fixed_base_batch_mul_internal(
    const std::span<cycle_scalar> scalars,
    const std::span<AffineElement> base_points,
    const std::span<AffineElement const> /*unused*/)
    requires IsUltraArithmetic<Composer>
{
    ASSERT(scalars.size() == base_points.size());

    const size_t num_points = base_points.size();
    using MultiTableId = plookup::MultiTableId;
    using ColumnIdx = plookup::ColumnIdx;

    std::vector<MultiTableId> plookup_table_ids;
    std::vector<AffineElement> plookup_base_points;
    std::vector<field_t> plookup_scalars;

    for (size_t i = 0; i < num_points; ++i) {
        std::optional<std::array<MultiTableId, 2>> table_id =
            plookup::fixed_base::table::get_lookup_table_ids_for_point(base_points[i]);
        ASSERT(table_id.has_value());
        plookup_table_ids.emplace_back(table_id.value()[0]);
        plookup_table_ids.emplace_back(table_id.value()[1]);
        plookup_base_points.emplace_back(base_points[i]);
        plookup_base_points.emplace_back(Element(base_points[i]) * (uint256_t(1) << cycle_scalar::LO_BITS));
        plookup_scalars.emplace_back(scalars[i].lo);
        plookup_scalars.emplace_back(scalars[i].hi);
    }

    std::vector<cycle_group> lookup_points;
    Element offset_generator_accumulator = Group::point_at_infinity;
    for (size_t i = 0; i < plookup_scalars.size(); ++i) {
        plookup::ReadData<field_t> lookup_data =
            plookup_read<Composer>::get_lookup_accumulators(plookup_table_ids[i], plookup_scalars[i]);
        for (size_t j = 0; j < lookup_data[ColumnIdx::C2].size(); ++j) {
            const auto x = lookup_data[ColumnIdx::C2][j];
            const auto y = lookup_data[ColumnIdx::C3][j];
            lookup_points.emplace_back(cycle_group(x, y, false));
        }

        std::optional<AffineElement> offset_1 =
            plookup::fixed_base::table::get_generator_offset_for_table_id(plookup_table_ids[i]);

        ASSERT(offset_1.has_value());
        offset_generator_accumulator += offset_1.value();
    }
    cycle_group accumulator = lookup_points[0];
    // Perform all point additions sequentially. The Ultra ecc_addition relation costs 1 gate iff additions are chained
    // and output point of previous addition = input point of current addition.
    // If this condition is not met, the addition relation costs 2 gates. So it's good to do these sequentially!
    for (size_t i = 1; i < lookup_points.size(); ++i) {
        accumulator = accumulator.unconditional_add(lookup_points[i]);
    }
    /**
     * offset_generator_accumulator represents the sum of all the offset generator terms present in `accumulator`.
     * We don't subtract off yet, as we may be able to combine `offset_generator_accumulator` with other constant terms
     * in `batch_mul` before performing the subtraction.
     */
    return { accumulator, offset_generator_accumulator };
}

/**
 * @brief Internal algorithm to perform a fixed-base batch mul for Non-ULTRA Composers
 *
 * @details Multiples of the base point are precomputed, which avoids us having to add ecc doubling gates.
 *          More efficient than variable-base version.
 *
 * @tparam Composer
 * @param scalars
 * @param base_points
 * @param off
 * @return cycle_group<Composer>::batch_mul_internal_output
 */
template <typename Composer>
typename cycle_group<Composer>::batch_mul_internal_output cycle_group<Composer>::_fixed_base_batch_mul_internal(
    const std::span<cycle_scalar> scalars,
    const std::span<AffineElement> base_points,
    const std::span<AffineElement const> offset_generators)
    requires IsNotUltraArithmetic<Composer>

{
    ASSERT(scalars.size() == base_points.size());
    static_assert(TABLE_BITS == 1);

    Composer* context = nullptr;
    for (auto& scalar : scalars) {
        if (scalar.get_context() != nullptr) {
            context = scalar.get_context();
            break;
        }
    }

    size_t num_bits = 0;
    for (auto& s : scalars) {
        num_bits = std::max(num_bits, s.num_bits());
    }
    size_t num_rounds = (num_bits + TABLE_BITS - 1) / TABLE_BITS;
    // core algorithm
    // define a `table_bits` size lookup table
    const size_t num_points = scalars.size();
    using straus_round_tables = std::vector<straus_lookup_table>;

    std::vector<straus_scalar_slice> scalar_slices;
    std::vector<straus_round_tables> point_tables(num_points);

    // creating these point tables should cost 0 constraints if base points are constant
    for (size_t i = 0; i < num_points; ++i) {
        std::vector<Element> round_points(num_rounds);
        std::vector<Element> round_offset_generators(num_rounds);
        round_points[0] = base_points[i];
        round_offset_generators[0] = offset_generators[i + 1];
        for (size_t j = 1; j < num_rounds; ++j) {
            round_points[j] = round_points[j - 1].dbl();
            round_offset_generators[j] = round_offset_generators[j - 1].dbl();
        }
        Element::batch_normalize(&round_points[0], num_rounds);
        Element::batch_normalize(&round_offset_generators[0], num_rounds);
        point_tables[i].resize(num_rounds);
        for (size_t j = 0; j < num_rounds; ++j) {
            point_tables[i][j] = straus_lookup_table(
                context, cycle_group(round_points[j]), cycle_group(round_offset_generators[j]), TABLE_BITS);
        }
        scalar_slices.emplace_back(straus_scalar_slice(context, scalars[i], TABLE_BITS));
    }
    Element offset_generator_accumulator = offset_generators[0];
    cycle_group accumulator = cycle_group(Element(offset_generators[0]) * (uint256_t(1) << (num_rounds - 1)));
    for (size_t i = 0; i < num_rounds; ++i) {
        offset_generator_accumulator = (i > 0) ? offset_generator_accumulator.dbl() : offset_generator_accumulator;
        for (size_t j = 0; j < num_points; ++j) {
            auto& point_table = point_tables[j][i];
            const std::optional<field_t> scalar_slice = scalar_slices[j].read(i);
            // if we are doing a batch mul over scalars of different bit-lengths, we may not have any scalar bits for a
            // given round and a given scalar
            if (scalar_slice.has_value()) {
                const cycle_group point = point_table.read(scalar_slice.value());
                accumulator = accumulator.unconditional_add(point);
                offset_generator_accumulator = offset_generator_accumulator + Element(offset_generators[j + 1]);
            }
        }
    }

    /**
     * offset_generator_accumulator represents the sum of all the offset generator terms present in `accumulator`.
     * We don't subtract off yet, as we may be able to combine `offset_generator_accumulator` with other constant terms
     * in `batch_mul` before performing the subtraction.
     */
    return { accumulator, offset_generator_accumulator };
}

/**
 * @brief Multiscalar multiplication algorithm.
 *
 * @details Uses the Straus MSM algorithm. `batch_mul` splits inputs into three categories:
 *          1. point and scalar multiplier are both constant
 *          2. point is constant, scalar multiplier is a witness
 *          3. point is a witness, scalar multiplier can be witness or constant
 *
 * For Category 1, the scalar mul can be precomuted without constraints
 * For Category 2, we use a fixed-base variant of Straus (with plookup tables if available).
 * For Category 3, we use standard Straus.
 * The results from all 3 categories are combined and returned as an output point.
 *
 * @note batch_mul can handle all known cases of trigger incomplete addition formula exceptions and other weirdness:
 *       1. some/all of the input points are points at infinity
 *       2. some/all of the input scalars are 0
 *       3. some/all input points are equal to each other
 *       4. output is the point at infinity
 *       5. input vectors are empty
 *
 * @note offset_generator_data is a pointer to precomputed offset generator list.
 *       There is a default parameter point that poitns to a list with DEFAULT_NUM_GENERATORS generator points (32)
 *       If more offset generators are required, they will be derived in-place which can be expensive.
 *       (num required offset generators is either num input points + 1 or num input points + 2,
 *        depends on if one or both of _fixed_base_batch_mul_internal, _variable_base_batch_mul_internal are called)
 *       If you're calling this function repeatedly and you KNOW you need >32 offset generators,
 *       it's faster to create a `generator_data` object with the required size and pass it in as a parameter.
 * @tparam Composer
 * @param scalars
 * @param base_points
 * @param offset_generator_data
 * @return cycle_group<Composer>
 */
template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::batch_mul(const std::vector<cycle_scalar>& scalars,
                                                       const std::vector<cycle_group>& base_points,
                                                       const GeneratorContext context)
{
    ASSERT(scalars.size() == base_points.size());

    std::vector<cycle_scalar> variable_base_scalars;
    std::vector<cycle_group> variable_base_points;
    std::vector<cycle_scalar> fixed_base_scalars;
    std::vector<AffineElement> fixed_base_points;

    size_t num_bits = 0;
    for (auto& s : scalars) {
        num_bits = std::max(num_bits, s.num_bits());

        // Note: is this the best place to put `validate_is_in_field`? Should it not be part of the constructor?
        // Note note: validate_scalar_is_in_field does not apply range checks to the hi/lo slices, this is performed
        // implicitly via the scalar mul algorithm
        s.validate_scalar_is_in_field();
    }

    // if num_bits != NUM_BITS, skip lookup-version of fixed-base scalar mul. too much complexity
    bool num_bits_not_full_field_size = num_bits != NUM_BITS;

    // When calling `_variable_base_batch_mul_internal`, we can unconditionally add iff all of the input points
    // are fixed-base points
    // (i.e. we are ULTRA Composer and we are doing fixed-base mul over points not present in our plookup tables)
    bool can_unconditional_add = true;
    bool has_non_constant_component = false;
    Element constant_acc = Group::point_at_infinity;
    for (size_t i = 0; i < scalars.size(); ++i) {
        bool scalar_constant = scalars[i].is_constant();
        bool point_constant = base_points[i].is_constant();
        if (scalar_constant && point_constant) {
            constant_acc += (base_points[i].get_value()) * (scalars[i].get_value());
        } else if (!scalar_constant && point_constant) {
            if (base_points[i].get_value().is_point_at_infinity()) {
                // oi mate, why are you creating a circuit that multiplies a known point at infinity?
                continue;
            }
            if constexpr (IS_ULTRA) {
                if (!num_bits_not_full_field_size &&
                    plookup::fixed_base::table::lookup_table_exists_for_point(base_points[i].get_value())) {
                    fixed_base_scalars.push_back(scalars[i]);
                    fixed_base_points.push_back(base_points[i].get_value());
                } else {
                    // womp womp. We have lookup tables at home. ROM tables.
                    variable_base_scalars.push_back(scalars[i]);
                    variable_base_points.push_back(base_points[i]);
                }
            } else {
                fixed_base_scalars.push_back(scalars[i]);
                fixed_base_points.push_back(base_points[i].get_value());
            }
            has_non_constant_component = true;
        } else {
            variable_base_scalars.push_back(scalars[i]);
            variable_base_points.push_back(base_points[i]);
            can_unconditional_add = false;
            has_non_constant_component = true;
            // variable base
        }
    }

    // If all inputs are constant, return the computed constant component and call it a day.
    if (!has_non_constant_component) {
        return cycle_group(constant_acc);
    }

    // add the constant component into our offset accumulator
    // (we'll subtract `offset_accumulator` from the MSM output i.e. we negate here to counter the future negation)
    Element offset_accumulator = -constant_acc;
    const bool has_variable_points = !variable_base_points.empty();
    const bool has_fixed_points = !fixed_base_points.empty();

    // Compute all required offset generators.
    const size_t num_offset_generators =
        variable_base_points.size() + fixed_base_points.size() + has_variable_points + has_fixed_points;
    const std::span<AffineElement const> offset_generators =
        context.generators->get(num_offset_generators, 0, OFFSET_GENERATOR_DOMAIN_SEPARATOR);

    cycle_group result;
    if (has_fixed_points) {
        const auto [fixed_accumulator, offset_generator_delta] =
            _fixed_base_batch_mul_internal(fixed_base_scalars, fixed_base_points, offset_generators);
        offset_accumulator += offset_generator_delta;
        result = fixed_accumulator;
    }

    if (has_variable_points) {
        std::span<AffineElement const> offset_generators_for_variable_base_batch_mul{
            offset_generators.data() + fixed_base_points.size(), offset_generators.size() - fixed_base_points.size()
        };
        const auto [variable_accumulator, offset_generator_delta] =
            _variable_base_batch_mul_internal(variable_base_scalars,
                                              variable_base_points,
                                              offset_generators_for_variable_base_batch_mul,
                                              can_unconditional_add);
        offset_accumulator += offset_generator_delta;
        if (has_fixed_points) {
            result = can_unconditional_add ? result.unconditional_add(variable_accumulator)
                                           : result.checked_unconditional_add(variable_accumulator);
        } else {
            result = variable_accumulator;
        }
    }

    // Update `result` to remove the offset generator terms, and add in any constant terms from `constant_acc`.
    // We have two potential modes here:
    // 1. All inputs are fixed-base and we constant_acc is not the point at infinity
    // 2. Everything else.
    // Case 1 is a special case, as we *know* we cannot hit incomplete addition edge cases,
    // under the assumption that all input points are linearly independent of one another.
    // Because constant_acc is not the point at infnity we know that at least 1 input scalar was not zero,
    // i.e. the output will not be the point at infinity. We also know under case 1, we won't trigger the
    // doubling formula either, as every point is lienarly independent of every other point (including offset
    // generators).
    if (!constant_acc.is_point_at_infinity() && can_unconditional_add) {
        result = result.unconditional_add(AffineElement(-offset_accumulator));
    } else {
        // For case 2, we must use a full subtraction operation that handles all possible edge cases, as the output
        // point may be the point at infinity.
        // TODO(@zac-williamson) We can probably optimize this a bit actually. We might hit the point at infinity,
        // but an honest prover won't trigger the doubling edge case.
        // (doubling edge case implies input points are also the offset generator points,
        // which we can assume an honest Prover will not do if we make this case produce unsatisfiable constraints)
        // We could do the following:
        // 1. If x-coords match, assert y-coords do not match
        // 2. If x-coords match, return point at infinity, else return result - offset_accumulator.
        // This would be slightly cheaper than operator- as we do not have to evaluate the double edge case.
        result = result - AffineElement(offset_accumulator);
    }
    return result;
}

template <typename Composer> cycle_group<Composer> cycle_group<Composer>::operator*(const cycle_scalar& scalar) const
{
    return batch_mul({ scalar }, { *this });
}

template <typename Composer> cycle_group<Composer>& cycle_group<Composer>::operator*=(const cycle_scalar& scalar)
{
    *this = operator*(scalar);
    return *this;
}

template <typename Composer> bool_t<Composer> cycle_group<Composer>::operator==(const cycle_group& other) const
{
    const auto equal_and_not_infinity =
        (x == other.x) && (y == other.y) && !is_point_at_infinity() && !other.is_point_at_infinity();
    const auto both_infinity = is_point_at_infinity() && other.is_point_at_infinity();
    return equal_and_not_infinity || both_infinity;
}

template <typename Composer>
void cycle_group<Composer>::assert_equal(const cycle_group& other, std::string const& msg) const
{
    x.assert_equal(other.x, msg);
    y.assert_equal(other.y, msg);
}

template <typename Composer>
cycle_group<Composer> cycle_group<Composer>::conditional_assign(const bool_t& predicate,
                                                                const cycle_group& lhs,
                                                                const cycle_group& rhs)
{
    return { field_t::conditional_assign(predicate, lhs.x, rhs.x),
             field_t::conditional_assign(predicate, lhs.y, rhs.y),
             bool_t::conditional_assign(predicate, lhs.is_point_at_infinity(), rhs.is_point_at_infinity()) };
};
template <typename Composer> cycle_group<Composer> cycle_group<Composer>::operator/(const cycle_group& /*unused*/) const
{
    // TODO(@kevaundray solve the discrete logarithm problem)
    throw_or_abort("Implementation under construction...");
}

template class cycle_group<bb::StandardCircuitBuilder>;
template class cycle_group<bb::UltraCircuitBuilder>;
template class cycle_group<bb::GoblinUltraCircuitBuilder>;

} // namespace bb::stdlib
