#pragma once

#include "../byte_array/byte_array.hpp"
#include "../bigfield/bigfield.hpp"
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

// ( ͡° ͜ʖ ͡°)
template <typename Composer, class Fq, class Fr, class Params> class element {
  public:
    element();
    element(const Fq& x, const Fq& y);

    element(const element& other);
    element(element&& other);

    bool_t<Composer> on_curve()
    {
        // TODO FIX
        // return bool_t<Composer>(get_context(), true);
        Fq xx = x.sqr();
        Fq lhs = xx * x;
        Fq rhs = y.sqr();
        Fq b(get_context(), uint256_t(Params::b));
        lhs = lhs + b;
        if constexpr (Params::has_a) {
            Fq a(get_context(), uint256_t(Params::a));
            lhs = lhs + (a * x);
        }
        Fq result = lhs - rhs;
        result.assert_is_in_field();
        field_t<Composer> product(get_context());
        for (size_t i = 0; i < 4; ++i) {
            product = product * result.binary_basis_limbs[i].element;
        }
        return product.is_zero();
    }

    static element one(Composer* ctx)
    {
        uint256_t x = uint256_t(Params::one_x);
        uint256_t y = uint256_t(Params::one_y);
        Fq x_fq(ctx, x);
        Fq y_fq(ctx, y);
        element result(x_fq, y_fq);
        return result;
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
        // TODO fix
        result.y = result.y.conditional_negate(bool_t<Composer>(get_context(), true));
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

    static element twin_mul(const element& base_a, const Fr& scalar_a, const element& base_b, const Fr& scalar_b);

    static element quad_mul(const element& base_a,
                            const Fr& scalar_a,
                            const element& base_b,
                            const Fr& scalar_b,
                            const element& base_c,
                            const Fr& scalar_c,
                            const element& base_d,
                            const Fr& scalar_d);

    static element batch_mul(const std::vector<element>& points, const std::vector<Fr>& scalars);

    static std::vector<bool_t<Composer>> compute_naf(const Fr& scalar);
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

    struct twin_lookup_table {
        twin_lookup_table(const std::array<element, 2>& inputs)
        {
            T0 = inputs[1] + inputs[0];
            T1 = inputs[1] - inputs[0];
            T0.x.self_reduce();
            T0.y.self_reduce();
            T1.x.self_reduce();
            T1.y.self_reduce();
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
            for (size_t i = 0; i < 8; ++i) {
                element_table[i].x.self_reduce();
                element_table[i].y.self_reduce();
            }
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
            x_prime_table = field_t<Composer>::preprocess_three_bit_table(element_table[0].x.prime_basis_limb,
                                                                          element_table[1].x.prime_basis_limb,
                                                                          element_table[2].x.prime_basis_limb,
                                                                          element_table[3].x.prime_basis_limb,
                                                                          element_table[4].x.prime_basis_limb,
                                                                          element_table[5].x.prime_basis_limb,
                                                                          element_table[6].x.prime_basis_limb,
                                                                          element_table[7].x.prime_basis_limb);

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
            y_prime_table = field_t<Composer>::preprocess_three_bit_table(element_table[0].y.prime_basis_limb,
                                                                          element_table[1].y.prime_basis_limb,
                                                                          element_table[2].y.prime_basis_limb,
                                                                          element_table[3].y.prime_basis_limb,
                                                                          element_table[4].y.prime_basis_limb,
                                                                          element_table[5].y.prime_basis_limb,
                                                                          element_table[6].y.prime_basis_limb,
                                                                          element_table[7].y.prime_basis_limb);
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
            field_t<Composer> x_p = field_t<Composer>::select_from_three_bit_table(x_prime_table, t2, t1, t0);

            field_t<Composer> y_b0 = field_t<Composer>::select_from_three_bit_table(y_b0_table, t2, t1, t0);
            field_t<Composer> y_b1 = field_t<Composer>::select_from_three_bit_table(y_b1_table, t2, t1, t0);
            field_t<Composer> y_b2 = field_t<Composer>::select_from_three_bit_table(y_b2_table, t2, t1, t0);
            field_t<Composer> y_b3 = field_t<Composer>::select_from_three_bit_table(y_b3_table, t2, t1, t0);
            field_t<Composer> y_p = field_t<Composer>::select_from_three_bit_table(y_prime_table, t2, t1, t0);

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

            if (uint256_t(to_add_y.binary_basis_limbs[1].element.get_value()) > Fq::DEFAULT_MAXIMUM_LIMB) {
                std::cout << "error? " << to_add_y.binary_basis_limbs[1].element.get_value() << "vs "
                          << Fq::DEFAULT_MAXIMUM_LIMB << std::endl;
            }
            to_add_y.prime_basis_limb = y_p;
            element to_add(to_add_x, to_add_y.conditional_negate(v3));
            return to_add;
        }

        element operator[](const size_t idx) const { return element_table[idx]; }

        std::array<field_t<Composer>, 8> x_b0_table;
        std::array<field_t<Composer>, 8> x_b1_table;
        std::array<field_t<Composer>, 8> x_b2_table;
        std::array<field_t<Composer>, 8> x_b3_table;
        std::array<field_t<Composer>, 8> x_prime_table;

        std::array<field_t<Composer>, 8> y_b0_table;
        std::array<field_t<Composer>, 8> y_b1_table;
        std::array<field_t<Composer>, 8> y_b2_table;
        std::array<field_t<Composer>, 8> y_b3_table;
        std::array<field_t<Composer>, 8> y_prime_table;

        std::array<element, 8> element_table;
    };
};
} // namespace stdlib
} // namespace plonk

#include "biggroup_impl.hpp"