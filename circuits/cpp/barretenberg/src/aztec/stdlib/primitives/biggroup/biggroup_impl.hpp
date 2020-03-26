#pragma once

namespace plonk {
namespace stdlib {

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T>::element()
    : x()
    , y()
{}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T>::element(const Fq& x_in, const Fq& y_in)
    : x(x_in)
    , y(y_in)
{}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T>::element(const element& other)
    : x(other.x)
    , y(other.y)
{}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T>::element(element&& other)
    : x(other.x)
    , y(other.y)
{}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T>& element<C, Fq, Fr, T>::operator=(const element& other)
{
    x = other.x;
    y = other.y;
    return *this;
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T>& element<C, Fq, Fr, T>::operator=(element&& other)
{
    x = other.x;
    y = other.y;
    return *this;
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::operator+(const element& other) const
{
    uint512_t diff = other.x.get_value() - x.get_value();
    if (diff == uint512_t(0) || (diff % Fq::modulus_u512 == uint512_t(0))) {
        std::cout << "exception condition hit?" << std::endl;
    }
    const Fq lambda = (other.y - y) / (other.x - x);
    const Fq x3 = lambda.sqr() - (other.x + x);
    const Fq y3 = lambda * (x - x3) - y;
    return element(x3, y3);
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::operator-(const element& other) const
{
    const Fq lambda = (other.y + y) / (other.x - x);
    const Fq x_3 = lambda.sqr() - (other.x + x);
    const Fq y_3 = lambda * (x_3 - x) - y;
    return element(x_3, y_3);
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::montgomery_ladder(const element& other) const
{
    const Fq lambda_1 = (other.y - y) / (other.x - x);

    const Fq x_3 = lambda_1.sqr() - (other.x + x);

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

    const Fq minus_lambda_2 = lambda_1 + ((y + y) / (x_3 - x));

    const Fq x_4 = minus_lambda_2.sqr() - (x + x_3);

    const Fq y_4 = minus_lambda_2 * (x_4 - x) - y;
    return element(x_4, y_4);
}

template <typename C, class Fq, class Fr, class T> element<C, Fq, Fr, T> element<C, Fq, Fr, T>::dbl() const
{
    Fq T0 = x.sqr();
    Fq T1 = T0 + T0 + T0;
    if constexpr (T::has_a) {
        Fq a(get_context(), uint256_t(T::a));
        T1 = T1 + a;
    }
    Fq lambda = T1 / (y + y);
    Fq x_3 = lambda.sqr() - (x + x);
    Fq y_3 = lambda * (x - x_3);
    y_3 = y_3 - y;
    return element(x_3, y_3);
}

template <typename C, class Fq, class Fr, class T>
std::vector<bool_t<C>> element<C, Fq, Fr, T>::compute_naf(const Fr& scalar)
{
    C* ctx = scalar.context;
    uint512_t scalar_multiplier_512 = uint512_t(uint256_t(scalar.get_value()) % Fr::modulus);
    scalar_multiplier_512 += uint512_t(Fr::modulus);

    constexpr uint64_t default_offset_bits = Fr::modulus.get_msb() + 2; // 2^254
    constexpr uint512_t default_offset = uint512_t(1) << default_offset_bits;

    while (scalar_multiplier_512 < default_offset) {
        scalar_multiplier_512 += uint512_t(Fr::modulus);
    }
    scalar_multiplier_512 -= default_offset;
    uint256_t scalar_multiplier = scalar_multiplier_512.lo;

    constexpr uint64_t num_rounds = Fr::modulus.get_msb() + 1;
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
    naf_entries[0] = bool_t<C>(witness_t(ctx, false)); // most significant entry is always true

    // validate correctness of NAF
    Fr accumulator(ctx, uint256_t(2));
    for (size_t i = 0; i < num_rounds; ++i) {
        accumulator = accumulator + accumulator;

        Fr to_add(ctx, uint256_t(1));
        accumulator = accumulator + to_add.conditional_negate(naf_entries[i]);
    }

    if constexpr (Fr::is_composite) {
        Fr skew(ctx, uint256_t(0));
        skew.binary_basis_limbs[0].element = field_t<C>(naf_entries[num_rounds]);
        skew.prime_basis_limb = field_t<C>(naf_entries[num_rounds]);
        accumulator = accumulator - skew;
        // accumulator.assert_is_in_field();
    } else {
        accumulator -= field_t<C>(naf_entries[num_rounds]);
    }
    std::cout << "naf check start" << std::endl;
    std::cout << "scalar = " << scalar << std::endl;
    std::cout << "recons = " << accumulator << std::endl;
    accumulator.assert_equal(scalar);
    std::cout << "naf check end" << std::endl;
    return naf_entries;
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::twin_mul(const element& base_a,
                                                      const Fr& scalar_a,
                                                      const element& base_b,
                                                      const Fr& scalar_b)
{

    constexpr uint64_t num_rounds = Fq::modulus_u512.get_msb() + 1;

    std::vector<bool_t<C>> naf_entries_a = compute_naf(scalar_a);
    std::vector<bool_t<C>> naf_entries_b = compute_naf(scalar_b);

    // compute precomputed lookup table of:
    // B + A
    // B - A
    // -B + A
    // -B - A
    twin_lookup_table table({ base_a, base_b });
    // element T0 = base_b + base_a;
    // element T1 = base_b - base_a;

    element accumulator = table[0].dbl(); // (base_a.montgomery_ladder(base_b)) + base_b;
    // bool_t<C> initial_selector = naf_entries_a[1] ^ naf_entries_b[1];
    // Fq initial_x = T0.x.conditional_select(T1.x, initial_selector);
    // Fq initial_y = T0.y.conditional_select(T1.y, initial_selector);
    // accumulator = accumulator + element(initial_x, initial_y.conditional_negate(naf_entries_b[1]));

    for (size_t i = 0; i < num_rounds; ++i) {
        element to_add = table.get(naf_entries_a[i], naf_entries_b[i]);
        accumulator = accumulator.montgomery_ladder(to_add);
    }

    element skew_output_a = accumulator - base_a;

    Fq out_x = accumulator.x.conditional_select(skew_output_a.x, naf_entries_a[num_rounds]);
    Fq out_y = accumulator.y.conditional_select(skew_output_a.y, naf_entries_a[num_rounds]);

    accumulator = element(out_x, out_y);

    element skew_output_b = accumulator - base_b;

    out_x = accumulator.x.conditional_select(skew_output_b.x, naf_entries_b[num_rounds]);
    out_y = accumulator.y.conditional_select(skew_output_b.y, naf_entries_b[num_rounds]);

    out_x.self_reduce();
    out_y.self_reduce();
    return element(out_x, out_y);
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::quad_mul(const element& base_a,
                                                      const Fr& scalar_a,
                                                      const element& base_b,
                                                      const Fr& scalar_b,
                                                      const element& base_c,
                                                      const Fr& scalar_c,
                                                      const element& base_d,
                                                      const Fr& scalar_d)

{
    constexpr uint64_t num_rounds = Fq::modulus_u512.get_msb() + 1;

    std::vector<bool_t<C>> naf_entries_a = compute_naf(scalar_a);
    std::vector<bool_t<C>> naf_entries_b = compute_naf(scalar_b);
    std::vector<bool_t<C>> naf_entries_c = compute_naf(scalar_c);
    std::vector<bool_t<C>> naf_entries_d = compute_naf(scalar_d);

    quad_lookup_table element_table({ base_a, base_b, base_c, base_d });

    element accumulator = element_table[0].dbl();

    for (size_t i = 0; i < num_rounds; ++i) {
        element to_add = element_table.get(naf_entries_a[i], naf_entries_b[i], naf_entries_c[i], naf_entries_d[i]);
        accumulator = accumulator.montgomery_ladder(to_add);
    }

    element skew_output_a = accumulator - base_a;

    Fq out_x = accumulator.x.conditional_select(skew_output_a.x, naf_entries_a[num_rounds]);
    Fq out_y = accumulator.y.conditional_select(skew_output_a.y, naf_entries_a[num_rounds]);

    accumulator = element(out_x, out_y);

    element skew_output_b = accumulator - base_b;

    out_x = accumulator.x.conditional_select(skew_output_b.x, naf_entries_b[num_rounds]);
    out_y = accumulator.y.conditional_select(skew_output_b.y, naf_entries_b[num_rounds]);

    accumulator = element(out_x, out_y);

    element skew_output_c = accumulator - base_c;

    out_x = accumulator.x.conditional_select(skew_output_c.x, naf_entries_c[num_rounds]);
    out_y = accumulator.y.conditional_select(skew_output_c.y, naf_entries_c[num_rounds]);

    accumulator = element(out_x, out_y);

    element skew_output_d = accumulator - base_d;

    out_x = accumulator.x.conditional_select(skew_output_d.x, naf_entries_d[num_rounds]);
    out_y = accumulator.y.conditional_select(skew_output_d.y, naf_entries_d[num_rounds]);

    accumulator = element(out_x, out_y);
    accumulator.x.self_reduce();
    accumulator.y.self_reduce();
    return accumulator;
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::batch_mul(const std::vector<element>& points,
                                                       const std::vector<Fr>& scalars)
{
    const size_t num_points = points.size();
    ASSERT(scalars.size() == num_points);

    const size_t num_quads = num_points / 4;

    const bool has_twin = ((num_quads * 4) < num_points - 1);

    const bool has_singleton = num_points != ((num_quads * 4) + ((size_t)has_twin * 2));

    std::cout << "num points = " << num_points << std::endl;
    std::cout << "num quads = " << num_quads << std::endl;
    std::cout << "has twin = " << has_twin << std::endl;
    std::cout << "has singleton" << has_singleton << std::endl;
    std::vector<quad_lookup_table> quad_tables;
    std::vector<twin_lookup_table> twin_tables;
    std::vector<element> singletons;
    for (size_t i = 0; i < num_quads; ++i) {
        quad_tables.emplace_back(
            quad_lookup_table({ points[4 * i], points[4 * i + 1], points[4 * i + 2], points[4 * i + 3] }));
    }

    if (has_twin) {
        twin_tables.emplace_back(twin_lookup_table({ points[4 * num_quads], points[4 * num_quads + 1] }));
    }

    if (has_singleton) {
        singletons.emplace_back(points[points.size() - 1]);
        singletons[0].x.self_reduce();
        singletons[0].y.self_reduce();
    }

    constexpr uint64_t num_rounds = Fq::modulus.get_msb() + 1;

    std::vector<std::vector<bool_t<C>>> naf_entries;
    for (size_t i = 0; i < num_points; ++i) {
        naf_entries.emplace_back(compute_naf(scalars[i]));
    }

    std::vector<element> add_accumulator;
    for (size_t i = 0; i < num_quads; ++i) {
        add_accumulator.emplace_back(quad_tables[i][0]);
    }
    if (has_twin) {
        add_accumulator.emplace_back(twin_tables[0][0]);
    }
    if (has_singleton) {
        add_accumulator.emplace_back(singletons[0]);
    }

    element accumulator = add_accumulator[0];
    for (size_t i = 1; i < add_accumulator.size(); ++i) {
        accumulator = accumulator + add_accumulator[i];
    }
    accumulator = accumulator.dbl();

    for (size_t i = 0; i < num_rounds; ++i) {
        std::cout << "i = " << i << std::endl;
        std::vector<element> round_accumulator;
        for (size_t j = 0; j < num_quads; ++j) {
            round_accumulator.push_back(quad_tables[j].get(naf_entries[4 * j][i],
                                                           naf_entries[4 * j + 1][i],
                                                           naf_entries[4 * j + 2][i],
                                                           naf_entries[4 * j + 3][i]));
        }
        if (has_twin) {
            round_accumulator.push_back(
                twin_tables[0].get(naf_entries[num_quads * 4][i], naf_entries[num_quads * 4 + 1][i]));
        }
        if (has_singleton) {
            round_accumulator.push_back(singletons[0].conditional_negate(naf_entries[num_points - 1][i]));
        }

        std::cout << "naf entries 1 = " << naf_entries[0][i].get_value() << naf_entries[1][i].get_value()
                  << naf_entries[2][i].get_value() << naf_entries[3][i].get_value() << std::endl;
        std::cout << "naf entries 2 = " << naf_entries[4][i].get_value() << naf_entries[5][i].get_value()
                  << naf_entries[6][i].get_value() << naf_entries[7][i].get_value() << std::endl;

        element to_add = round_accumulator[0];
        for (size_t j = 1; j < round_accumulator.size(); ++j) {
            std::cout << "adding at index " << j << std::endl;
            std::cout << "x1 y1 = " << to_add.x.get_value() << " , " << to_add.y.get_value() << std::endl;
            std::cout << "x2 y2 = " << round_accumulator[j].x.get_value() << " , " << round_accumulator[j].y.get_value()
                      << std::endl;
            to_add = to_add + round_accumulator[j];
        }
        std::cout << "calling mont ladder" << std::endl;
        accumulator = accumulator.montgomery_ladder(to_add);
    }

    for (size_t i = 0; i < num_points; ++i) {
        element skew = accumulator - points[i];
        Fq out_x = accumulator.x.conditional_select(skew.x, naf_entries[i][num_rounds]);
        Fq out_y = accumulator.y.conditional_select(skew.y, naf_entries[i][num_rounds]);
        accumulator = element(out_x, out_y);
    }
    accumulator.x.self_reduce();
    accumulator.y.self_reduce();
    return accumulator;
}

template <typename C, class Fq, class Fr, class T>
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::operator*(const Fr& scalar) const
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

    uint512_t reconstructed_positive(0);
    uint512_t reconstructed_negative(0);
    for (size_t i = 0; i < num_rounds; ++i) {
        reconstructed_positive += reconstructed_positive;
        reconstructed_negative += reconstructed_negative;

        if (naf_entries[i].get_value()) {
            reconstructed_negative += uint512_t(1);
        } else {
            reconstructed_positive += uint512_t(1);
        }
    }
    uint512_t reconstructed = reconstructed_positive - reconstructed_negative;
    if (naf_entries[num_rounds].get_value()) {
        reconstructed -= uint512_t(1);
    }

    element accumulator = (*this).dbl();

    for (size_t i = 0; i < num_rounds; ++i) {
        bool_t<C> predicate = naf_entries[i];
        bigfield y_test = y.conditional_negate(predicate);
        element to_add(x, y_test);
        accumulator = accumulator.montgomery_ladder(to_add);
    }

    element skew_output = accumulator - (*this);

    Fq out_x = accumulator.x.conditional_select(skew_output.x, naf_entries[num_rounds]);
    Fq out_y = accumulator.y.conditional_select(skew_output.y, naf_entries[num_rounds]);

    out_x.self_reduce();
    out_y.self_reduce();
    return element(out_x, out_y);
}

} // namespace stdlib
} // namespace plonk