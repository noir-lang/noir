#pragma once

#include "../byte_array/byte_array.hpp"
#include "../bigfield/bigfield.hpp"
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

// ( ͡° ͜ʖ ͡°)
template <typename Composer, class Fq, class Fr, class NativeGroup> class element {
  public:
    element();
    element(const typename NativeGroup::affine_element& input);
    element(const Fq& x, const Fq& y);

    element(const element& other);
    element(element&& other);

    void validate_on_curve()
    {
        Fq xx = x.sqr();
        Fq lhs = xx * x;
        Fq rhs = y.sqr();
        Fq b(get_context(), uint256_t(NativeGroup::curve_b));
        lhs = lhs + b;
        if constexpr (NativeGroup::has_a) {
            Fq a(get_context(), uint256_t(NativeGroup::curve_a));
            lhs = lhs + (a * x);
        }
        lhs.assert_equal(rhs);
    }

    static element one(Composer* ctx)
    {
        uint256_t x = uint256_t(NativeGroup::one.x);
        uint256_t y = uint256_t(NativeGroup::one.y);
        Fq x_fq(ctx, x);
        Fq y_fq(ctx, y);
        return element(x_fq, y_fq);
    }

    element& operator=(const element& other);
    element& operator=(element&& other);

    byte_array<Composer> to_byte_array() const
    {
        byte_array<Composer> result(get_context());
        result.write(y.to_byte_array());
        result.write(x.to_byte_array());
        return result;
    }

    element operator+(const element& other) const;
    element operator-(const element& other) const;
    element operator-() const
    {
        element result(*this);
        result.y = -result.y;
        return result;
    }

    element operator*(const Fr& other) const;

    element conditional_negate(const bool_t<Composer>& predicate) const
    {
        element result(*this);
        result.y = result.y.conditional_negate(predicate);
        return result;
    }

    element normalize() const
    {
        element result(*this);
        result.x.assert_is_in_field();
        result.y.assert_is_in_field();
        return result;
    }

    element dbl() const;
    element montgomery_ladder(const element& other) const;

    static element batch_mul(const std::vector<element>& points,
                             const std::vector<Fr>& scalars,
                             const size_t max_num_bits = 0);

    static element mixed_batch_mul(const std::vector<element>& big_points,
                                   const std::vector<Fr>& big_scalars,
                                   const std::vector<element>& small_points,
                                   const std::vector<Fr>& small_scalars,
                                   const size_t max_num_small_bits);

    static std::vector<bool_t<Composer>> compute_naf(const Fr& scalar, const size_t max_num_bits = 0);

    Composer* get_context() const
    {
        if (x.context != nullptr) {
            return x.context;
        }
        if (y.context != nullptr) {
            return y.context;
        }
        return nullptr;
    }

    Composer* get_context(const element& other) const
    {
        if (x.context != nullptr) {
            return x.context;
        }
        if (y.context != nullptr) {
            return y.context;
        }
        if (other.x.context != nullptr) {
            return other.x.context;
        }
        if (other.y.context != nullptr) {
            return other.y.context;
        }
    }

    Fq x;
    Fq y;

  private:
    static std::pair<element, element> compute_offset_generators(const size_t num_rounds);

    struct twin_lookup_table {
        twin_lookup_table(const std::array<element, 2>& inputs)
        {
            T0 = inputs[1] + inputs[0];
            T1 = inputs[1] - inputs[0];
        }

        twin_lookup_table(const twin_lookup_table& other) = default;
        twin_lookup_table& operator=(const twin_lookup_table& other) = default;

        element get(const bool_t<Composer>& v0, const bool_t<Composer>& v1) const
        {
            bool_t<Composer> table_selector = v0 ^ v1;
            bool_t<Composer> sign_selector = v1;
            Fq to_add_x = T0.x.conditional_select(T1.x, table_selector);
            Fq to_add_y = T0.y.conditional_select(T1.y, table_selector);
            element to_add(to_add_x, to_add_y.conditional_negate(sign_selector));
            return to_add;
        }

        element operator[](const size_t idx) const
        {
            if (idx == 0) {
                return T0;
            }
            return T1;
        }

        element T0;
        element T1;
    };

    struct triple_lookup_table {
        triple_lookup_table(const std::array<element, 3>& inputs)
        {
            element T0 = inputs[1] + inputs[0];
            element T1 = inputs[1] - inputs[0];
            element_table[0] = inputs[2] + T0; // C + B + A
            element_table[1] = inputs[2] + T1; // C + B - A
            element_table[2] = inputs[2] - T1; // C - B + A
            element_table[3] = inputs[2] - T0; // C - B - A

            x_b0_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[0].element,
                                                                     element_table[1].x.binary_basis_limbs[0].element,
                                                                     element_table[2].x.binary_basis_limbs[0].element,
                                                                     element_table[3].x.binary_basis_limbs[0].element);
            x_b1_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[1].element,
                                                                     element_table[1].x.binary_basis_limbs[1].element,
                                                                     element_table[2].x.binary_basis_limbs[1].element,
                                                                     element_table[3].x.binary_basis_limbs[1].element);
            x_b2_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[2].element,
                                                                     element_table[1].x.binary_basis_limbs[2].element,
                                                                     element_table[2].x.binary_basis_limbs[2].element,
                                                                     element_table[3].x.binary_basis_limbs[2].element);
            x_b3_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[3].element,
                                                                     element_table[1].x.binary_basis_limbs[3].element,
                                                                     element_table[2].x.binary_basis_limbs[3].element,
                                                                     element_table[3].x.binary_basis_limbs[3].element);

            y_b0_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[0].element,
                                                                     element_table[1].y.binary_basis_limbs[0].element,
                                                                     element_table[2].y.binary_basis_limbs[0].element,
                                                                     element_table[3].y.binary_basis_limbs[0].element);
            y_b1_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[1].element,
                                                                     element_table[1].y.binary_basis_limbs[1].element,
                                                                     element_table[2].y.binary_basis_limbs[1].element,
                                                                     element_table[3].y.binary_basis_limbs[1].element);
            y_b2_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[2].element,
                                                                     element_table[1].y.binary_basis_limbs[2].element,
                                                                     element_table[2].y.binary_basis_limbs[2].element,
                                                                     element_table[3].y.binary_basis_limbs[2].element);
            y_b3_table = field_t<Composer>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[3].element,
                                                                     element_table[1].y.binary_basis_limbs[3].element,
                                                                     element_table[2].y.binary_basis_limbs[3].element,
                                                                     element_table[3].y.binary_basis_limbs[3].element);
        }

        triple_lookup_table(const triple_lookup_table& other) = default;
        triple_lookup_table& operator=(const triple_lookup_table& other) = default;

        element get(const bool_t<Composer>& v0, const bool_t<Composer>& v1, const bool_t<Composer>& v2) const
        {
            bool_t<Composer> t0 = v2 ^ v0;
            bool_t<Composer> t1 = v2 ^ v1;

            field_t<Composer> x_b0 = field_t<Composer>::select_from_two_bit_table(x_b0_table, t1, t0);
            field_t<Composer> x_b1 = field_t<Composer>::select_from_two_bit_table(x_b1_table, t1, t0);
            field_t<Composer> x_b2 = field_t<Composer>::select_from_two_bit_table(x_b2_table, t1, t0);
            field_t<Composer> x_b3 = field_t<Composer>::select_from_two_bit_table(x_b3_table, t1, t0);

            field_t<Composer> y_b0 = field_t<Composer>::select_from_two_bit_table(y_b0_table, t1, t0);
            field_t<Composer> y_b1 = field_t<Composer>::select_from_two_bit_table(y_b1_table, t1, t0);
            field_t<Composer> y_b2 = field_t<Composer>::select_from_two_bit_table(y_b2_table, t1, t0);
            field_t<Composer> y_b3 = field_t<Composer>::select_from_two_bit_table(y_b3_table, t1, t0);

            Fq to_add_x;
            Fq to_add_y;
            to_add_x.binary_basis_limbs[0] = typename Fq::Limb(x_b0, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_x.binary_basis_limbs[1] = typename Fq::Limb(x_b1, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_x.binary_basis_limbs[2] = typename Fq::Limb(x_b2, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_x.binary_basis_limbs[3] = typename Fq::Limb(x_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
            to_add_x.prime_basis_limb =
                to_add_x.binary_basis_limbs[0].element.add_two(to_add_x.binary_basis_limbs[1].element * Fq::shift_1,
                                                               to_add_x.binary_basis_limbs[2].element * Fq::shift_2);
            to_add_x.prime_basis_limb += to_add_x.binary_basis_limbs[3].element * Fq::shift_3;

            to_add_y.binary_basis_limbs[0] = typename Fq::Limb(y_b0, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_y.binary_basis_limbs[1] = typename Fq::Limb(y_b1, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_y.binary_basis_limbs[2] = typename Fq::Limb(y_b2, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_y.binary_basis_limbs[3] = typename Fq::Limb(y_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);

            to_add_y.prime_basis_limb =
                to_add_y.binary_basis_limbs[0].element.add_two(to_add_y.binary_basis_limbs[1].element * Fq::shift_1,
                                                               to_add_y.binary_basis_limbs[2].element * Fq::shift_2);
            to_add_y.prime_basis_limb += to_add_y.binary_basis_limbs[3].element * Fq::shift_3;
            element to_add(to_add_x, to_add_y.conditional_negate(v2));

            return to_add;
        }

        element operator[](const size_t idx) const { return element_table[idx]; }

        std::array<field_t<Composer>, 4> x_b0_table;
        std::array<field_t<Composer>, 4> x_b1_table;
        std::array<field_t<Composer>, 4> x_b2_table;
        std::array<field_t<Composer>, 4> x_b3_table;

        std::array<field_t<Composer>, 4> y_b0_table;
        std::array<field_t<Composer>, 4> y_b1_table;
        std::array<field_t<Composer>, 4> y_b2_table;
        std::array<field_t<Composer>, 4> y_b3_table;

        std::array<element, 4> element_table;
    };

    struct quad_lookup_table {
        quad_lookup_table(const std::array<element, 4>& inputs)
        {
            element T0 = inputs[1] + inputs[0];
            element T1 = inputs[1] - inputs[0];
            element T2 = inputs[3] + inputs[2];
            element T3 = inputs[3] - inputs[2];

            element_table[0] = T2 + T0; // D + C + B + A
            element_table[1] = T2 + T1; // D + C + B - A
            element_table[2] = T2 - T1; // D + C - B + A
            element_table[3] = T2 - T0; // D + C - B - A
            element_table[4] = T3 + T0; // D - C + B + A
            element_table[5] = T3 + T1; // D - C + B - A
            element_table[6] = T3 - T1; // D - C - B + A
            element_table[7] = T3 - T0; // D - C - B - A

            x_b0_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[0].element,
                                                              element_table[1].x.binary_basis_limbs[0].element,
                                                              element_table[2].x.binary_basis_limbs[0].element,
                                                              element_table[3].x.binary_basis_limbs[0].element,
                                                              element_table[4].x.binary_basis_limbs[0].element,
                                                              element_table[5].x.binary_basis_limbs[0].element,
                                                              element_table[6].x.binary_basis_limbs[0].element,
                                                              element_table[7].x.binary_basis_limbs[0].element);
            x_b1_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[1].element,
                                                              element_table[1].x.binary_basis_limbs[1].element,
                                                              element_table[2].x.binary_basis_limbs[1].element,
                                                              element_table[3].x.binary_basis_limbs[1].element,
                                                              element_table[4].x.binary_basis_limbs[1].element,
                                                              element_table[5].x.binary_basis_limbs[1].element,
                                                              element_table[6].x.binary_basis_limbs[1].element,
                                                              element_table[7].x.binary_basis_limbs[1].element);
            x_b2_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[2].element,
                                                              element_table[1].x.binary_basis_limbs[2].element,
                                                              element_table[2].x.binary_basis_limbs[2].element,
                                                              element_table[3].x.binary_basis_limbs[2].element,
                                                              element_table[4].x.binary_basis_limbs[2].element,
                                                              element_table[5].x.binary_basis_limbs[2].element,
                                                              element_table[6].x.binary_basis_limbs[2].element,
                                                              element_table[7].x.binary_basis_limbs[2].element);
            x_b3_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[3].element,
                                                              element_table[1].x.binary_basis_limbs[3].element,
                                                              element_table[2].x.binary_basis_limbs[3].element,
                                                              element_table[3].x.binary_basis_limbs[3].element,
                                                              element_table[4].x.binary_basis_limbs[3].element,
                                                              element_table[5].x.binary_basis_limbs[3].element,
                                                              element_table[6].x.binary_basis_limbs[3].element,
                                                              element_table[7].x.binary_basis_limbs[3].element);

            y_b0_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[0].element,
                                                              element_table[1].y.binary_basis_limbs[0].element,
                                                              element_table[2].y.binary_basis_limbs[0].element,
                                                              element_table[3].y.binary_basis_limbs[0].element,
                                                              element_table[4].y.binary_basis_limbs[0].element,
                                                              element_table[5].y.binary_basis_limbs[0].element,
                                                              element_table[6].y.binary_basis_limbs[0].element,
                                                              element_table[7].y.binary_basis_limbs[0].element);
            y_b1_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[1].element,
                                                              element_table[1].y.binary_basis_limbs[1].element,
                                                              element_table[2].y.binary_basis_limbs[1].element,
                                                              element_table[3].y.binary_basis_limbs[1].element,
                                                              element_table[4].y.binary_basis_limbs[1].element,
                                                              element_table[5].y.binary_basis_limbs[1].element,
                                                              element_table[6].y.binary_basis_limbs[1].element,
                                                              element_table[7].y.binary_basis_limbs[1].element);
            y_b2_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[2].element,
                                                              element_table[1].y.binary_basis_limbs[2].element,
                                                              element_table[2].y.binary_basis_limbs[2].element,
                                                              element_table[3].y.binary_basis_limbs[2].element,
                                                              element_table[4].y.binary_basis_limbs[2].element,
                                                              element_table[5].y.binary_basis_limbs[2].element,
                                                              element_table[6].y.binary_basis_limbs[2].element,
                                                              element_table[7].y.binary_basis_limbs[2].element);
            y_b3_table =
                field_t<Composer>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[3].element,
                                                              element_table[1].y.binary_basis_limbs[3].element,
                                                              element_table[2].y.binary_basis_limbs[3].element,
                                                              element_table[3].y.binary_basis_limbs[3].element,
                                                              element_table[4].y.binary_basis_limbs[3].element,
                                                              element_table[5].y.binary_basis_limbs[3].element,
                                                              element_table[6].y.binary_basis_limbs[3].element,
                                                              element_table[7].y.binary_basis_limbs[3].element);
        }
        quad_lookup_table(const quad_lookup_table& other) = default;
        quad_lookup_table& operator=(const quad_lookup_table& other) = default;

        element get(const bool_t<Composer>& v0,
                    const bool_t<Composer>& v1,
                    const bool_t<Composer>& v2,
                    const bool_t<Composer>& v3) const
        {
            bool_t<Composer> t0 = v3 ^ v0;
            bool_t<Composer> t1 = v3 ^ v1;
            bool_t<Composer> t2 = v3 ^ v2;

            field_t<Composer> x_b0 = field_t<Composer>::select_from_three_bit_table(x_b0_table, t2, t1, t0);
            field_t<Composer> x_b1 = field_t<Composer>::select_from_three_bit_table(x_b1_table, t2, t1, t0);
            field_t<Composer> x_b2 = field_t<Composer>::select_from_three_bit_table(x_b2_table, t2, t1, t0);
            field_t<Composer> x_b3 = field_t<Composer>::select_from_three_bit_table(x_b3_table, t2, t1, t0);

            field_t<Composer> y_b0 = field_t<Composer>::select_from_three_bit_table(y_b0_table, t2, t1, t0);
            field_t<Composer> y_b1 = field_t<Composer>::select_from_three_bit_table(y_b1_table, t2, t1, t0);
            field_t<Composer> y_b2 = field_t<Composer>::select_from_three_bit_table(y_b2_table, t2, t1, t0);
            field_t<Composer> y_b3 = field_t<Composer>::select_from_three_bit_table(y_b3_table, t2, t1, t0);

            Fq to_add_x;
            Fq to_add_y;
            to_add_x.binary_basis_limbs[0] = typename Fq::Limb(x_b0, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_x.binary_basis_limbs[1] = typename Fq::Limb(x_b1, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_x.binary_basis_limbs[2] = typename Fq::Limb(x_b2, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_x.binary_basis_limbs[3] = typename Fq::Limb(x_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
            to_add_x.prime_basis_limb =
                to_add_x.binary_basis_limbs[0].element.add_two(to_add_x.binary_basis_limbs[1].element * Fq::shift_1,
                                                               to_add_x.binary_basis_limbs[2].element * Fq::shift_2);
            to_add_x.prime_basis_limb += to_add_x.binary_basis_limbs[3].element * Fq::shift_3;

            to_add_y.binary_basis_limbs[0] = typename Fq::Limb(y_b0, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_y.binary_basis_limbs[1] = typename Fq::Limb(y_b1, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_y.binary_basis_limbs[2] = typename Fq::Limb(y_b2, Fq::DEFAULT_MAXIMUM_LIMB);
            to_add_y.binary_basis_limbs[3] = typename Fq::Limb(y_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
            to_add_y.prime_basis_limb =
                to_add_y.binary_basis_limbs[0].element.add_two(to_add_y.binary_basis_limbs[1].element * Fq::shift_1,
                                                               to_add_y.binary_basis_limbs[2].element * Fq::shift_2);
            to_add_y.prime_basis_limb += to_add_y.binary_basis_limbs[3].element * Fq::shift_3;

            element to_add(to_add_x, to_add_y.conditional_negate(v3));

            return to_add;
        }

        element operator[](const size_t idx) const { return element_table[idx]; }

        std::array<field_t<Composer>, 8> x_b0_table;
        std::array<field_t<Composer>, 8> x_b1_table;
        std::array<field_t<Composer>, 8> x_b2_table;
        std::array<field_t<Composer>, 8> x_b3_table;

        std::array<field_t<Composer>, 8> y_b0_table;
        std::array<field_t<Composer>, 8> y_b1_table;
        std::array<field_t<Composer>, 8> y_b2_table;
        std::array<field_t<Composer>, 8> y_b3_table;

        std::array<element, 8> element_table;
    };

    struct batch_lookup_table {
        batch_lookup_table(const std::vector<element>& points)
        {
            num_points = points.size();
            num_quads = num_points / 4;

            has_triple = ((num_quads * 4) < num_points - 2) && (num_points >= 3);

            has_twin = ((num_quads * 4 + (size_t)has_triple * 3) < num_points - 1) && (num_points >= 2);

            has_singleton = num_points != ((num_quads * 4) + ((size_t)has_triple * 3) + ((size_t)has_twin * 2));

            for (size_t i = 0; i < num_quads; ++i) {
                quad_tables.push_back(
                    quad_lookup_table({ points[4 * i], points[4 * i + 1], points[4 * i + 2], points[4 * i + 3] }));
            }

            if (has_triple) {
                triple_tables.push_back(triple_lookup_table(
                    { points[4 * num_quads], points[4 * num_quads + 1], points[4 * num_quads + 2] }));
            }
            if (has_twin) {
                twin_tables.push_back(twin_lookup_table({ points[4 * num_quads], points[4 * num_quads + 1] }));
            }

            if (has_singleton) {
                singletons.push_back(points[points.size() - 1]);
                // singletons[0].x.self_reduce();
                // singletons[0].y.self_reduce();
            }
        }

        element get_initial_entry() const
        {
            std::vector<element> add_accumulator;
            for (size_t i = 0; i < num_quads; ++i) {
                add_accumulator.push_back(quad_tables[i][0]);
            }
            if (has_twin) {
                add_accumulator.push_back(twin_tables[0][0]);
            }
            if (has_triple) {
                add_accumulator.push_back(triple_tables[0][0]);
            }
            if (has_singleton) {
                add_accumulator.push_back(singletons[0]);
            }

            element accumulator = add_accumulator[0];
            for (size_t i = 1; i < add_accumulator.size(); ++i) {
                accumulator = accumulator + add_accumulator[i];
            }
            return accumulator;
        }

        element get(std::vector<bool_t<Composer>>& naf_entries) const
        {
            std::vector<element> round_accumulator;
            for (size_t j = 0; j < num_quads; ++j) {
                round_accumulator.push_back(quad_tables[j].get(
                    naf_entries[4 * j], naf_entries[4 * j + 1], naf_entries[4 * j + 2], naf_entries[4 * j + 3]));
            }
            if (has_triple) {
                round_accumulator.push_back(triple_tables[0].get(
                    naf_entries[num_quads * 4], naf_entries[num_quads * 4 + 1], naf_entries[num_quads * 4 + 2]));
            }
            if (has_twin) {
                round_accumulator.push_back(
                    twin_tables[0].get(naf_entries[num_quads * 4], naf_entries[num_quads * 4 + 1]));
            }
            if (has_singleton) {
                round_accumulator.push_back(singletons[0].conditional_negate(naf_entries[num_points - 1]));
            }

            element result = round_accumulator[0];
            for (size_t j = 1; j < round_accumulator.size(); ++j) {
                result = result + round_accumulator[j];
            }
            return result;
        }

        std::vector<quad_lookup_table> quad_tables;
        std::vector<triple_lookup_table> triple_tables;
        std::vector<twin_lookup_table> twin_tables;
        std::vector<element> singletons;
        size_t num_points;
        size_t num_quads;
        bool has_triple;
        bool has_twin;
        bool has_singleton;
    };
};

template <typename C, typename Fq, typename Fr, typename G>
inline std::ostream& operator<<(std::ostream& os, element<C, Fq, Fr, G> const& v)
{
    return os << "{ " << v.x << " , " << v.y << " }";
}
} // namespace stdlib
} // namespace plonk

#include "biggroup_impl.hpp"