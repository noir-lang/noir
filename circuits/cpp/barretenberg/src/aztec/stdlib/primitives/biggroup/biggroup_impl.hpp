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
    uint512_t scalar_multiplier_512 = scalar.get_value() % Fr::modulus_u512;
    scalar_multiplier_512 += Fr::modulus_u512;

    constexpr uint64_t default_offset_bits = Fr::modulus_u512.get_msb() + 2; // 2^254
    constexpr uint512_t default_offset = uint512_t(1) << default_offset_bits;

    while (scalar_multiplier_512 < default_offset) {
        scalar_multiplier_512 += Fr::modulus_u512;
    }
    scalar_multiplier_512 -= default_offset;
    uint256_t scalar_multiplier = scalar_multiplier_512.lo;

    constexpr uint64_t num_rounds = Fr::modulus_u512.get_msb() + 1;
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

    element T0 = base_b + base_a;
    element T1 = base_b - base_a;

    element accumulator = T0.dbl(); // (base_a.montgomery_ladder(base_b)) + base_b;
    // bool_t<C> initial_selector = naf_entries_a[1] ^ naf_entries_b[1];
    // Fq initial_x = T0.x.conditional_select(T1.x, initial_selector);
    // Fq initial_y = T0.y.conditional_select(T1.y, initial_selector);
    // accumulator = accumulator + element(initial_x, initial_y.conditional_negate(naf_entries_b[1]));

    for (size_t i = 0; i < num_rounds; ++i) {
        bool_t<C> table_selector = naf_entries_a[i] ^ naf_entries_b[i];
        bool_t<C> sign_selector = naf_entries_b[i];
        Fq to_add_x = T0.x.conditional_select(T1.x, table_selector);
        Fq to_add_y = T0.y.conditional_select(T1.y, table_selector);
        element to_add(to_add_x, to_add_y.conditional_negate(sign_selector));
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

    // compute precomputed lookup table of:
    // B + A
    // B - A
    // -B + A
    // -B - A
    element T0 = base_b + base_a;
    element T1 = base_b - base_a;
    element T2 = base_d + base_c;
    element T3 = base_d - base_c;

    std::array<element, 8> element_table;
    element_table[0] = T2 + T0; // D + C + B + A
    element_table[1] = T2 + T1; // D + C + B - A
    element_table[2] = T2 - T1; // D + C - B + A
    element_table[3] = T2 - T0; // D + C - B - A
    element_table[4] = T3 + T0; // D - C + B + A
    element_table[5] = T3 + T1; // D - C + B - A
    element_table[6] = T3 - T1; // D - C - B + A
    element_table[7] = T3 - T0; // D - C - B - A
    for (size_t i = 0; i < 8; ++i) {
        element_table[i].x.self_reduce();
        element_table[i].y.self_reduce();
    }
    std::array<field_t<C>, 8> x_b0_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[0].element,
                                               element_table[1].x.binary_basis_limbs[0].element,
                                               element_table[2].x.binary_basis_limbs[0].element,
                                               element_table[3].x.binary_basis_limbs[0].element,
                                               element_table[4].x.binary_basis_limbs[0].element,
                                               element_table[5].x.binary_basis_limbs[0].element,
                                               element_table[6].x.binary_basis_limbs[0].element,
                                               element_table[7].x.binary_basis_limbs[0].element);
    std::array<field_t<C>, 8> x_b1_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[1].element,
                                               element_table[1].x.binary_basis_limbs[1].element,
                                               element_table[2].x.binary_basis_limbs[1].element,
                                               element_table[3].x.binary_basis_limbs[1].element,
                                               element_table[4].x.binary_basis_limbs[1].element,
                                               element_table[5].x.binary_basis_limbs[1].element,
                                               element_table[6].x.binary_basis_limbs[1].element,
                                               element_table[7].x.binary_basis_limbs[1].element);
    std::array<field_t<C>, 8> x_b2_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[2].element,
                                               element_table[1].x.binary_basis_limbs[2].element,
                                               element_table[2].x.binary_basis_limbs[2].element,
                                               element_table[3].x.binary_basis_limbs[2].element,
                                               element_table[4].x.binary_basis_limbs[2].element,
                                               element_table[5].x.binary_basis_limbs[2].element,
                                               element_table[6].x.binary_basis_limbs[2].element,
                                               element_table[7].x.binary_basis_limbs[2].element);
    std::array<field_t<C>, 8> x_b3_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[3].element,
                                               element_table[1].x.binary_basis_limbs[3].element,
                                               element_table[2].x.binary_basis_limbs[3].element,
                                               element_table[3].x.binary_basis_limbs[3].element,
                                               element_table[4].x.binary_basis_limbs[3].element,
                                               element_table[5].x.binary_basis_limbs[3].element,
                                               element_table[6].x.binary_basis_limbs[3].element,
                                               element_table[7].x.binary_basis_limbs[3].element);
    std::array<field_t<C>, 8> x_prime_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].x.prime_basis_limb,
                                               element_table[1].x.prime_basis_limb,
                                               element_table[2].x.prime_basis_limb,
                                               element_table[3].x.prime_basis_limb,
                                               element_table[4].x.prime_basis_limb,
                                               element_table[5].x.prime_basis_limb,
                                               element_table[6].x.prime_basis_limb,
                                               element_table[7].x.prime_basis_limb);

    std::array<field_t<C>, 8> y_b0_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[0].element,
                                               element_table[1].y.binary_basis_limbs[0].element,
                                               element_table[2].y.binary_basis_limbs[0].element,
                                               element_table[3].y.binary_basis_limbs[0].element,
                                               element_table[4].y.binary_basis_limbs[0].element,
                                               element_table[5].y.binary_basis_limbs[0].element,
                                               element_table[6].y.binary_basis_limbs[0].element,
                                               element_table[7].y.binary_basis_limbs[0].element);
    std::array<field_t<C>, 8> y_b1_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[1].element,
                                               element_table[1].y.binary_basis_limbs[1].element,
                                               element_table[2].y.binary_basis_limbs[1].element,
                                               element_table[3].y.binary_basis_limbs[1].element,
                                               element_table[4].y.binary_basis_limbs[1].element,
                                               element_table[5].y.binary_basis_limbs[1].element,
                                               element_table[6].y.binary_basis_limbs[1].element,
                                               element_table[7].y.binary_basis_limbs[1].element);
    std::array<field_t<C>, 8> y_b2_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[2].element,
                                               element_table[1].y.binary_basis_limbs[2].element,
                                               element_table[2].y.binary_basis_limbs[2].element,
                                               element_table[3].y.binary_basis_limbs[2].element,
                                               element_table[4].y.binary_basis_limbs[2].element,
                                               element_table[5].y.binary_basis_limbs[2].element,
                                               element_table[6].y.binary_basis_limbs[2].element,
                                               element_table[7].y.binary_basis_limbs[2].element);
    std::array<field_t<C>, 8> y_b3_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[3].element,
                                               element_table[1].y.binary_basis_limbs[3].element,
                                               element_table[2].y.binary_basis_limbs[3].element,
                                               element_table[3].y.binary_basis_limbs[3].element,
                                               element_table[4].y.binary_basis_limbs[3].element,
                                               element_table[5].y.binary_basis_limbs[3].element,
                                               element_table[6].y.binary_basis_limbs[3].element,
                                               element_table[7].y.binary_basis_limbs[3].element);
    std::array<field_t<C>, 8> y_prime_table =
        field_t<C>::preprocess_three_bit_table(element_table[0].y.prime_basis_limb,
                                               element_table[1].y.prime_basis_limb,
                                               element_table[2].y.prime_basis_limb,
                                               element_table[3].y.prime_basis_limb,
                                               element_table[4].y.prime_basis_limb,
                                               element_table[5].y.prime_basis_limb,
                                               element_table[6].y.prime_basis_limb,
                                               element_table[7].y.prime_basis_limb);

    element accumulator = element_table[0].dbl();

    // bool_t<C> t0 = naf_entries_d[1] ^ naf_entries_a[1];
    // bool_t<C> t1 = naf_entries_d[1] ^ naf_entries_b[1];
    // bool_t<C> t2 = naf_entries_d[1] ^ naf_entries_c[1];
    // field_t<C> initial_x_b0 = field_t<C>::select_from_three_bit_table(x_b0_table, t2, t1, t0);
    // field_t<C> initial_x_b1 = field_t<C>::select_from_three_bit_table(x_b1_table, t2, t1, t0);
    // field_t<C> initial_x_b2 = field_t<C>::select_from_three_bit_table(x_b2_table, t2, t1, t0);
    // field_t<C> initial_x_b3 = field_t<C>::select_from_three_bit_table(x_b3_table, t2, t1, t0);
    // field_t<C> initial_x_p = field_t<C>::select_from_three_bit_table(x_prime_table, t2, t1, t0);

    // field_t<C> initial_y_b0 = field_t<C>::select_from_three_bit_table(y_b0_table, t2, t1, t0);
    // field_t<C> initial_y_b1 = field_t<C>::select_from_three_bit_table(y_b1_table, t2, t1, t0);
    // field_t<C> initial_y_b2 = field_t<C>::select_from_three_bit_table(y_b2_table, t2, t1, t0);
    // field_t<C> initial_y_b3 = field_t<C>::select_from_three_bit_table(y_b3_table, t2, t1, t0);
    // field_t<C> initial_y_p = field_t<C>::select_from_three_bit_table(y_prime_table, t2, t1, t0);

    // Fq initial_x;
    // Fq initial_y;
    // initial_x.binary_basis_limbs[0] = typename Fq::Limb(initial_x_b0, Fq::DEFAULT_MAXIMUM_LIMB);
    // initial_x.binary_basis_limbs[1] = typename Fq::Limb(initial_x_b1, Fq::DEFAULT_MAXIMUM_LIMB);
    // initial_x.binary_basis_limbs[2] = typename Fq::Limb(initial_x_b2, Fq::DEFAULT_MAXIMUM_LIMB);
    // initial_x.binary_basis_limbs[3] = typename Fq::Limb(initial_x_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
    // initial_x.prime_basis_limb = initial_x_p;

    // initial_y.binary_basis_limbs[0] = typename Fq::Limb(initial_y_b0, Fq::DEFAULT_MAXIMUM_LIMB);
    // initial_y.binary_basis_limbs[1] = typename Fq::Limb(initial_y_b1, Fq::DEFAULT_MAXIMUM_LIMB);
    // initial_y.binary_basis_limbs[2] = typename Fq::Limb(initial_y_b2, Fq::DEFAULT_MAXIMUM_LIMB);
    // initial_y.binary_basis_limbs[3] = typename Fq::Limb(initial_y_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
    // initial_y.prime_basis_limb = initial_y_p;

    // initial_y = initial_y.conditional_negate(naf_entries_d[1]);
    // accumulator = accumulator + element(initial_x, initial_y);

    for (size_t i = 0; i < num_rounds; ++i) {
        bool_t<C> t0 = naf_entries_d[i] ^ naf_entries_a[i];
        bool_t<C> t1 = naf_entries_d[i] ^ naf_entries_b[i];
        bool_t<C> t2 = naf_entries_d[i] ^ naf_entries_c[i];

        field_t<C> x_b0 = field_t<C>::select_from_three_bit_table(x_b0_table, t2, t1, t0);
        field_t<C> x_b1 = field_t<C>::select_from_three_bit_table(x_b1_table, t2, t1, t0);
        field_t<C> x_b2 = field_t<C>::select_from_three_bit_table(x_b2_table, t2, t1, t0);
        field_t<C> x_b3 = field_t<C>::select_from_three_bit_table(x_b3_table, t2, t1, t0);
        field_t<C> x_p = field_t<C>::select_from_three_bit_table(x_prime_table, t2, t1, t0);

        field_t<C> y_b0 = field_t<C>::select_from_three_bit_table(y_b0_table, t2, t1, t0);
        field_t<C> y_b1 = field_t<C>::select_from_three_bit_table(y_b1_table, t2, t1, t0);
        field_t<C> y_b2 = field_t<C>::select_from_three_bit_table(y_b2_table, t2, t1, t0);
        field_t<C> y_b3 = field_t<C>::select_from_three_bit_table(y_b3_table, t2, t1, t0);
        field_t<C> y_p = field_t<C>::select_from_three_bit_table(y_prime_table, t2, t1, t0);

        Fq to_add_x;
        Fq to_add_y;
        to_add_x.binary_basis_limbs[0] = typename Fq::Limb(x_b0, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[1] = typename Fq::Limb(x_b1, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[2] = typename Fq::Limb(x_b2, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[3] = typename Fq::Limb(x_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        to_add_x.prime_basis_limb = x_p;

        to_add_y.binary_basis_limbs[0] = typename Fq::Limb(y_b0, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[1] = typename Fq::Limb(y_b1, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[2] = typename Fq::Limb(y_b2, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[3] = typename Fq::Limb(y_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        to_add_y.prime_basis_limb = y_p;
        element to_add(to_add_x, to_add_y.conditional_negate(naf_entries_d[i]));
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
element<C, Fq, Fr, T> element<C, Fq, Fr, T>::operator*(const Fr& scalar) const
{
    /**
     *
     * Alright! We can finally tackle scalar multiplications over the *wrong* prime field.
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