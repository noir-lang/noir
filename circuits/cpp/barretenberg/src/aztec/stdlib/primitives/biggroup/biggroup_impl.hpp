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

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::montgomery_ladder(const element& other) const
{
    other.x.assert_is_not_equal(x);
    const Fq lambda_1 = Fq::div({ other.y - y }, (other.x - x));

    const Fq x_3 = lambda_1.sqradd({ -other.x, -x });

    /**
     * Compute D = A + B + A, where A = `this` and B = `other`
     *
     * We can skip computing the y-coordinate of C = A + B:
     *
     * To compute D = C + A, we need the gradient of our two coordinates, specifically:
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

    const Fq minus_lambda_2 = lambda_1 + (y + y) / (x_3 - x);

    const Fq x_4 = minus_lambda_2.sqradd({ -x, -x_3 });

    const Fq y_4 = minus_lambda_2.madd(x_4 - x, { -y });
    return element(x_4, y_4);
}

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

template <typename C, class Fq, class Fr, class G>
std::vector<bool_t<C>> element<C, Fq, Fr, G>::compute_naf(const Fr& scalar, const size_t max_num_bits)
{
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

template <typename C, class Fq, class Fr, class G>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::batch_mul(const std::vector<element>& points,
                                                       const std::vector<Fr>& scalars,
                                                       const size_t max_num_bits)
{
    const size_t num_points = points.size();
    ASSERT(scalars.size() == num_points);

    batch_lookup_table point_table(points);

    const size_t num_rounds = (max_num_bits == 0) ? Fq::modulus.get_msb() + 1 : max_num_bits;

    std::vector<std::vector<bool_t<C>>> naf_entries;
    for (size_t i = 0; i < num_points; ++i) {
        naf_entries.emplace_back(compute_naf(scalars[i], max_num_bits));
    }

    const auto offset_generators = compute_offset_generators(num_rounds);

    element accumulator = offset_generators.first + point_table.get_initial_entry();

    for (size_t i = 1; i < num_rounds; ++i) {
        std::vector<bool_t<C>> nafs;
        for (size_t j = 0; j < num_points; ++j) {
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
    C* ctx;
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
        barretenberg::fr k = scalar.get_value();
        barretenberg::fr k1(0);
        barretenberg::fr k2(0);
        barretenberg::fr::split_into_endomorphism_scalars(k.from_montgomery_form(), k1, k2);
        Fr scalar_k1 = witness_t<C>(ctx, k1.to_montgomery_form());
        Fr scalar_k2 = witness_t<C>(ctx, k2.to_montgomery_form());
        ctx->assert_equal((scalar_k1 - scalar_k2 * barretenberg::fr::beta()).witness_index,
                          scalar.normalize().witness_index);
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
    constexpr uint64_t num_rounds = Fq::modulus_u512.get_msb() + 1;

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