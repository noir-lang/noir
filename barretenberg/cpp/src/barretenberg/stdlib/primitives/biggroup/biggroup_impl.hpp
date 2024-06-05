#pragma once

#include "../bit_array/bit_array.hpp"
#include "../circuit_builders/circuit_builders.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"

namespace bb::stdlib {

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element()
    : x()
    , y()
    , _is_infinity()
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(const typename G::affine_element& input)
    : x(nullptr, input.x)
    , y(nullptr, input.y)
    , _is_infinity(nullptr, input.is_point_at_infinity())
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(const Fq& x_in, const Fq& y_in)
    : x(x_in)
    , y(y_in)
    , _is_infinity(x.get_context() ? x.get_context() : y.get_context(), false)
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(const element& other)
    : x(other.x)
    , y(other.y)
    , _is_infinity(other.is_point_at_infinity())
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(element&& other) noexcept
    : x(other.x)
    , y(other.y)
    , _is_infinity(other.is_point_at_infinity())
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>& element<C, Fq, Fr, G>::operator=(const element& other)
{
    if (&other == this) {
        return *this;
    }
    x = other.x;
    y = other.y;
    _is_infinity = other.is_point_at_infinity();
    return *this;
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>& element<C, Fq, Fr, G>::operator=(element&& other) noexcept
{
    if (&other == this) {
        return *this;
    }
    x = other.x;
    y = other.y;
    _is_infinity = other.is_point_at_infinity();
    return *this;
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::operator+(const element& other) const
{
    // return checked_unconditional_add(other);
    if constexpr (IsMegaBuilder<C> && std::same_as<G, bb::g1>) {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/707) Optimize
        // Current gate count: 6398
        std::vector<element> points{ *this, other };
        std::vector<Fr> scalars{ 1, 1 };
        return batch_mul(points, scalars);
    }

    // Adding in `x_coordinates_match` ensures that lambda will always be well-formed
    // Our curve has the form y^2 = x^3 + b.
    // If (x_1, y_1), (x_2, y_2) have x_1 == x_2, and the generic formula for lambda has a division by 0.
    // Then y_1 == y_2 (i.e. we are doubling) or y_2 == y_1 (the sum is infinity).
    // The cases have a special addition formula. The following booleans allow us to handle these cases uniformly.
    const bool_ct x_coordinates_match = other.x == x;
    const bool_ct y_coordinates_match = (y == other.y);
    const bool_ct infinity_predicate = (x_coordinates_match && !y_coordinates_match);
    const bool_ct double_predicate = (x_coordinates_match && y_coordinates_match);
    const bool_ct lhs_infinity = is_point_at_infinity();
    const bool_ct rhs_infinity = other.is_point_at_infinity();

    // Compute the gradient `lambda`. If we add, `lambda = (y2 - y1)/(x2 - x1)`, else `lambda = 3x1*x1/2y1
    const Fq add_lambda_numerator = other.y - y;
    const Fq xx = x * x;
    const Fq dbl_lambda_numerator = xx + xx + xx;
    const Fq lambda_numerator = Fq::conditional_assign(double_predicate, dbl_lambda_numerator, add_lambda_numerator);

    const Fq add_lambda_denominator = other.x - x;
    const Fq dbl_lambda_denominator = y + y;
    Fq lambda_denominator = Fq::conditional_assign(double_predicate, dbl_lambda_denominator, add_lambda_denominator);
    // If either inputs are points at infinity, we set lambda_denominator to be 1. This ensures we never trigger a
    // divide by zero error.
    // Note: if either inputs are points at infinity we will not use the result of this computation.
    Fq safe_edgecase_denominator = Fq(field_t<C>(1), field_t<C>(0), field_t<C>(0), field_t<C>(0));
    lambda_denominator = Fq::conditional_assign(
        lhs_infinity || rhs_infinity || infinity_predicate, safe_edgecase_denominator, lambda_denominator);
    const Fq lambda = Fq::div_without_denominator_check({ lambda_numerator }, lambda_denominator);

    const Fq x3 = lambda.sqradd({ -other.x, -x });
    const Fq y3 = lambda.madd(x - x3, { -y });

    element result(x3, y3);
    // if lhs infinity, return rhs
    result.x = Fq::conditional_assign(lhs_infinity, other.x, result.x);
    result.y = Fq::conditional_assign(lhs_infinity, other.y, result.y);
    // if rhs infinity, return lhs
    result.x = Fq::conditional_assign(rhs_infinity, x, result.x);
    result.y = Fq::conditional_assign(rhs_infinity, y, result.y);

    // is result point at infinity?
    // yes = infinity_predicate && !lhs_infinity && !rhs_infinity
    // yes = lhs_infinity && rhs_infinity
    // n.b. can likely optimize this
    bool_ct result_is_infinity = infinity_predicate && (!lhs_infinity && !rhs_infinity);
    result_is_infinity = result_is_infinity || (lhs_infinity && rhs_infinity);
    result.set_point_at_infinity(result_is_infinity);
    return result;
}

/**
 * @brief Enforce x and y coordinates of a point to be (0,0) in the case of point at infinity
 *
 * @details We need to have a standard witness in Noir and the point at infinity can have non-zero random coefficients
 * when we get it as output from our optimised algorithms. This function returns a (0,0) point, if it is a point at
 * infinity
 */
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::get_standard_form() const
{

    const bool_ct is_infinity = is_point_at_infinity();
    element result(*this);
    const Fq zero = Fq::zero();
    result.x = Fq::conditional_assign(is_infinity, zero, this->x);
    result.y = Fq::conditional_assign(is_infinity, zero, this->y);
    return result;
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::operator-(const element& other) const
{
    // return checked_unconditional_add(other);
    if constexpr (IsMegaBuilder<C> && std::same_as<G, bb::g1>) {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/707) Optimize
        // Current gate count: 6398
        std::vector<element> points{ *this, other };
        std::vector<Fr> scalars{ 1, -Fr(1) };
        return batch_mul(points, scalars);
    }

    // if x_coordinates match, lambda triggers a divide by zero error.
    // Adding in `x_coordinates_match` ensures that lambda will always be well-formed
    const bool_ct x_coordinates_match = other.x == x;
    const bool_ct y_coordinates_match = (y == other.y);
    const bool_ct infinity_predicate = (x_coordinates_match && y_coordinates_match);
    const bool_ct double_predicate = (x_coordinates_match && !y_coordinates_match);
    const bool_ct lhs_infinity = is_point_at_infinity();
    const bool_ct rhs_infinity = other.is_point_at_infinity();

    // Compute the gradient `lambda`. If we add, `lambda = (y2 - y1)/(x2 - x1)`, else `lambda = 3x1*x1/2y1
    const Fq add_lambda_numerator = -other.y - y;
    const Fq xx = x * x;
    const Fq dbl_lambda_numerator = xx + xx + xx;
    const Fq lambda_numerator = Fq::conditional_assign(double_predicate, dbl_lambda_numerator, add_lambda_numerator);

    const Fq add_lambda_denominator = other.x - x;
    const Fq dbl_lambda_denominator = y + y;
    Fq lambda_denominator = Fq::conditional_assign(double_predicate, dbl_lambda_denominator, add_lambda_denominator);
    // If either inputs are points at infinity, we set lambda_denominator to be 1. This ensures we never trigger a
    // divide by zero error.
    // (if either inputs are points at infinity we will not use the result of this computation)
    Fq safe_edgecase_denominator = Fq(field_t<C>(1), field_t<C>(0), field_t<C>(0), field_t<C>(0));
    lambda_denominator = Fq::conditional_assign(
        lhs_infinity || rhs_infinity || infinity_predicate, safe_edgecase_denominator, lambda_denominator);
    const Fq lambda = Fq::div_without_denominator_check({ lambda_numerator }, lambda_denominator);

    const Fq x3 = lambda.sqradd({ -other.x, -x });
    const Fq y3 = lambda.madd(x - x3, { -y });

    element result(x3, y3);
    // if lhs infinity, return rhs
    result.x = Fq::conditional_assign(lhs_infinity, other.x, result.x);
    result.y = Fq::conditional_assign(lhs_infinity, -other.y, result.y);
    // if rhs infinity, return lhs
    result.x = Fq::conditional_assign(rhs_infinity, x, result.x);
    result.y = Fq::conditional_assign(rhs_infinity, y, result.y);

    // is result point at infinity?
    // yes = infinity_predicate && !lhs_infinity && !rhs_infinity
    // yes = lhs_infinity && rhs_infinity
    // n.b. can likely optimize this
    bool_ct result_is_infinity = infinity_predicate && (!lhs_infinity && !rhs_infinity);
    result_is_infinity = result_is_infinity || (lhs_infinity && rhs_infinity);
    result.set_point_at_infinity(result_is_infinity);
    return result;
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::checked_unconditional_add(const element& other) const
{
    if constexpr (IsMegaBuilder<C> && std::same_as<G, bb::g1>) {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/707) Optimize
        // Current gate count: 6398
        std::vector<element> points{ *this, other };
        std::vector<Fr> scalars{ 1, 1 };
        return goblin_batch_mul(points, scalars);
    }

    other.x.assert_is_not_equal(x);
    const Fq lambda = Fq::div_without_denominator_check({ other.y, -y }, (other.x - x));
    const Fq x3 = lambda.sqradd({ -other.x, -x });
    const Fq y3 = lambda.madd(x - x3, { -y });
    return element(x3, y3);
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::checked_unconditional_subtract(const element& other) const
{
    if constexpr (IsMegaBuilder<C> && std::same_as<G, bb::g1>) {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/707) Optimize
        std::vector<element> points{ *this, other };
        std::vector<Fr> scalars{ 1, -Fr(1) };
        return goblin_batch_mul(points, scalars);
    }

    other.x.assert_is_not_equal(x);
    const Fq lambda = Fq::div_without_denominator_check({ other.y, y }, (other.x - x));
    const Fq x_3 = lambda.sqradd({ -other.x, -x });
    const Fq y_3 = lambda.madd(x_3 - x, { -y });

    return element(x_3, y_3);
}

/**
 * @brief Compute (*this) + other AND (*this) - other as a size-2 array
 *
 * @details We require this operation when computing biggroup lookup tables for
 *          multi-scalar-multiplication. This combined method reduces the number of
 *          field additions, field subtractions required (as well as 1 less assert_is_not_equal check)
 *
 * @tparam C
 * @tparam Fq
 * @tparam Fr
 * @tparam G
 * @param other
 * @return std::array<element<C, Fq, Fr, G>, 2>
 */
// TODO(https://github.com/AztecProtocol/barretenberg/issues/657): This function is untested
template <typename C, class Fq, class Fr, class G>
std::array<element<C, Fq, Fr, G>, 2> element<C, Fq, Fr, G>::checked_unconditional_add_sub(const element& other) const
{
    if constexpr (IsMegaBuilder<C> && std::same_as<G, bb::g1>) {
        return { *this + other, *this - other };
    }

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/971): This will fail when the two elements are the same
    // even in the case of a valid circuit
    other.x.assert_is_not_equal(x);

    const Fq denominator = other.x - x;
    const Fq x2x1 = -(other.x + x);

    const Fq lambda1 = Fq::div_without_denominator_check({ other.y, -y }, denominator);
    const Fq x_3 = lambda1.sqradd({ x2x1 });
    const Fq y_3 = lambda1.madd(x - x_3, { -y });
    const Fq lambda2 = Fq::div_without_denominator_check({ -other.y, -y }, denominator);
    const Fq x_4 = lambda2.sqradd({ x2x1 });
    const Fq y_4 = lambda2.madd(x - x_4, { -y });

    return { element(x_3, y_3), element(x_4, y_4) };
}

template <typename C, class Fq, class Fr, class G> element<C, Fq, Fr, G> element<C, Fq, Fr, G>::dbl() const
{

    Fq two_x = x + x;
    if constexpr (G::has_a) {
        Fq a(get_context(), uint256_t(G::curve_a));
        Fq neg_lambda = Fq::msub_div({ x }, { (two_x + x) }, (y + y), { a });
        Fq x_3 = neg_lambda.sqradd({ -(two_x) });
        Fq y_3 = neg_lambda.madd(x_3 - x, { -y });
        return element(x_3, y_3);
    }
    Fq neg_lambda = Fq::msub_div({ x }, { (two_x + x) }, (y + y), {});
    Fq x_3 = neg_lambda.sqradd({ -(two_x) });
    Fq y_3 = neg_lambda.madd(x_3 - x, { -y });
    element result = element(x_3, y_3);
    result.set_point_at_infinity(is_point_at_infinity());
    return result;
}

/**
 * Evaluate a chain addition!
 *
 * When adding a set of points P_1 + ... + P_N, we do not need to compute the y-coordinate of intermediate addition
 *terms.
 *
 * i.e. we substitute `acc.y` with `acc.y = acc.lambda_prev * (acc.x1_prev - acc.x) - acc.y1_prev`
 *
 * `lambda_prev, x1_prev, y1_prev` are the `lambda, x1, y1` terms from the previous addition operation.
 *
 * `chain_add` requires 1 less non-native field reduction than a regular add operation.
 **/

/**
 * begin a chain of additions
 * input points p1 p2
 * output accumulator = x3_prev (output x coordinate), x1_prev, y1_prev (p1), lambda_prev (y2 - y1) / (x2 - x1)
 **/
template <typename C, class Fq, class Fr, class G>
typename element<C, Fq, Fr, G>::chain_add_accumulator element<C, Fq, Fr, G>::chain_add_start(const element& p1,
                                                                                             const element& p2)
    requires(IsNotGoblinInefficiencyTrap<C, G>)
{
    chain_add_accumulator output;
    output.x1_prev = p1.x;
    output.y1_prev = p1.y;

    p1.x.assert_is_not_equal(p2.x);
    const Fq lambda = Fq::div_without_denominator_check({ p2.y, -p1.y }, (p2.x - p1.x));

    const Fq x3 = lambda.sqradd({ -p2.x, -p1.x });
    output.x3_prev = x3;
    output.lambda_prev = lambda;
    return output;
}

template <typename C, class Fq, class Fr, class G>
typename element<C, Fq, Fr, G>::chain_add_accumulator element<C, Fq, Fr, G>::chain_add(const element& p1,
                                                                                       const chain_add_accumulator& acc)
    requires(IsNotGoblinInefficiencyTrap<C, G>)
{
    // use `chain_add_start` to start an addition chain (i.e. if acc has a y-coordinate)
    if (acc.is_element) {
        return chain_add_start(p1, element(acc.x3_prev, acc.y3_prev));
    }
    // validate we can use incomplete addition formulae
    p1.x.assert_is_not_equal(acc.x3_prev);

    // lambda = (y2 - y1) / (x2 - x1)
    // but we don't have y2!
    // however, we do know that y2 = lambda_prev * (x1_prev - x2) - y1_prev
    // => lambda * (x2 - x1) = lambda_prev * (x1_prev - x2) - y1_prev - y1
    // => lambda * (x2 - x1) + lambda_prev * (x2 - x1_prev) + y1 + y1_pev = 0
    // => lambda = lambda_prev * (x1_prev - x2) - y1_prev - y1 / (x2 - x1)
    // => lambda = - (lambda_prev * (x2 - x1_prev) + y1_prev + y1) / (x2 - x1)

    /**
     *
     * We compute the following terms:
     *
     * lambda = acc.lambda_prev * (acc.x1_prev - acc.x) - acc.y1_prev - p1.y / acc.x - p1.x
     * x3 = lambda * lambda - acc.x - p1.x
     *
     * Requires only 2 non-native field reductions
     **/
    auto& x2 = acc.x3_prev;
    const auto lambda = Fq::msub_div({ acc.lambda_prev }, { (x2 - acc.x1_prev) }, (x2 - p1.x), { acc.y1_prev, p1.y });
    const auto x3 = lambda.sqradd({ -x2, -p1.x });

    chain_add_accumulator output;
    output.x3_prev = x3;
    output.x1_prev = p1.x;
    output.y1_prev = p1.y;
    output.lambda_prev = lambda;

    return output;
}

/**
 * End an addition chain. Produces a full output group element with a y-coordinate
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::chain_add_end(const chain_add_accumulator& acc)
    requires(IsNotGoblinInefficiencyTrap<C, G>)
{
    if (acc.is_element) {
        return element(acc.x3_prev, acc.y3_prev);
    }
    auto& x3 = acc.x3_prev;
    auto& lambda = acc.lambda_prev;

    Fq y3 = lambda.madd((acc.x1_prev - x3), { -acc.y1_prev });
    return element(x3, y3);
}
/**
 * Compute one round of a Montgomery ladder: i.e. compute 2 * (*this) + other
 * Compute D = A + B + A, where A = `this` and B = `other`
 *
 * We can skip computing the y-coordinate of C = A + B:
 *
 * To compute D = A + C, A=(x_1,y_1), we need the gradient of our two coordinates, specifically:
 *
 *
 *               y_3 - y_1    lambda_1 * (x_1 - x_3) - 2 * y_1                 2 * y_1
 *  lambda_2 =  __________ =  ________________________________ = -\lambda_1 - _________
 *               x_3 - x_1              x_3 - x_1                             x_3 - x_1
 *
 * We don't need y_3 to compute this. We can then compute D.x and D.y as usual:
 *
 *  D.x = lambda_2 * lambda_2 - (C.x + A.x)
 *  D.y = lambda_2 * (A.x - D.x) - A.y
 *
 * Requires 5 non-native field reductions. Doubling and adding would require 6
 **/

// #################################
// ### SCALAR MULTIPLICATION METHODS
// #################################
/**
 * Compute D = A + B + A, where A = `this` and B = `other`
 *
 * We can skip computing the y-coordinate of C = A + B:
 *
 * To compute D = A + C, A=(x_1,y_1), we need the gradient of our two coordinates, specifically:
 *
 *
 *               y_3 - y_1    lambda_1 * (x_1 - x_3) - 2 * y_1                 2 * y_1
 *  lambda_2 =  __________ =  ________________________________ = -\lambda_1 - _________
 *               x_3 - x_1              x_3 - x_1                             x_3 - x_1
 *
 * We don't need y_3 to compute this. We can then compute D.x and D.y as usual:
 *
 *  D.x = lambda_2 * lambda_2 - (C.x + A.x)
 *  D.y = lambda_2 * (A.x - D.x) - A.y
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::montgomery_ladder(const element& other) const
    requires(IsNotGoblinInefficiencyTrap<C, G>)
{
    other.x.assert_is_not_equal(x);
    const Fq lambda_1 = Fq::div_without_denominator_check({ other.y - y }, (other.x - x));

    const Fq x_3 = lambda_1.sqradd({ -other.x, -x });

    const Fq minus_lambda_2 = lambda_1 + Fq::div_without_denominator_check({ y + y }, (x_3 - x));

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    const Fq y_4 = minus_lambda_2.madd(x_4 - x, { -y });
    return element(x_4, y_4);
}

/**
 * Implementation of `montgomery_ladder` using chain_add_accumulator.
 *
 * If the input to `montgomery_ladder` is the output of a chain of additions,
 * we can avoid computing the y-coordinate of the input `to_add`, which saves us a non-native field reduction.
 *
 * We substitute `to_add.y` with `lambda_prev * (to_add.x1_prev - to_add.x) - to_add.y1_prev`
 *
 * Here, `x1_prev, y1_prev, lambda_prev` are the values of `x1, y1, lambda` for the addition operation that PRODUCED
 *to_add
 *
 * The reason why this saves us gates, is because the montgomery ladder does not multiply to_add.y by any values
 * i.e. to_add.y is only used in addition operations
 *
 * This allows us to substitute to_add.y with the above relation without requiring additional field reductions
 *
 * e.g. the term (lambda * (x3 - x1) + to_add.y) remains "quadratic" if we replace to_add.y with the above quadratic
 *relation
 *
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::montgomery_ladder(const chain_add_accumulator& to_add)
    requires(IsNotGoblinInefficiencyTrap<C, G>)
{
    if (to_add.is_element) {
        throw_or_abort("An accumulator expected");
    }
    x.assert_is_not_equal(to_add.x3_prev);

    // lambda = (y2 - y1) / (x2 - x1)
    // but we don't have y2!
    // however, we do know that y2 = lambda_prev * (x1_prev - x2) - y1_prev
    // => lambda * (x2 - x1) = lambda_prev * (x1_prev - x2) - y1_prev - y1
    // => lambda * (x2 - x1) + lambda_prev * (x2 - x1_prev) + y1 + y1_pev = 0
    // => lambda = lambda_prev * (x1_prev - x2) - y1_prev - y1 / (x2 - x1)
    // => lambda = - (lambda_prev * (x2 - x1_prev) + y1_prev + y1) / (x2 - x1)

    auto& x2 = to_add.x3_prev;
    const auto lambda =
        Fq::msub_div({ to_add.lambda_prev }, { (x2 - to_add.x1_prev) }, (x2 - x), { to_add.y1_prev, y });
    const auto x3 = lambda.sqradd({ -x2, -x });

    const Fq minus_lambda_2 = lambda + Fq::div_without_denominator_check({ y + y }, (x3 - x));

    const Fq x4 = minus_lambda_2.sqradd({ -x, -x3 });

    const Fq y4 = minus_lambda_2.madd(x4 - x, { -y });
    return element(x4, y4);
}

/**
 * @brief Compute 4.P + to_add[0] + ... + to_add[to_add.size() - 1]
 *
 * @details Used in wnaf_batch_mul method. Combining operations requires fewer bigfield reductions.
 *
 * Method computes R[i] = (2P + A[0]) + (2P + A[1]) + A[2] + ... + A[n-1]
 *
 * @tparam C
 * @tparam Fq
 * @tparam Fr
 * @tparam G
 * @param to_add
 * @return element<C, Fq, Fr, G>
 */
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::quadruple_and_add(const std::vector<element>& to_add) const
    requires(IsNotGoblinInefficiencyTrap<C, G>)
{
    const Fq two_x = x + x;
    Fq x_1;
    Fq minus_lambda_dbl;
    if constexpr (G::has_a) {
        Fq a(get_context(), uint256_t(G::curve_a));
        minus_lambda_dbl = Fq::msub_div({ x }, { (two_x + x) }, (y + y), { a });
        x_1 = minus_lambda_dbl.sqradd({ -(two_x) });
    } else {
        minus_lambda_dbl = Fq::msub_div({ x }, { (two_x + x) }, (y + y), {});
        x_1 = minus_lambda_dbl.sqradd({ -(two_x) });
    }

    ASSERT(to_add.size() > 0);
    to_add[0].x.assert_is_not_equal(x_1);

    const Fq x_minus_x_1 = x - x_1;

    const Fq lambda_1 = Fq::msub_div({ minus_lambda_dbl }, { x_minus_x_1 }, (x_1 - to_add[0].x), { to_add[0].y, y });

    const Fq x_3 = lambda_1.sqradd({ -to_add[0].x, -x_1 });

    const Fq half_minus_lambda_2_minus_lambda_1 =
        Fq::msub_div({ minus_lambda_dbl }, { x_minus_x_1 }, (x_3 - x_1), { y });

    const Fq minus_lambda_2_minus_lambda_1 = half_minus_lambda_2_minus_lambda_1 + half_minus_lambda_2_minus_lambda_1;
    const Fq minus_lambda_2 = minus_lambda_2_minus_lambda_1 + lambda_1;

    const Fq x_4 = minus_lambda_2.sqradd({ -x_1, -x_3 });

    const Fq x_4_sub_x_1 = x_4 - x_1;

    if (to_add.size() == 1) {
        const Fq y_4 = Fq::dual_madd(minus_lambda_2, x_4_sub_x_1, minus_lambda_dbl, x_minus_x_1, { y });
        return element(x_4, y_4);
    }
    to_add[1].x.assert_is_not_equal(to_add[0].x);

    Fq minus_lambda_3 = Fq::msub_div(
        { minus_lambda_dbl, minus_lambda_2 }, { x_minus_x_1, x_4_sub_x_1 }, (x_4 - to_add[1].x), { y, -(to_add[1].y) });

    // X5 = L3.L3 - X4 - XB
    const Fq x_5 = minus_lambda_3.sqradd({ -x_4, -to_add[1].x });

    if (to_add.size() == 2) {
        // Y5 = L3.(XB - X5) - YB
        const Fq y_5 = minus_lambda_3.madd(x_5 - to_add[1].x, { -to_add[1].y });
        return element(x_5, y_5);
    }

    Fq x_prev = x_5;
    Fq minus_lambda_prev = minus_lambda_3;

    for (size_t i = 2; i < to_add.size(); ++i) {

        to_add[i].x.assert_is_not_equal(to_add[i - 1].x);
        // Lambda = Yprev - Yadd[i] / Xprev - Xadd[i]
        //        = -Lprev.(Xprev - Xadd[i-1]) - Yadd[i - 1] - Yadd[i] / Xprev - Xadd[i]
        const Fq minus_lambda = Fq::msub_div({ minus_lambda_prev },
                                             { to_add[i - 1].x - x_prev },
                                             (to_add[i].x - x_prev),
                                             { to_add[i - 1].y, to_add[i].y });
        // X = Lambda * Lambda - Xprev - Xadd[i]
        const Fq x_out = minus_lambda.sqradd({ -x_prev, -to_add[i].x });

        x_prev = x_out;
        minus_lambda_prev = minus_lambda;
    }
    const Fq y_out = minus_lambda_prev.madd(x_prev - to_add[to_add.size() - 1].x, { -to_add[to_add.size() - 1].y });

    return element(x_prev, y_out);
}

/**
 * @brief Perform repeated iterations of the montgomery ladder algorithm.
 *
 * For points P, Q, montgomery ladder computes R = (P + Q) + P
 * i.e. it's "double-and-add" without explicit doublings.
 *
 * This method can apply repeated iterations of the montgomery ladder.
 * Each iteration reduces the number of field multiplications by 1, at the cost of more additions.
 * (i.e. we don't compute intermediate y-coordinates).
 *
 * The number of additions scales with the size of the input vector. The optimal input size appears to be 4.
 *
 * @tparam C
 * @tparam Fq
 * @tparam Fr
 * @tparam G
 * @param add
 * @return element<C, Fq, Fr, G>
 */
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::multiple_montgomery_ladder(
    const std::vector<chain_add_accumulator>& add) const
    requires(IsNotGoblinInefficiencyTrap<C, G>)
{
    struct composite_y {
        std::vector<Fq> mul_left;
        std::vector<Fq> mul_right;
        std::vector<Fq> add;
        bool is_negative = false;
    };

    Fq previous_x = x;
    composite_y previous_y{ std::vector<Fq>(), std::vector<Fq>(), std::vector<Fq>(), false };
    for (size_t i = 0; i < add.size(); ++i) {
        previous_x.assert_is_not_equal(add[i].x3_prev);

        // composite_y add_y;
        bool negate_add_y = (i > 0) && !previous_y.is_negative;
        std::vector<Fq> lambda1_left;
        std::vector<Fq> lambda1_right;
        std::vector<Fq> lambda1_add;

        if (i == 0) {
            lambda1_add.emplace_back(-y);
        } else {
            lambda1_left = previous_y.mul_left;
            lambda1_right = previous_y.mul_right;
            lambda1_add = previous_y.add;
        }

        if (!add[i].is_element) {
            lambda1_left.emplace_back(add[i].lambda_prev);
            lambda1_right.emplace_back(negate_add_y ? add[i].x3_prev - add[i].x1_prev
                                                    : add[i].x1_prev - add[i].x3_prev);
            lambda1_add.emplace_back(negate_add_y ? add[i].y1_prev : -add[i].y1_prev);
        } else if (i > 0) {
            lambda1_add.emplace_back(negate_add_y ? -add[i].y3_prev : add[i].y3_prev);
        }
        // if previous_y is negated then add stays positive
        // if previous_y is positive then add stays negated
        // | add.y is negated | previous_y is negated | output of msub_div is -lambda |
        // | --- | --- | --- |
        // | no  | yes | yes |
        // | yes | no  | no  |

        Fq lambda1;
        if (!add[i].is_element || i > 0) {
            bool flip_lambda1_denominator = !negate_add_y;
            Fq denominator = flip_lambda1_denominator ? previous_x - add[i].x3_prev : add[i].x3_prev - previous_x;
            lambda1 = Fq::msub_div(lambda1_left, lambda1_right, denominator, lambda1_add);
        } else {
            lambda1 = Fq::div_without_denominator_check({ add[i].y3_prev - y }, (add[i].x3_prev - x));
        }

        Fq x_3 = lambda1.madd(lambda1, { -add[i].x3_prev, -previous_x });

        // We can avoid computing y_4, instead substituting the expression `minus_lambda_2 * (x_4 - x) - y` where
        // needed. This is cheaper, because we can evaluate two field multiplications (or a field multiplication + a
        // field division) with only one non-native field reduction. E.g. evaluating (a * b) + (c * d) = e mod p only
        // requires 1 quotient and remainder, which is the major cost of a non-native field multiplication
        Fq lambda2;
        if (i == 0) {
            lambda2 = Fq::div_without_denominator_check({ y + y }, (previous_x - x_3)) - lambda1;
        } else {
            Fq l2_denominator = previous_y.is_negative ? previous_x - x_3 : x_3 - previous_x;
            Fq partial_lambda2 =
                Fq::msub_div(previous_y.mul_left, previous_y.mul_right, l2_denominator, previous_y.add);
            partial_lambda2 = partial_lambda2 + partial_lambda2;
            lambda2 = partial_lambda2 - lambda1;
        }

        Fq x_4 = lambda2.sqradd({ -x_3, -previous_x });
        composite_y y_4;
        if (i == 0) {
            // We want to make sure that at the final iteration, `y_previous.is_negative = false`
            // Each iteration flips the sign of y_previous.is_negative.
            // i.e. whether we store y_4 or -y_4 depends on the number of points we have
            bool num_points_even = ((add.size() & 0x01UL) == 0);
            y_4.add.emplace_back(num_points_even ? y : -y);
            y_4.mul_left.emplace_back(lambda2);
            y_4.mul_right.emplace_back(num_points_even ? x_4 - previous_x : previous_x - x_4);
            y_4.is_negative = num_points_even;
        } else {
            y_4.is_negative = !previous_y.is_negative;
            y_4.mul_left.emplace_back(lambda2);
            y_4.mul_right.emplace_back(previous_y.is_negative ? previous_x - x_4 : x_4 - previous_x);
            // append terms in previous_y to y_4. We want to make sure the terms above are added into the start of y_4.
            // This is to ensure they are cached correctly when
            // `builder::evaluate_partial_non_native_field_multiplication` is called.
            // (the 1st mul_left, mul_right elements will trigger builder::evaluate_non_native_field_multiplication
            //  when Fq::mult_madd is called - this term cannot be cached so we want to make sure it is unique)
            std::copy(previous_y.mul_left.begin(), previous_y.mul_left.end(), std::back_inserter(y_4.mul_left));
            std::copy(previous_y.mul_right.begin(), previous_y.mul_right.end(), std::back_inserter(y_4.mul_right));
            std::copy(previous_y.add.begin(), previous_y.add.end(), std::back_inserter(y_4.add));
        }
        previous_x = x_4;
        previous_y = y_4;
    }
    Fq x_out = previous_x;

    ASSERT(!previous_y.is_negative);

    Fq y_out = Fq::mult_madd(previous_y.mul_left, previous_y.mul_right, previous_y.add);
    return element(x_out, y_out);
}

/**
 * compute_offset_generators! Let's explain what an offset generator is...
 *
 * We evaluate biggroup group operations using INCOMPLETE addition formulae for short weierstrass curves:
 *
 * L   = y - y  / x  - x
 *        2   1    2    1
 *
 *          2
 * x   =   L  - x  - x
 *  3            2    1
 *
 * y   =  L (x  - x ) - y
 *  3         1    3     1
 *
 * These formuale do not work for the edge case where x2 == x1
 *
 * Instead of handling the edge case (which is expensive!) we instead FORBID it from happening by
 * requiring x2 != x1 (other.x.assert_is_not_equal(x) will be present in all group operation methods)
 *
 * This means it is essential we ensure an honest prover will NEVER run into this edge case, or our circuit will lack
 * completeness!
 *
 * To ensure an honest prover will not fall foul of this edge case when performing a SCALAR MULTIPLICATION,
 * we init the accumulator with an `offset_generator` point.
 * This point is a generator point that is not equal to the regular generator point for this curve.
 *
 * When adding points into the accumulator, the probability that an honest prover will find a collision is now ~ 1 in
 * 2^128
 *
 * We init `accumulator = generator` and then perform an n-bit scalar mul.
 * The output accumulator will contain a term `2^{n-1} * generator` that we need to subtract off.
 *
 * `offset_generators.first = generator` (the initial generator point)
 * `offset_generators.second = 2^{n-1} * generator` (the final generator point we need to subtract off from our
 * accumulator)
 */
template <typename C, class Fq, class Fr, class G>
std::pair<element<C, Fq, Fr, G>, element<C, Fq, Fr, G>> element<C, Fq, Fr, G>::compute_offset_generators(
    const size_t num_rounds)
{
    constexpr typename G::affine_element offset_generator = G::derive_generators("biggroup offset generator", 1)[0];

    const uint256_t offset_multiplier = uint256_t(1) << uint256_t(num_rounds - 1);

    const typename G::affine_element offset_generator_end = typename G::element(offset_generator) * offset_multiplier;
    return std::make_pair<element, element>(offset_generator, offset_generator_end);
}

/**
 * @brief Generic batch multiplication that works for all elliptic curve types.
 *
 * @details Implementation is identical to `bn254_endo_batch_mul` but WITHOUT the endomorphism transforms OR support for
 * short scalars See `bn254_endo_batch_mul` for description of algorithm.
 *
 * @tparam C The circuit builder type.
 * @tparam Fq The field of definition of the points in `_points`.
 * @tparam Fr The field of scalars acting on `_points`.
 * @tparam G The group whose arithmetic is emulated by `element`.
 * @param _points
 * @param _scalars
 * @param max_num_bits The max of the bit lengths of the scalars.
 * @param with_edgecases Use when points are linearly dependent. Randomises them.
 * @return element<C, Fq, Fr, G>
 */
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::batch_mul(const std::vector<element>& _points,
                                                       const std::vector<Fr>& _scalars,
                                                       const size_t max_num_bits,
                                                       const bool with_edgecases)
{
    auto [points, scalars] = handle_points_at_infinity(_points, _scalars);

    if constexpr (IsSimulator<C>) {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/663)
        auto context = points[0].get_context();
        using element_t = typename G::element;
        element_t result = G::one;
        result.self_set_infinity();
        for (size_t i = 0; i < points.size(); i++) {
            result += (element_t(points[i].get_value()) * scalars[i].get_value());
        }
        result = result.normalize();
        return from_witness(context, result);
    } else {
        // Perform goblinized batched mul if available; supported only for BN254
        if constexpr (IsMegaBuilder<C> && std::same_as<G, bb::g1>) {
            return goblin_batch_mul(points, scalars);
        } else {
            if (with_edgecases) {
                std::tie(points, scalars) = mask_points(points, scalars);
            }
            const size_t num_points = points.size();
            ASSERT(scalars.size() == num_points);

            batch_lookup_table point_table(points);
            const size_t num_rounds = (max_num_bits == 0) ? Fr::modulus.get_msb() + 1 : max_num_bits;

            std::vector<std::vector<bool_ct>> naf_entries;
            for (size_t i = 0; i < num_points; ++i) {
                naf_entries.emplace_back(compute_naf(scalars[i], max_num_bits));
            }
            const auto offset_generators = compute_offset_generators(num_rounds);
            element accumulator = element::chain_add_end(
                element::chain_add(offset_generators.first, point_table.get_chain_initial_entry()));

            constexpr size_t num_rounds_per_iteration = 4;
            size_t num_iterations = num_rounds / num_rounds_per_iteration;
            num_iterations += ((num_iterations * num_rounds_per_iteration) == num_rounds) ? 0 : 1;
            const size_t num_rounds_per_final_iteration =
                (num_rounds - 1) - ((num_iterations - 1) * num_rounds_per_iteration);
            for (size_t i = 0; i < num_iterations; ++i) {

                std::vector<bool_ct> nafs(num_points);
                std::vector<element::chain_add_accumulator> to_add;
                const size_t inner_num_rounds =
                    (i != num_iterations - 1) ? num_rounds_per_iteration : num_rounds_per_final_iteration;
                for (size_t j = 0; j < inner_num_rounds; ++j) {
                    for (size_t k = 0; k < num_points; ++k) {
                        nafs[k] = (naf_entries[k][i * num_rounds_per_iteration + j + 1]);
                    }
                    to_add.emplace_back(point_table.get_chain_add_accumulator(nafs));
                }
                accumulator = accumulator.multiple_montgomery_ladder(to_add);
            }
            for (size_t i = 0; i < num_points; ++i) {
                element skew = accumulator - points[i];
                Fq out_x = accumulator.x.conditional_select(skew.x, naf_entries[i][num_rounds]);
                Fq out_y = accumulator.y.conditional_select(skew.y, naf_entries[i][num_rounds]);
                accumulator = element(out_x, out_y);
            }
            accumulator = accumulator - offset_generators.second;

            return accumulator;
        }
    }
}

