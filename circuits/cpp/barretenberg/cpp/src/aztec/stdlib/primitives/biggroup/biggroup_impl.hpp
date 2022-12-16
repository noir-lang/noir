#pragma once

#include <numeric/uint256/uint256.hpp>
#include <numeric/uintx/uintx.hpp>
#include <tuple>

#include "../composers/composers.hpp"

#include "../bit_array/bit_array.hpp"
// #include "../field/field.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element()
    : x()
    , y()
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(const typename G::affine_element& input)
    : x(nullptr, input.x)
    , y(nullptr, input.y)
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(const Fq& x_in, const Fq& y_in)
    : x(x_in)
    , y(y_in)
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(const element& other)
    : x(other.x)
    , y(other.y)
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>::element(element&& other)
    : x(other.x)
    , y(other.y)
{}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>& element<C, Fq, Fr, G>::operator=(const element& other)
{
    x = other.x;
    y = other.y;
    return *this;
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G>& element<C, Fq, Fr, G>::operator=(element&& other)
{
    x = other.x;
    y = other.y;
    return *this;
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::operator+(const element& other) const
{
    other.x.assert_is_not_equal(x);
    const Fq lambda = Fq::div_without_denominator_check({ other.y, -y }, (other.x - x));
    const Fq x3 = lambda.sqradd({ -other.x, -x });
    const Fq y3 = lambda.madd(x - x3, { -y });
    return element(x3, y3);
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::operator-(const element& other) const
{
    other.x.assert_is_not_equal(x);
    const Fq lambda = Fq::div_without_denominator_check({ other.y, y }, (other.x - x));
    const Fq x_3 = lambda.sqradd({ -other.x, -x });
    const Fq y_3 = lambda.madd(x_3 - x, { -y });

    return element(x_3, y_3);
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
    return element(x_3, y_3);
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
 * Compute (4 * (*this)) + (2 * add1) + add2
 * If we chain two iterations of the montgomery ladder together, we can squeeze out a non-native field reduction.
 *
 * Total number of field reductions = 9
 *
 * Two calls to mont ladder woud require 10
 *
 * Using doublings and additions would require 12!
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::double_montgomery_ladder(const element& add1, const element& add2) const
{
    add1.x.assert_is_not_equal(x);
    const Fq lambda_1 = Fq::div_without_denominator_check({ add1.y, -y }, (add1.x - x));

    const Fq x_3 = lambda_1.sqradd({ -add1.x, -x });

    const Fq minus_lambda_2 =
        lambda_1 + Fq::div_without_denominator_check({ y + y }, (x_3 - x)); // (y + y) / (x_3 - x);

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    // We can avoid computing y_4, instead substituting the expression `minus_lambda_2 * (x_4 - x) - y` where needed.
    // This is cheaper, because we can evaluate two field multiplications (or a field multiplication + a field division)
    // with only one non-native field reduction.
    // E.g. evaluating (a * b) + (c * d) = e mod p only requires 1 quotient and remainder.
    // Defining the quotient and remainder elements is the major cost of a non-native field multiplication
    // because each requires ~256 bits of range checks
    const Fq x_sub_x4 = x - x_4;

    const Fq x4_sub_add2x = x_4 - add2.x;

    // msub_div; 'compute a multiplication and a division and multiply the two together. Requires only 1 non native
    // field reduction`
    const Fq lambda_3 = Fq::msub_div({ minus_lambda_2 }, { (x_sub_x4) }, (x4_sub_add2x), { y, add2.y });

    // validate we can use incomplete addition formulae
    x_4.assert_is_not_equal(add2.x);

    const Fq x_5 = lambda_3.sqradd({ -x_4, -add2.x });
    const Fq x5_sub_x4 = x_5 - x_4;

    const Fq half_minus_lambda_4_minus_lambda_3 = Fq::msub_div({ minus_lambda_2 }, { x_sub_x4 }, (x5_sub_x4), { y });

    const Fq minus_lambda_4_minus_lambda_3 = half_minus_lambda_4_minus_lambda_3 + half_minus_lambda_4_minus_lambda_3;
    const Fq minus_lambda_4 = minus_lambda_4_minus_lambda_3 + lambda_3;
    const Fq x_6 = minus_lambda_4.sqradd({ -x_4, -x_5 });

    const Fq x6_sub_x4 = x_6 - x_4;

    // y_6 = -L_4 * (x_6 - x_4) - L_2 * (x - x_4) + y
    const Fq y_6 = Fq::dual_madd(minus_lambda_4, (x6_sub_x4), minus_lambda_2, x_sub_x4, { y });

    return element(x_6, y_6);
}

/**
 * If we chain two iterations of the montgomery ladder together, we can squeeze out a non-native field reduction
 *
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::double_montgomery_ladder(const chain_add_accumulator& add1,
                                                                      const element& add2) const
{
    if (add1.is_element) {
        throw_or_abort("An accumulator expected");
    }
    add1.x3_prev.assert_is_not_equal(x);
    Fq lambda_1 = Fq::msub_div(
        { add1.lambda_prev }, { (add1.x1_prev - add1.x3_prev) }, (x - add1.x3_prev), { -add1.y1_prev, -y });

    const Fq x_3 = lambda_1.sqradd({ -add1.x3_prev, -x });

    const Fq minus_lambda_2 =
        lambda_1 + Fq::div_without_denominator_check({ y + y }, (x_3 - x)); // (y + y) / (x_3 - x);

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    // We can avoid computing y_4, instead substituting the expression `minus_lambda_2 * (x_4 - x) - y` where needed.
    // This is cheaper, because we can evaluate two field multiplications (or a field multiplication + a field division)
    // with only one non-native field reduction.
    // E.g. evaluating (a * b) + (c * d) = e mod p only requires 1 quotient and remainder, which is the major cost
    // of a non-native field multiplication
    const Fq x_sub_x4 = x - x_4;

    const Fq x4_sub_add2x = x_4 - add2.x;
    const Fq lambda_3 = Fq::msub_div({ minus_lambda_2 }, { (x_sub_x4) }, (x4_sub_add2x), { y, add2.y });

    x_4.assert_is_not_equal(add2.x);

    const Fq x_5 = lambda_3.sqradd({ -x_4, -add2.x });
    const Fq x5_sub_x4 = x_5 - x_4;

    const Fq half_minus_lambda_4_minus_lambda_3 = Fq::msub_div({ minus_lambda_2 }, { x_sub_x4 }, (x5_sub_x4), { y });

    const Fq minus_lambda_4_minus_lambda_3 = half_minus_lambda_4_minus_lambda_3 + half_minus_lambda_4_minus_lambda_3;
    const Fq minus_lambda_4 = minus_lambda_4_minus_lambda_3 + lambda_3;
    const Fq x_6 = minus_lambda_4.sqradd({ -x_4, -x_5 });

    const Fq x6_sub_x4 = x_6 - x_4;

    const Fq y_6 = Fq::dual_madd(minus_lambda_4, (x6_sub_x4), minus_lambda_2, x_sub_x4, { y });

    return element(x_6, y_6);
}

/**
 * If we chain two iterations of the montgomery ladder together, we can squeeze out a non-native field reduction
 *
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::double_montgomery_ladder(const chain_add_accumulator& add1,
                                                                      const chain_add_accumulator& add2) const
{
    if ((add1.is_element) || (add2.is_element)) {
        throw_or_abort("An accumulator expected");
    }
    add1.x3_prev.assert_is_not_equal(x);
    // add1.y = lambda_prev * (x1_prev - x3_prev) - y1_prev
    Fq lambda_1 = Fq::msub_div(
        { add1.lambda_prev }, { (add1.x1_prev - add1.x3_prev) }, (x - add1.x3_prev), { -add1.y1_prev, -y });

    const Fq x_3 = lambda_1.sqradd({ -add1.x3_prev, -x });

    const Fq minus_lambda_2 =
        lambda_1 + Fq::div_without_denominator_check({ y + y }, (x_3 - x)); // (y + y) / (x_3 - x);

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    // We can avoid computing y_4, instead substituting the expression `minus_lambda_2 * (x_4 - x) - y` where needed.
    // This is cheaper, because we can evaluate two field multiplications (or a field multiplication + a field division)
    // with only one non-native field reduction.
    // E.g. evaluating (a * b) + (c * d) = e mod p only requires 1 quotient and remainder, which is the major cost
    // of a non-native field multiplication
    const Fq x_sub_x4 = x - x_4;

    const Fq x4_sub_add2x = x_4 - add2.x3_prev;

    const Fq lambda_3 = Fq::msub_div({ minus_lambda_2, add2.lambda_prev },
                                     { (x_sub_x4), (add2.x1_prev - add2.x3_prev) },
                                     (x4_sub_add2x),
                                     { y, -add2.y1_prev });

    x_4.assert_is_not_equal(add2.x3_prev);

    const Fq x_5 = lambda_3.sqradd({ -x_4, -add2.x3_prev });
    const Fq x5_sub_x4 = x_5 - x_4;

    const Fq half_minus_lambda_4_minus_lambda_3 = Fq::msub_div({ minus_lambda_2 }, { x_sub_x4 }, (x5_sub_x4), { y });

    const Fq minus_lambda_4_minus_lambda_3 = half_minus_lambda_4_minus_lambda_3 + half_minus_lambda_4_minus_lambda_3;
    const Fq minus_lambda_4 = minus_lambda_4_minus_lambda_3 + lambda_3;
    const Fq x_6 = minus_lambda_4.sqradd({ -x_4, -x_5 });

    const Fq x6_sub_x4 = x_6 - x_4;

    const Fq y_6 = Fq::dual_madd(minus_lambda_4, (x6_sub_x4), minus_lambda_2, x_sub_x4, { y });

    return element(x_6, y_6);
}
/**
 * If we chain two iterations of the montgomery ladder together, we can squeeze out a non-native field reduction
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::double_into_montgomery_ladder(const element& add1) const
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

    add1.x.assert_is_not_equal(x_1);

    const Fq x_minus_x_1 = x - x_1;
    const Fq lambda_1 = Fq::msub_div({ minus_lambda_dbl }, { x_minus_x_1 }, (x_1 - add1.x), { add1.y, y });

    const Fq x_3 = lambda_1.sqradd({ -add1.x, -x_1 });
    const Fq half_minus_lambda_2_minus_lambda_1 =
        Fq::msub_div({ minus_lambda_dbl }, { x_minus_x_1 }, (x_3 - x_1), { y });
    const Fq minus_lambda_2_minus_lambda_1 = half_minus_lambda_2_minus_lambda_1 + half_minus_lambda_2_minus_lambda_1;
    const Fq minus_lambda_2 = minus_lambda_2_minus_lambda_1 + lambda_1;

    const Fq x_4 = minus_lambda_2.sqradd({ -x_1, -x_3 });

    const Fq y_4 = Fq::dual_madd(minus_lambda_2, (x_4 - x_1), minus_lambda_dbl, x_minus_x_1, { y });

    return element(x_4, y_4);
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
    std::array<typename G::affine_element, 1> generator_array = G::template derive_generators<1>();
    typename G::affine_element offset_generator_start(generator_array[0]);

    uint256_t offset_multiplier = uint256_t(1) << uint256_t(num_rounds - 1);

    typename G::affine_element offset_generator_end = typename G::element(offset_generator_start) * offset_multiplier;
    return std::make_pair<element, element>(offset_generator_start, offset_generator_end);
}

/**
 * Generic batch multiplication that works for all elliptic curve types.
 *
 * Implementation is identical to `bn254_endo_batch_mul` but WITHOUT the endomorphism transforms OR support for short
 * scalars See `bn254_endo_batch_mul` for description of algorithm
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::batch_mul(const std::vector<element>& points,
                                                       const std::vector<Fr>& scalars,
                                                       const size_t max_num_bits)
{
    const size_t num_points = points.size();
    ASSERT(scalars.size() == num_points);
    batch_lookup_table point_table(points);
    const size_t num_rounds = (max_num_bits == 0) ? Fr::modulus.get_msb() + 1 : max_num_bits;

    std::vector<std::vector<bool_t<C>>> naf_entries;
    for (size_t i = 0; i < num_points; ++i) {
        naf_entries.emplace_back(compute_naf(scalars[i], max_num_bits));
    }
    const auto offset_generators = compute_offset_generators(num_rounds);
    element accumulator =
        element::chain_add_end(element::chain_add(offset_generators.first, point_table.get_chain_initial_entry()));
    for (size_t i = 1; i < num_rounds / 2; ++i) {
        std::vector<bool_t<C>> nafs;
        for (size_t j = 0; j < num_points; ++j) {
            nafs.emplace_back(naf_entries[j][i * 2 - 1]);
        }
        element::chain_add_accumulator add_1 = point_table.get_chain_add_accumulator(nafs);
        for (size_t j = 0; j < num_points; ++j) {
            nafs[j] = (naf_entries[j][i * 2]);
        }
        element::chain_add_accumulator add_2 = point_table.get_chain_add_accumulator(nafs);

        if (!add_1.is_element) {
            accumulator = accumulator.double_montgomery_ladder(add_1, add_2);
        } else {
            accumulator = accumulator.double_montgomery_ladder(element(add_1.x3_prev, add_1.y3_prev),
                                                               element(add_2.x3_prev, add_2.y3_prev));
        }
    }
    if ((num_rounds & 0x01ULL) == 0x00ULL) {
        std::vector<bool_t<C>> nafs;
        for (size_t j = 0; j < points.size(); ++j) {
            nafs.emplace_back(naf_entries[j][num_rounds - 1]);
        }
        element::chain_add_accumulator add_1 = point_table.get_chain_add_accumulator(nafs);
        if (add_1.is_element) {
            element temp(add_1.x3_prev, add_1.y3_prev);
            accumulator = accumulator.montgomery_ladder(temp);
        } else {
            accumulator = accumulator.montgomery_ladder(add_1);
        }
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
     * All of our addition / multiplication / turbo gates are going to be evaluating low degree multivariate
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
    constexpr uint64_t num_rounds = Fr::modulus.get_msb() + 1;

    std::vector<bool_t<C>> naf_entries = compute_naf(scalar);

    const auto offset_generators = compute_offset_generators(num_rounds);

    element accumulator = *this + offset_generators.first;

    for (size_t i = 1; i < num_rounds; ++i) {
        bool_t<C> predicate = naf_entries[i];
        bigfield y_test = y.conditional_negate(predicate);
        element to_add(x, y_test);
        accumulator = accumulator.montgomery_ladder(to_add);
    }

    element skew_output = accumulator - (*this);

    Fq out_x = accumulator.x.conditional_select(skew_output.x, naf_entries[num_rounds]);
    Fq out_y = accumulator.y.conditional_select(skew_output.y, naf_entries[num_rounds]);

    return element(out_x, out_y) - element(offset_generators.second);
}
} // namespace stdlib
} // namespace plonk