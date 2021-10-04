#pragma once

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
    const Fq lambda = Fq::div({ other.y, -y }, (other.x - x));
    const Fq x3 = lambda.sqradd({ -other.x, -x });
    const Fq y3 = lambda.madd(x - x3, { -y });
    return element(x3, y3);
}

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::operator-(const element& other) const
{
    other.x.assert_is_not_equal(x);
    const Fq lambda = Fq::div({ other.y, y }, (other.x - x));
    const Fq x_3 = lambda.sqradd({ -other.x, -x });
    const Fq y_3 = lambda.madd(x_3 - x, { -y });
    return element(x_3, y_3);
}

template <typename C, class Fq, class Fr, class G> element<C, Fq, Fr, G> element<C, Fq, Fr, G>::dbl() const
{
    Fq T0 = x.sqr();
    Fq T1 = T0 + T0 + T0;
    if constexpr (G::has_a) {
        Fq a(get_context(), uint256_t(G::curve_a));
        T1 = T1 + a;
    }
    Fq lambda = T1 / (y + y);
    Fq x_3 = lambda.sqradd({ -x, -x });
    Fq y_3 = lambda.madd(x - x_3, { -y });
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
    typename Fq::cached_product temp;
    const auto lambda =
        Fq::msub_div({ acc.lambda_prev }, { (x2 - acc.x1_prev) }, (x2 - p1.x), { acc.y1_prev, p1.y }, temp);
    const auto x3 = lambda.sqradd({ -x2, -p1.x });

    chain_add_accumulator output;
    output.x3_prev = x3;
    output.x1_prev = p1.x;
    output.y1_prev = p1.y;
    output.lambda_prev = lambda;

    return output;
}

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
    const Fq lambda = Fq::div({ p2.y, -p1.y }, (p2.x - p1.x));

    const Fq x3 = lambda.sqradd({ -p2.x, -p1.x });
    output.x3_prev = x3;
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
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::montgomery_ladder(const element& other) const
{
    other.x.assert_is_not_equal(x);
    const Fq lambda_1 = Fq::div({ other.y - y }, (other.x - x));

    const Fq x_3 = lambda_1.sqradd({ -other.x, -x });

    const Fq minus_lambda_2 = lambda_1 + (y + y) / (x_3 - x);

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
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::montgomery_ladder(const chain_add_accumulator& to_add)
{
    x.assert_is_not_equal(to_add.x3_prev);

    // lambda = (y2 - y1) / (x2 - x1)
    // but we don't have y2!
    // however, we do know that y2 = lambda_prev * (x1_prev - x2) - y1_prev
    // => lambda * (x2 - x1) = lambda_prev * (x1_prev - x2) - y1_prev - y1
    // => lambda * (x2 - x1) + lambda_prev * (x2 - x1_prev) + y1 + y1_pev = 0
    // => lambda = lambda_prev * (x1_prev - x2) - y1_prev - y1 / (x2 - x1)
    // => lambda = - (lambda_prev * (x2 - x1_prev) + y1_prev + y1) / (x2 - x1)

    auto& x2 = to_add.x3_prev;
    typename Fq::cached_product cache;
    const auto lambda =
        Fq::msub_div({ to_add.lambda_prev }, { (x2 - to_add.x1_prev) }, (x2 - x), { to_add.y1_prev, y }, cache);
    const auto x3 = lambda.sqradd({ -x2, -x });

    const Fq minus_lambda_2 = lambda + Fq::div({ y + y }, (x3 - x));

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
    const Fq lambda_1 = Fq::div({ add1.y, -y }, (add1.x - x));

    const Fq x_3 = lambda_1.sqradd({ -add1.x, -x });

    const Fq minus_lambda_2 = lambda_1 + Fq::div({ y + y }, (x_3 - x)); // (y + y) / (x_3 - x);

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    // We can avoid computing y_4, instead substituting the expression `minus_lambda_2 * (x_4 - x) - y` where needed.
    // This is cheaper, because we can evaluate two field multiplications (or a field multiplication + a field division)
    // with only one non-native field reduction.
    // E.g. evaluating (a * b) + (c * d) = e mod p only requires 1 quotient and remainder.
    // Defining the quotient and remainder elements is the major cost of a non-native field multiplication
    // because each requires ~256 bits of range checks
    const Fq x_sub_x4 = x - x_4;

    const Fq x4_sub_add2x = x_4 - add2.x;
    typename Fq::cached_product minus_lambda_2_mul_x_sub_4_cache;

    // msub_div; 'compute a multiplication and a division and multiply the two together. Requires only 1 non native
    // field reduction`
    const Fq lambda_3 = Fq::msub_div(
        { minus_lambda_2 }, { (x_sub_x4) }, (x4_sub_add2x), { y, add2.y }, minus_lambda_2_mul_x_sub_4_cache);

    // validate we can use incomplete addition formulae
    x_4.assert_is_not_equal(add2.x);

    const Fq x_5 = lambda_3.sqradd({ -x_4, -add2.x });
    const Fq x5_sub_x4 = x_5 - x_4;

    const Fq half_minus_lambda_4_minus_lambda_3 =
        Fq::msub_div({ minus_lambda_2 }, { x_sub_x4 }, (x5_sub_x4), { y }, minus_lambda_2_mul_x_sub_4_cache);

    const Fq minus_lambda_4_minus_lambda_3 = half_minus_lambda_4_minus_lambda_3 + half_minus_lambda_4_minus_lambda_3;
    const Fq minus_lambda_4 = minus_lambda_4_minus_lambda_3 + lambda_3;
    const Fq x_6 = minus_lambda_4.sqradd({ -x_4, -x_5 });

    const Fq x6_sub_x4 = x_6 - x_4;

    // y_6 = -L_4 * (x_6 - x_4) - L_2 * (x - x_4) + y
    const Fq y_6 =
        Fq::dual_madd(minus_lambda_4, (x6_sub_x4), minus_lambda_2, x_sub_x4, { y }, minus_lambda_2_mul_x_sub_4_cache);

    return element(x_6, y_6);
}

/**
 * If we chain two iterations of the montgomery ladder together, we can squeeze out a non-native field reduction
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::double_montgomery_ladder(const chain_add_accumulator& add1,
                                                                      const element& add2) const
{
    add1.x3_prev.assert_is_not_equal(x);
    typename Fq::cached_product cache;
    Fq lambda_1 = Fq::msub_div(
        { add1.lambda_prev }, { (add1.x1_prev - add1.x3_prev) }, (x - add1.x3_prev), { -add1.y1_prev, -y }, cache);

    const Fq x_3 = lambda_1.sqradd({ -add1.x3_prev, -x });

    const Fq minus_lambda_2 = lambda_1 + Fq::div({ y + y }, (x_3 - x)); // (y + y) / (x_3 - x);

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    // We can avoid computing y_4, instead substituting the expression `minus_lambda_2 * (x_4 - x) - y` where needed.
    // This is cheaper, because we can evaluate two field multiplications (or a field multiplication + a field division)
    // with only one non-native field reduction.
    // E.g. evaluating (a * b) + (c * d) = e mod p only requires 1 quotient and remainder, which is the major cost
    // of a non-native field multiplication
    const Fq x_sub_x4 = x - x_4;

    const Fq x4_sub_add2x = x_4 - add2.x;
    typename Fq::cached_product minus_lambda_2_mul_x_sub_4_cache;
    const Fq lambda_3 = Fq::msub_div(
        { minus_lambda_2 }, { (x_sub_x4) }, (x4_sub_add2x), { y, add2.y }, minus_lambda_2_mul_x_sub_4_cache);

    x_4.assert_is_not_equal(add2.x);

    const Fq x_5 = lambda_3.sqradd({ -x_4, -add2.x });
    const Fq x5_sub_x4 = x_5 - x_4;

    const Fq half_minus_lambda_4_minus_lambda_3 =
        Fq::msub_div({ minus_lambda_2 }, { x_sub_x4 }, (x5_sub_x4), { y }, minus_lambda_2_mul_x_sub_4_cache);

    const Fq minus_lambda_4_minus_lambda_3 = half_minus_lambda_4_minus_lambda_3 + half_minus_lambda_4_minus_lambda_3;
    const Fq minus_lambda_4 = minus_lambda_4_minus_lambda_3 + lambda_3;
    const Fq x_6 = minus_lambda_4.sqradd({ -x_4, -x_5 });

    const Fq x6_sub_x4 = x_6 - x_4;

    const Fq y_6 =
        Fq::dual_madd(minus_lambda_4, (x6_sub_x4), minus_lambda_2, x_sub_x4, { y }, minus_lambda_2_mul_x_sub_4_cache);

    return element(x_6, y_6);
}

/**
 * If we chain two iterations of the montgomery ladder together, we can squeeze out a non-native field reduction
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::double_montgomery_ladder(const chain_add_accumulator& add1,
                                                                      const chain_add_accumulator& add2) const
{
    add1.x3_prev.assert_is_not_equal(x);
    // add1.y = lambda_prev * (x1_prev - x3_prev) - y1_prev
    typename Fq::cached_product cache;
    Fq lambda_1 = Fq::msub_div(
        { add1.lambda_prev }, { (add1.x1_prev - add1.x3_prev) }, (x - add1.x3_prev), { -add1.y1_prev, -y }, cache);

    const Fq x_3 = lambda_1.sqradd({ -add1.x3_prev, -x });

    const Fq minus_lambda_2 = lambda_1 + Fq::div({ y + y }, (x_3 - x)); // (y + y) / (x_3 - x);

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    // We can avoid computing y_4, instead substituting the expression `minus_lambda_2 * (x_4 - x) - y` where needed.
    // This is cheaper, because we can evaluate two field multiplications (or a field multiplication + a field division)
    // with only one non-native field reduction.
    // E.g. evaluating (a * b) + (c * d) = e mod p only requires 1 quotient and remainder, which is the major cost
    // of a non-native field multiplication
    const Fq x_sub_x4 = x - x_4;

    const Fq x4_sub_add2x = x_4 - add2.x3_prev;
    typename Fq::cached_product minus_lambda_2_mul_x_sub_4_cache;

    const Fq lambda_3 = Fq::msub_div({ minus_lambda_2, add2.lambda_prev },
                                     { (x_sub_x4), (add2.x1_prev - add2.x3_prev) },
                                     (x4_sub_add2x),
                                     { y, -add2.y1_prev },
                                     minus_lambda_2_mul_x_sub_4_cache);

    x_4.assert_is_not_equal(add2.x3_prev);

    const Fq x_5 = lambda_3.sqradd({ -x_4, -add2.x3_prev });
    const Fq x5_sub_x4 = x_5 - x_4;

    const Fq half_minus_lambda_4_minus_lambda_3 =
        Fq::msub_div({ minus_lambda_2 }, { x_sub_x4 }, (x5_sub_x4), { y }, minus_lambda_2_mul_x_sub_4_cache);

    const Fq minus_lambda_4_minus_lambda_3 = half_minus_lambda_4_minus_lambda_3 + half_minus_lambda_4_minus_lambda_3;
    const Fq minus_lambda_4 = minus_lambda_4_minus_lambda_3 + lambda_3;
    const Fq x_6 = minus_lambda_4.sqradd({ -x_4, -x_5 });

    const Fq x6_sub_x4 = x_6 - x_4;

    const Fq y_6 =
        Fq::dual_madd(minus_lambda_4, (x6_sub_x4), minus_lambda_2, x_sub_x4, { y }, minus_lambda_2_mul_x_sub_4_cache);

    return element(x_6, y_6);
}

/**
 * If we chain two iterations of the montgomery ladder together, we can squeeze out a non-native field reduction
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::double_into_montgomery_ladder(const element& add1) const
{
    const Fq two_x = x + x;
    typename Fq::cached_product temp;
    Fq x_1;
    Fq minus_lambda_dbl;
    if constexpr (G::has_a) {
        Fq a(get_context(), uint256_t(G::curve_a));
        minus_lambda_dbl = Fq::msub_div({ x }, { (two_x + x) }, (y + y), { a }, temp);
        x_1 = minus_lambda_dbl.sqradd({ -(two_x) });
    } else {
        minus_lambda_dbl = Fq::msub_div({ x }, { (two_x + x) }, (y + y), {}, temp);
        x_1 = minus_lambda_dbl.sqradd({ -(two_x) });
    }

    add1.x.assert_is_not_equal(x_1);

    const Fq x_minus_x_1 = x - x_1;
    typename Fq::cached_product cache;
    const Fq lambda_1 = Fq::msub_div({ minus_lambda_dbl }, { x_minus_x_1 }, (x_1 - add1.x), { add1.y, y }, cache);

    const Fq x_3 = lambda_1.sqradd({ -add1.x, -x_1 });
    const Fq half_minus_lambda_2_minus_lambda_1 =
        Fq::msub_div({ minus_lambda_dbl }, { x_minus_x_1 }, (x_3 - x_1), { y }, cache);
    const Fq minus_lambda_2_minus_lambda_1 = half_minus_lambda_2_minus_lambda_1 + half_minus_lambda_2_minus_lambda_1;
    const Fq minus_lambda_2 = minus_lambda_2_minus_lambda_1 + lambda_1;

    const Fq x_4 = minus_lambda_2.sqradd({ -x_1, -x_3 });

    const Fq y_4 = Fq::dual_madd(minus_lambda_2, (x_4 - x_1), minus_lambda_dbl, x_minus_x_1, { y }, cache);

    return element(x_4, y_4);
}

// #################################
// ### SCALAR MULTIPLICATION METHODS
// #################################

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
 * Compute the Non Adjacent Form representation of a scalar multiplier.
 *
 * This method works for both native field elements (e.g. Fr = field_t) AND bigfield elements (e.g. Fr = bigfield)
 **/
template <typename C, class Fq, class Fr, class G>
std::vector<bool_t<C>> element<C, Fq, Fr, G>::compute_naf(const Fr& scalar, const size_t max_num_bits)
{

    static_assert((Fr::modulus.get_msb() + 1) / 2 < barretenberg::fr::modulus.get_msb());
    C* ctx = scalar.context;
    uint512_t scalar_multiplier_512 = uint512_t(uint256_t(scalar.get_value()) % Fr::modulus);
    uint256_t scalar_multiplier = scalar_multiplier_512.lo;

    const size_t num_rounds = (max_num_bits == 0) ? Fr::modulus.get_msb() + 1 : max_num_bits;
    std::vector<bool_t<C>> naf_entries(num_rounds + 1);

    // if boolean is false => do NOT flip y
    // if boolean is true => DO flip y
    // first entry is skew. i.e. do we subtract one from the final result or not
    if (scalar_multiplier.get_bit(0) == false) {
        // add skew
        naf_entries[num_rounds] = bool_t<C>(witness_t(ctx, true));
        scalar_multiplier += uint256_t(1);
    } else {
        naf_entries[num_rounds] = bool_t<C>(witness_t(ctx, false));
    }
    for (size_t i = 0; i < num_rounds - 1; ++i) {
        bool next_entry = scalar_multiplier.get_bit(i + 1);
        // if the next entry is false, we need to flip the sign of the current entry. i.e. make negative
        if (next_entry == false) {
            naf_entries[num_rounds - i - 1] = bool_t<C>(witness_t(ctx, true)); // flip sign
        } else {
            naf_entries[num_rounds - i - 1] = bool_t<C>(witness_t(ctx, false)); // don't flip!
        }
    }
    naf_entries[0] = bool_t<C>(ctx, false); // most significant entry is always true

    // validate correctness of NAF
    if constexpr (!Fr::is_composite) {
        Fr accumulator(ctx, uint256_t(0));
        const size_t num_even_rounds = (num_rounds >> 1) << 1;
        for (size_t i = 0; i < num_even_rounds; i += 2) {
            accumulator = accumulator + accumulator;
            accumulator = accumulator + accumulator;
            Fr hi = Fr(1) - (static_cast<Fr>(naf_entries[i]) * Fr(2));
            Fr lo = Fr(1) - (static_cast<Fr>(naf_entries[i + 1]) * Fr(2));
            accumulator = accumulator.add_two(hi + hi, lo);
        }
        if ((num_rounds & 1UL) == 1UL) {
            accumulator = accumulator + accumulator;

            accumulator = accumulator + Fr(1) - (static_cast<Fr>(naf_entries[num_rounds - 1]) * Fr(2));
        }
        accumulator -= field_t<C>(naf_entries[num_rounds]);
        accumulator.assert_equal(scalar);
    } else {
        const auto reconstruct_half_naf = [](bool_t<C>* nafs, const size_t half_round_length) {
            // Q: need constraint to start from zero?
            field_t<C> negative_accumulator(0);
            field_t<C> positive_accumulator(0);
            for (size_t i = 0; i < half_round_length; ++i) {
                negative_accumulator = negative_accumulator + negative_accumulator + field_t<C>(nafs[i]);
                positive_accumulator =
                    positive_accumulator + positive_accumulator + field_t<C>(1) - field_t<C>(nafs[i]);
            }
            return std::make_pair(positive_accumulator, negative_accumulator);
        };
        const size_t midpoint = num_rounds - Fr::NUM_LIMB_BITS * 2;
        auto hi_accumulators = reconstruct_half_naf(&naf_entries[0], midpoint);
        auto lo_accumulators = reconstruct_half_naf(&naf_entries[midpoint], num_rounds - midpoint);

        lo_accumulators.second = lo_accumulators.second + field_t<C>(naf_entries[num_rounds]);

        Fr reconstructed_positive = Fr(lo_accumulators.first, hi_accumulators.first);
        Fr reconstructed_negative = Fr(lo_accumulators.second, hi_accumulators.second);
        Fr accumulator = reconstructed_positive - reconstructed_negative;
        accumulator.assert_equal(scalar);
    }
    return naf_entries;
}

/**
 * A batch multiplication method for the BN254 curve. This method is only available if Fr == field_t<barretenberg::fr>
 *
 * big_points : group elements we will multiply by full 254-bit scalar multipliers
 * big_scalars : 254-bit scalar multipliers. We want to compute (\sum big_scalars[i] * big_points[i])
 * small_points : group elements we will multiply by short scalar mutipliers whose max value will be (1 <<
 *max_num_small_bits) small_scalars : short scalar mutipliers whose max value will be (1 << max_num_small_bits)
 * max_num_small_bits : MINIMUM value must be 128 bits
 * (we will be splitting `big_scalars` into two 128-bit scalars, we assume all scalars after this transformation are 128
 *bits)
 **/
template <typename C, class Fq, class Fr, class G>
template <typename, typename>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::bn254_endo_batch_mul(const std::vector<element>& big_points,
                                                                  const std::vector<Fr>& big_scalars,
                                                                  const std::vector<element>& small_points,
                                                                  const std::vector<Fr>& small_scalars,
                                                                  const size_t max_num_small_bits)
{
    ASSERT(max_num_small_bits >= 128);
    const size_t num_big_points = big_points.size();
    const size_t num_small_points = small_points.size();
    C* ctx = nullptr;
    for (auto element : big_points) {
        if (element.get_context()) {
            ctx = element.get_context();
            break;
        }
    }

    std::vector<element> points;
    std::vector<Fr> scalars;
    std::vector<element> endo_points;
    std::vector<Fr> endo_scalars;

    /**
     * Split big scalars into short 128-bit scalars.
     *
     * For `big_scalars` we use the BN254 curve endomorphism to split the scalar into two short 128-bit scalars.
     * i.e. for scalar multiplier `k` we derive 128-bit values `k1, k2` where:
     *   k = k1 - k2 * \beta
     * (\beta is the cube root of unity modulo the group order of the BN254 curve)
     *
     * This ensures ALL our scalar multipliers can now be treated as 128-bit scalars,
     * which halves the number of iterations of our main "double and add" loop!
     */
    for (size_t i = 0; i < num_big_points; ++i) {
        Fr scalar = big_scalars[i];
        // Q: is it a problem if wraps? get_value is 512 bits
        // A: it can't wrap, this method only compiles if the Fr type is a field_t<barretenberg::fr> type

        // Split k into short scalars (scalar_k1, scalar_k2) using bn254 endomorphism.
        barretenberg::fr k = uint256_t(scalar.get_value());
        barretenberg::fr k1(0);
        barretenberg::fr k2(0);
        barretenberg::fr::split_into_endomorphism_scalars(k.from_montgomery_form(), k1, k2);
        Fr scalar_k1 = witness_t<C>(ctx, k1.to_montgomery_form());
        Fr scalar_k2 = witness_t<C>(ctx, k2.to_montgomery_form());

        // Add copy constraint that validates k1 = scalar_k1 - scalar_k2 * \beta
        scalar.assert_equal(scalar_k1 - scalar_k2 * barretenberg::fr::beta());
        scalars.push_back(scalar_k1);
        endo_scalars.push_back(scalar_k2);
        element point = big_points[i];
        points.push_back(point);

        // negate the point that maps to the endo scalar `scalar_k2`
        // instead of computing scalar_k1 * [P] - scalar_k2 * [P], we compute scalar_k1 * [P] + scalar_k2 * [-P]
        point.y = -point.y;
        point.x = point.x * Fq(ctx, uint256_t(barretenberg::fq::beta()));
        point.y.self_reduce();
        endo_points.push_back(point);
    }
    for (size_t i = 0; i < num_small_points; ++i) {
        points.push_back(small_points[i]);
        scalars.push_back(small_scalars[i]);
    }
    std::copy(endo_points.begin(), endo_points.end(), std::back_inserter(points));
    std::copy(endo_scalars.begin(), endo_scalars.end(), std::back_inserter(scalars));

    ASSERT(big_scalars.size() == num_big_points);
    ASSERT(small_scalars.size() == num_small_points);

    /**
     * Compute batch_lookup_table
     *
     * batch_lookup_table implements a lookup table for a vector of points.
     *
     * For TurboPlonk, we subdivide `batch_lookup_table` into a set of 3-bit lookup tables,
     * (using 2-bit and 1-bit tables if points.size() is not a multiple of 8)
     *
     * We index the lookup table using a vector of NAF values for each point
     *
     * e.g. for points P_1, .., P_N and naf values s_1, ..., s_n (where S_i = +1 or -1),
     * the lookup table will compute:
     *
     *  \sum_{i=0}^n (s_i ? -P_i : P_i)
     **/
    batch_lookup_table point_table(points);

    /**
     * Compute scalar multiplier NAFs
     *
     * A Non Adjacent Form is a representation of an integer where each 'bit' is either +1 OR -1, i.e. each bit entry is
     *non-zero. This is VERY useful for biggroup operations, as this removes the need to conditionally add points
     *depending on whether the scalar mul bit is +1 or 0 (instead we multiply the y-coordinate by the NAF value, which
     *is cheaper)
     *
     * The vector `naf_entries` tracks the `naf` set for each point, where each `naf` set is a vector of bools
     * if `naf[i][j] = 0` this represents a NAF value of -1
     * if `naf[i][j] = 1` this represents a NAF value of +1
     **/
    const size_t num_rounds = max_num_small_bits;
    const size_t num_points = points.size();
    std::vector<std::vector<bool_t<C>>> naf_entries;
    for (size_t i = 0; i < num_points; ++i) {
        naf_entries.emplace_back(compute_naf(scalars[i], max_num_small_bits));
    }

    /**
     * Initialize accumulator point with an offset generator. See `compute_offset_generators` for detailed explanation
     **/
    const auto offset_generators = compute_offset_generators(num_rounds);
    element accumulator = offset_generators.first;

    /**
     * Get the initial entry of our point table. This is the same as point_table.get_accumulator for the most
     *significant NAF entry. HOWEVER, we know the most significant NAF value is +1 because our scalar muls are positive.
     * `get_initial_entry` handles this special case as it's cheaper than `point_table.get_accumulator`
     **/
    accumulator = accumulator + point_table.get_initial_entry();

    /**
     * Main "double and add" loop
     *
     * Each loop iteration traverses TWO bits of our scalar multiplier. Algorithm performs following:
     *
     * 1. Extract NAF value for bit `2*i - 1` for each scalar multiplier and store in `nafs` vector.
     * 2. Use `nafs` vector to derive the point that we need (`add_1`) to add into our accumulator.
     * 3. Repeat the above 2 steps but for bit `2 * i` (`add_2`)
     * 4. Compute `accumulator = 4 * accumulator + 2 * add_1 + add_2` using `double_montgomery_ladder` method
     *
     * The purpose of the above is to minimize the number of required range checks (vs a simple double and add algo).
     *
     * When computing two iterations of the montgomery ladder algorithm, we can neglect computing the y-coordinate of
     *the 1st ladder output. See `double_montgomery_ladder` for more details.
     **/
    for (size_t i = 1; i < num_rounds / 2; ++i) {
        // `nafs` tracks the naf value for each point for the current round
        std::vector<bool_t<C>> nafs;
        for (size_t j = 0; j < points.size(); ++j) {
            nafs.emplace_back(naf_entries[j][i * 2 - 1]);
        }

        /**
         * Get `chain_add_accumulator`.
         *
         * Recovering a point from our point table requires group additions iff the table is >3 bits.
         * We can chain repeated add operations together without computing the y-coordinate of intermediate addition
         *outputs.
         *
         * This is represented using the `chain_add_accumulator` type. See the type declaration for more details
         *
         * (this is cheaper than regular additions iff point_table.get_accumulator require 2 or more point additions.
         *  Cost is the same as `point_table.get_accumulator` if 1 or 0 point additions are required)
         **/
        element::chain_add_accumulator add_1 = point_table.get_chain_add_accumulator(nafs);
        for (size_t j = 0; j < points.size(); ++j) {
            nafs[j] = (naf_entries[j][i * 2]);
        }
        element::chain_add_accumulator add_2 = point_table.get_chain_add_accumulator(nafs);

        // Perform the double montgomery ladder. We need to convert our chain_add_accumulator types into regular
        // elements if the accumuator does not contain a y-coordinate
        if (!add_1.is_element) {
            accumulator = accumulator.double_montgomery_ladder(add_1, add_2);
        } else {
            accumulator = accumulator.double_montgomery_ladder(element(add_1.x3_prev, add_1.y3_prev),
                                                               element(add_2.x3_prev, add_2.y3_prev));
        }
    }

    // we need to iterate 1 more time if the number of rounds is even
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

    /**
     * Handle skew factors.
     *
     * We represent scalar multipliers via Non Adjacent Form values (NAF).
     * In a NAF, each bit value is either -1 or +1.
     * We use this representation to avoid having to conditionally add points
     * (i.e. every bit we iterate over will result in either a point addition or subtraction,
     *  instead of conditionally adding a point into an accumulator,
     *  we conditionally negate the point's y-coordinate and *always* add it into the accumulator)
     *
     * However! The problem here is that we can only represent odd integers with a NAF.
     * For even integers we add +1 to the integer and set that multiplier's `skew` value to `true`.
     *
     * We record a scalar multiplier's skew value at the end of their NAF values
     *(`naf_entries[point_index][num_rounds]`)
     *
     * If the skew is true, we must subtract the original point from the accumulator.
     **/
    for (size_t i = 0; i < num_points; ++i) {
        element skew = accumulator - points[i];
        Fq out_x = accumulator.x.conditional_select(skew.x, naf_entries[i][num_rounds]);
        Fq out_y = accumulator.y.conditional_select(skew.y, naf_entries[i][num_rounds]);
        accumulator = element(out_x, out_y);
    }

    // Remove the offset generator point!
    accumulator = accumulator - offset_generators.second;

    // Return our scalar mul output
    return accumulator;
}

/**
 * Generic batch multiplication that works for all elliptic curve types.
 *
 * Implementation is identical to `bn254_endo_batch_mul` but WITHOUT the endomorphism transforms OR support for short
 *scalars See `bn254_endo_batch_mul` for description of algorithm
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
 * Generic batch multiplication that works for all elliptic curve types.
 *
 * Implementation is identical to `bn254_endo_batch_mul` but WITHOUT the endomorphism transforms.
 * See `bn254_endo_batch_mul` for description of algorithm
 **/
template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::mixed_batch_mul(const std::vector<element>& big_points,
                                                             const std::vector<Fr>& big_scalars,
                                                             const std::vector<element>& small_points,
                                                             const std::vector<Fr>& small_scalars,
                                                             const size_t max_num_small_bits)
{
    if constexpr (!G::USE_ENDOMORPHISM || Fr::is_composite) {
        std::vector<element> points(big_points.begin(), big_points.end());
        std::copy(small_points.begin(), small_points.end(), std::back_inserter(big_points));
        std::vector<element> scalars(big_scalars.begin(), big_scalars.end());
        std::copy(small_scalars.begin(), small_scalars.end(), std::back_inserter(big_scalars));
        return batch_mul(points, scalars);
    }
    const size_t num_big_points = big_points.size();
    const size_t num_small_points = small_points.size();
    C* ctx = nullptr;
    for (auto element : big_points) {
        if (element.get_context()) {
            ctx = element.get_context();
            break;
        }
    }

    std::vector<element> points;
    std::vector<Fr> scalars;
    std::vector<element> endo_points;
    std::vector<Fr> endo_scalars;
    for (size_t i = 0; i < num_big_points; ++i) {
        Fr scalar = big_scalars[i];
        // Q:is it a problem if wraps? get_value is 512 bits
        barretenberg::fr k = scalar.get_value();
        barretenberg::fr k1(0);
        barretenberg::fr k2(0);
        barretenberg::fr::split_into_endomorphism_scalars(k.from_montgomery_form(), k1, k2);
        Fr scalar_k1 = witness_t<C>(
            ctx, k1.to_montgomery_form()); // Q:seems we are assuming barret fr and template Fr are same field
        Fr scalar_k2 = witness_t<C>(ctx, k2.to_montgomery_form());
        scalar.assert_equal(scalar_k1 - scalar_k2 * barretenberg::fr::beta(),
                            "biggroup endormorphism scalar split fail?");
        scalars.push_back(scalar_k1);
        endo_scalars.push_back(scalar_k2);
        element point = big_points[i];
        points.push_back(point);
        point.y = -point.y;
        point.x = point.x * Fq(ctx, uint256_t(barretenberg::fq::beta()));
        point.y.self_reduce();
        endo_points.push_back(point);
    }
    for (size_t i = 0; i < num_small_points; ++i) {
        points.push_back(small_points[i]);
        scalars.push_back(small_scalars[i]);
    }
    std::copy(endo_points.begin(), endo_points.end(), std::back_inserter(points));
    std::copy(endo_scalars.begin(), endo_scalars.end(), std::back_inserter(scalars));

    ASSERT(big_scalars.size() == num_big_points);
    ASSERT(small_scalars.size() == num_small_points);

    batch_lookup_table point_table(points);

    const size_t num_rounds = max_num_small_bits;
    const size_t num_points = points.size();
    std::vector<std::vector<bool_t<C>>> naf_entries;
    for (size_t i = 0; i < num_points; ++i) {
        naf_entries.emplace_back(compute_naf(scalars[i], max_num_small_bits));
    }
    const auto offset_generators = compute_offset_generators(num_rounds);

    element accumulator = offset_generators.first + point_table.get_initial_entry();

    for (size_t i = 1; i < num_rounds; ++i) {
        std::vector<bool_t<C>> nafs;
        for (size_t j = 0; j < points.size(); ++j) {
            nafs.emplace_back(naf_entries[j][i]);
        }

        element to_add = point_table.get(nafs);

        accumulator = accumulator.montgomery_ladder(to_add);
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