/**
 * Implements scalar multiplication.
 *
 * For multiple scalar multiplication use one of the `batch_mul` methods to save gates.
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::operator*(const Fr& scalar) const
{
    /**
     *
     * Let's say we have some curve E defined over a field Fq. The order of E is p, which is prime.
     *
     * Now lets say we are constructing a SNARK circuit over another curve E2, whose order is r.
     *
     * All of our addition / multiplication / custom gates are going to be evaluating low degree multivariate
     * polynomials modulo r.
     *
     * E.g. our addition/mul gate (for wires a, b, c and selectors q_m, q_l, q_r, q_o, q_c) is:
     *
     *  q_m * a * b + q_l * a + q_r * b + q_o * c + q_c = 0 mod r
     *
     * We want to construct a circuit that evaluates scalar multiplications of curve E. Where q > r and p > r.
     *
     * i.e. we need to perform arithmetic in one prime field, using prime field arithmetic in a completely different
     *prime field.
     *
     * To do *this*, we need to emulate a binary (or in our case quaternary) number system in Fr, so that we can
     * use the binary/quaternary basis to emulate arithmetic in Fq. Which is very messy. See bigfield.hpp for the
     * specifics.
     *
     **/

    if constexpr (IsMegaBuilder<C> && std::same_as<G, bb::g1>) {
        std::vector<element> points{ *this };
        std::vector<Fr> scalars{ scalar };
        return goblin_batch_mul(points, scalars);
    } else {
        constexpr uint64_t num_rounds = Fr::modulus.get_msb() + 1;

        std::vector<bool_ct> naf_entries = compute_naf(scalar);

        const auto offset_generators = compute_offset_generators(num_rounds);

        element accumulator = *this + offset_generators.first;

        for (size_t i = 1; i < num_rounds; ++i) {
            bool_ct predicate = naf_entries[i];
            bigfield y_test = y.conditional_negate(predicate);
            element to_add(x, y_test);
            accumulator = accumulator.montgomery_ladder(to_add);
        }

        element skew_output = accumulator - (*this);

        Fq out_x = accumulator.x.conditional_select(skew_output.x, naf_entries[num_rounds]);
        Fq out_y = accumulator.y.conditional_select(skew_output.y, naf_entries[num_rounds]);

        return element(out_x, out_y) - element(offset_generators.second);
    }
}
} // namespace bb::stdlib
