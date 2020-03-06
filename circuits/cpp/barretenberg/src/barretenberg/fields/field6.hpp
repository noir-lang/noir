#pragma once

#include <random>

namespace barretenberg {
template <typename base_field, typename Fq6Params> class field6 {
  public:
    constexpr field6(const base_field& a = base_field::zero(),
                     const base_field& b = base_field::zero(),
                     const base_field& c = base_field::zero())
        : c0(a)
        , c1(b)
        , c2(c)
    {}

    constexpr field6(const field6& other)
        : c0(other.c0)
        , c1(other.c1)
        , c2(other.c2)
    {}

    constexpr field6(field6&& other)
        : c0(other.c0)
        , c1(other.c1)
        , c2(other.c2)
    {}

    constexpr field6& operator=(const field6& other)
    {
        c0 = other.c0;
        c1 = other.c1;
        c2 = other.c2;
        return *this;
    }

    constexpr field6& operator=(field6&& other)
    {
        c0 = other.c0;
        c1 = other.c1;
        c2 = other.c2;
        return *this;
    }

    base_field c0;
    base_field c1;
    base_field c2;

    static constexpr field6 zero() { return { base_field::zero(), base_field::zero(), base_field::zero() }; };
    static constexpr field6 one() { return { base_field::one(), base_field::zero(), base_field::zero() }; };

    static constexpr base_field mul_by_non_residue(const base_field& a) { return Fq6Params::mul_by_non_residue(a); }

    constexpr field6 operator+(const field6& other) const
    {
        return {
            c0 + other.c0,
            c1 + other.c1,
            c2 + other.c2,
        };
    }

    constexpr field6 operator-(const field6& other) const
    {
        return {
            c0 - other.c0,
            c1 - other.c1,
            c2 - other.c2,
        };
    }

    constexpr field6 operator-() const
    {
        return {
            -c0,
            -c1,
            -c2,
        };
    }

    constexpr field6 operator*(const field6& other) const
    {
        // /* Devegili OhEig Scott Dahab --- Multiplication and Squaring on Pairing-Friendly Fields.pdf; Section 4
        //  * (Karatsuba) */

        base_field T0 = c0 * other.c0;
        base_field T1 = c1 * other.c1;
        base_field T2 = c2 * other.c2;

        base_field T3 = (c0 + c2) * (other.c0 + other.c2);
        base_field T4 = (c0 + c1) * (other.c0 + other.c1);
        base_field T5 = (c1 + c2) * (other.c1 + other.c2);

        return {
            T0 + mul_by_non_residue(T5 - (T1 + T2)),
            T4 - (T0 + T1) + mul_by_non_residue(T2),
            T3 + T1 - (T0 + T2),
        };
    }

    constexpr field6 operator/(const field6& other) const { return operator*(other.invert()); }

    constexpr field6 sqr() const
    {
        /* Devegili OhEig Scott Dahab --- Multiplication and Squaring on Pairing-Friendly Fields.pdf; Section 4
         * (CH-SQR2) */
        base_field S0 = c0.sqr();
        base_field S1 = c0 * c1;
        S1 += S1;
        base_field S2 = (c0 + c2 - c1).sqr();
        base_field S3 = c1 * c2;
        S3 += S3;
        base_field S4 = c2.sqr();
        return {
            mul_by_non_residue(S3) + S0,
            mul_by_non_residue(S4) + S1,
            S1 + S2 + S3 - S0 - S4,
        };
    }

    constexpr field6 operator+=(const field6& other)
    {
        c0 += other.c0;
        c1 += other.c1;
        c2 += other.c2;
        return *this;
    }

    constexpr field6 operator-=(const field6& other)
    {
        c0 -= other.c0;
        c1 -= other.c1;
        c2 -= other.c2;
        return *this;
    }

    constexpr field6 operator*=(const field6& other)
    {
        *this = operator*(other);
        return *this;
    }

    constexpr field6 operator/=(const field6& other)
    {
        *this = operator/(other);
        return *this;
    }

    constexpr field6 invert() const
    {
        /* From "High-Speed Software Implementation of the Optimal Ate Pairing over Barreto-Naehrig Curves"; Algorithm
         * 17 */
        base_field C0 = c0.sqr() - mul_by_non_residue(c1 * c2);
        base_field C1 = mul_by_non_residue(c2.sqr()) - (c0 * c1);
        base_field C2 = c1.sqr() - (c0 * c2);
        base_field T0 = ((c0 * C0) + mul_by_non_residue((c2 * C1) + (c1 * C2))).invert();

        return {
            T0 * C0,
            T0 * C1,
            T0 * C2,
        };
    }

    constexpr field6 mul_by_fq2(const base_field& other) const { return { other * c0, other * c1, other * c2 }; }

    constexpr field6 frobenius_map_three() const
    {
        return {
            c0.frobenius_map(),
            Fq6Params::frobenius_coeffs_c1_3 * c1.frobenius_map(),
            Fq6Params::frobenius_coeffs_c2_3 * c2.frobenius_map(),
        };
    }

    constexpr field6 frobenius_map_two() const
    {
        return { c0, Fq6Params::frobenius_coeffs_c1_2 * c1, Fq6Params::frobenius_coeffs_c2_2 * c2 };
    }

    constexpr field6 frobenius_map_one() const
    {
        return {
            c0.frobenius_map(),
            Fq6Params::frobenius_coeffs_c1_1 * c1.frobenius_map(),
            Fq6Params::frobenius_coeffs_c2_1 * c2.frobenius_map(),
        };
    }

    static constexpr field6 random_element(std::mt19937_64* engine = nullptr,
                                           std::uniform_int_distribution<uint64_t>* dist = nullptr)
    {
        return {
            base_field::random_element(engine, dist),
            base_field::random_element(engine, dist),
            base_field::random_element(engine, dist),
        };
    }

    constexpr field6 to_montgomery_form() const
    {
        return {
            c0.to_montgomery_form(),
            c1.to_montgomery_form(),
            c2.to_montgomery_form(),
        };
    }

    constexpr field6 from_montgomery_form() const
    {
        return {
            c0.from_montgomery_form(),
            c1.from_montgomery_form(),
            c2.from_montgomery_form(),
        };
    }

    constexpr bool is_zero() const { return c0.is_zero() && c1.is_zero() && c2.is_zero(); }

    constexpr bool operator==(const field6& other) const { return c0 == other.c0 && c1 == other.c1 && c2 == other.c2; }
};
} // namespace barretenberg
