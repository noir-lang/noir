#pragma once

#include <random>

namespace barretenberg {
template <typename quadratic_field, typename base_field, typename Fq12Params> class field12 {
  public:
    constexpr field12(const base_field& a = base_field::zero(), const base_field& b = base_field::zero())
        : c0(a)
        , c1(b)
    {}

    constexpr field12(const field12& other)
        : c0(other.c0)
        , c1(other.c1)
    {}

    constexpr field12(field12&& other)
        : c0(other.c0)
        , c1(other.c1)
    {}

    constexpr field12& operator=(const field12& other)
    {
        c0 = other.c0;
        c1 = other.c1;
        return *this;
    }

    constexpr field12& operator=(field12&& other)
    {
        c0 = other.c0;
        c1 = other.c1;
        return *this;
    }

    base_field c0;
    base_field c1;

    struct ell_coeffs {
        quadratic_field o;
        quadratic_field vw;
        quadratic_field vv;
    };

    static constexpr field12 zero() { return { base_field::zero(), base_field::zero() }; };
    static constexpr field12 one() { return { base_field::one(), base_field::zero() }; };

    static constexpr base_field mul_by_non_residue(const base_field& a)
    {
        return {
            base_field::mul_by_non_residue(a.c2),
            a.c0,
            a.c1,
        };
    }

    constexpr field12 operator+(const field12& other) const
    {
        return {
            c0 + other.c0,
            c1 + other.c1,
        };
    }

    constexpr field12 operator-(const field12& other) const
    {
        return {
            c0 - other.c0,
            c1 - other.c1,
        };
    }

    constexpr field12 operator*(const field12& other) const
    {
        base_field T0 = c0 * other.c0;
        base_field T1 = c1 * other.c1;
        base_field T2 = c0 + c1;
        base_field T3 = other.c0 + other.c1;

        return {
            mul_by_non_residue(T1) + T0,
            T2 * T3 - (T0 + T1),
        };
    }

    constexpr field12 operator/(const field12& other) const { return operator*(other.invert()); }

    constexpr field12 operator+=(const field12& other)
    {
        c0 += other.c0;
        c1 += other.c1;
        return *this;
    }

    constexpr field12 operator-=(const field12& other)
    {
        c0 -= other.c0;
        c1 -= other.c1;
        return *this;
    }

    constexpr field12 operator*=(const field12& other)
    {
        *this = operator*(other);
        return *this;
    }

    constexpr field12 operator/=(const field12& other)
    {
        *this = operator/(other);
        return *this;
    }

    constexpr void self_sparse_mul(const ell_coeffs& ell)
    {
        // multiplicand is sparse fp12 element (ell.0, 0, ell.vv) + \beta(0, ell.vw, 0)
        quadratic_field d0 = c0.c0 * ell.o;
        quadratic_field d2 = c0.c2 * ell.vv;
        quadratic_field d4 = c1.c1 * ell.vw;
        quadratic_field t2 = c0.c0 + c1.c1;
        quadratic_field t1 = c0.c0 + c0.c2;
        quadratic_field s0 = c0.c1 + c1.c0;
        s0 += c1.c2;

        quadratic_field s1 = c0.c1 * ell.vv;
        quadratic_field t3 = s1 + d4;
        quadratic_field t4 = base_field::mul_by_non_residue(t3);
        c0.c0 = t4 + d0;

        t3 = c1.c2 * ell.vw;
        s1 += t3;
        t3 += d2;
        t4 = base_field::mul_by_non_residue(t3);
        t3 = c0.c1 * ell.o;
        s1 += t3;
        c0.c1 = t4 + t3;

        quadratic_field t0 = ell.o + ell.vv;
        t3 = t1 * t0;
        t3 -= d0;
        t3 -= d2;
        t4 = c1.c0 * ell.vw;
        s1 += t4;

        t0 = c0.c2 + c1.c1;
        c0.c2 = t3 + t4;

        t1 = ell.vv + ell.vw;
        t3 = t0 * t1;
        t3 -= d2;
        t3 -= d4;
        t4 = base_field::mul_by_non_residue(t3);
        t3 = c1.c0 * ell.o;
        s1 += t3;
        c1.c0 = t3 + t4;

        t3 = c1.c2 * ell.vv;
        s1 += t3;
        t4 = base_field::mul_by_non_residue(t3);
        t0 = ell.o + ell.vw;
        t3 = t0 * t2;
        t3 -= d0;
        t3 -= d4;
        c1.c1 = t3 + t4;

        t0 = ell.o + ell.vv;
        t0 += ell.vw;
        t3 = s0 * t0;
        c1.c2 = t3 - s1;
    }

    constexpr field12 sqr() const
    {
        base_field T0 = c0 + c1;
        base_field T1 = mul_by_non_residue(c1) + c0;

        T0 *= T1;
        T1 = c0 * c1;

        return {
            T0 - (T1 + mul_by_non_residue(T1)),
            T1 + T1,
        };
    }

    constexpr field12 invert() const
    {
        /* From "High-Speed Software Implementation of the Optimal Ate Pairing over Barreto-Naehrig Curves"; Algorithm 8
         */
        base_field T0 = (c0.sqr() - mul_by_non_residue(c1.sqr())).invert();
        return {
            c0 * T0,
            -(c1 * T0),
        };
    }

    constexpr field12 frobenius_map_three() const
    {
        return {
            c0.frobenius_map_three(),
            c1.frobenius_map_three().mul_by_fq2(Fq12Params::frobenius_coefficients_3),
        };
    }

    constexpr field12 frobenius_map_two() const
    {
        return {
            c0.frobenius_map_two(),
            c1.frobenius_map_two().mul_by_fq2(Fq12Params::frobenius_coefficients_2),
        };
    }

    constexpr field12 frobenius_map_one() const
    {
        return {
            c0.frobenius_map_one(),
            c1.frobenius_map_one().mul_by_fq2(Fq12Params::frobenius_coefficients_1),
        };
    }

    constexpr field12 cyclotomic_squared() const
    {
        // TODO: write more efficient version...
        return sqr();
    }

    constexpr field12 unitary_inverse() const
    {
        return {
            c0,
            -c1,
        };
    }

    static constexpr field12 random_element(std::mt19937_64* engine = nullptr,
                                            std::uniform_int_distribution<uint64_t>* dist = nullptr)
    {
        return {
            base_field::random_element(engine, dist),
            base_field::random_element(engine, dist),
        };
    }

    constexpr field12 to_montgomery_form()
    {
        return {
            c0.to_montgomery_form(),
            c1.to_montgomery_form(),
        };
    }

    constexpr field12 from_montgomery_form()
    {
        return {
            c0.from_montgomery_form(),
            c1.from_montgomery_form(),
        };
    }

    constexpr bool is_zero() const { return c0.is_zero() && c1.is_zero(); }

    constexpr bool operator==(const field12& other) const { return c0 == other.c0 && c1 == other.c1; }
};
} // namespace barretenberg
