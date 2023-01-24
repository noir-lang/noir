#pragma once
#include <numeric/random/engine.hpp>

namespace barretenberg {
template <class base_field, class Params> struct alignas(32) field2 {
  public:
    constexpr field2(const base_field& a = base_field::zero(), const base_field& b = base_field::zero())
        : c0(a)
        , c1(b)
    {}

    constexpr field2(const field2& other)
        : c0(other.c0)
        , c1(other.c1)
    {}
    constexpr field2(field2&& other)
        : c0(other.c0)
        , c1(other.c1)
    {}

    constexpr field2& operator=(const field2& other)
    {
        c0 = other.c0;
        c1 = other.c1;
        return *this;
    }

    constexpr field2& operator=(field2&& other)
    {
        c0 = other.c0;
        c1 = other.c1;
        return *this;
    }

    base_field c0;
    base_field c1;

    static constexpr uint256_t modulus = base_field::modulus;

    static constexpr field2 zero() { return field2{ base_field::zero(), base_field::zero() }; }
    static constexpr field2 one() { return field2{ base_field::one(), base_field::zero() }; }
    static constexpr field2 twist_coeff_b() { return field2{ Params::twist_coeff_b_0, Params::twist_coeff_b_1 }; }
    static constexpr field2 twist_mul_by_q_x()
    {
        return field2{ Params::twist_mul_by_q_x_0, Params::twist_mul_by_q_x_1 };
    }
    static constexpr field2 twist_mul_by_q_y()
    {
        return field2{ Params::twist_mul_by_q_y_0, Params::twist_mul_by_q_y_1 };
    }
    static constexpr field2 beta() { return field2{ Params::twist_cube_root_0, Params::twist_cube_root_1 }; }

    constexpr field2 operator*(const field2& other) const noexcept;
    constexpr field2 operator+(const field2& other) const noexcept;
    constexpr field2 operator-(const field2& other) const noexcept;
    constexpr field2 operator-() const noexcept;
    constexpr field2 operator/(const field2& other) const noexcept;

    constexpr field2 operator*=(const field2& other) noexcept;
    constexpr field2 operator+=(const field2& other) noexcept;
    constexpr field2 operator-=(const field2& other) noexcept;
    constexpr field2 operator/=(const field2& other) noexcept;

    constexpr field2 mul_by_fq(const base_field& a) const noexcept
    {
        field2 r{ a * c0, a * c1 };
        return r;
    }

    // constexpr bool operator>(const field& other) const noexcept;
    // constexpr bool operator<(const field& other) const noexcept;
    constexpr bool operator==(const field2& other) const noexcept;
    constexpr bool operator!=(const field2& other) const noexcept { return !(*this == other); }
    constexpr field2 sqr() const noexcept;
    constexpr void self_sqr() noexcept;

    constexpr field2 pow(const uint256_t& exponent) const noexcept;
    constexpr field2 pow(const uint64_t exponent) const noexcept;

    constexpr field2 invert() const noexcept;

    constexpr void self_neg() noexcept;
    constexpr field2 to_montgomery_form() const noexcept;
    constexpr field2 from_montgomery_form() const noexcept;

    constexpr void self_to_montgomery_form() noexcept;
    constexpr void self_from_montgomery_form() noexcept;

    constexpr void self_conditional_negate(const uint64_t predicate) noexcept;

    constexpr field2 reduce_once() const noexcept;
    constexpr void self_reduce_once() noexcept;

    constexpr void self_set_msb() noexcept;
    constexpr bool is_msb_set() const noexcept;
    constexpr uint64_t is_msb_set_word() const noexcept;

    constexpr bool is_zero() const noexcept;

    constexpr field2 frobenius_map() const noexcept;
    constexpr void self_frobenius_map() noexcept;

    static field2 random_element(numeric::random::Engine* engine = nullptr);
    static void serialize_to_buffer(const field2& value, uint8_t* buffer)
    {
        base_field::serialize_to_buffer(value.c0, buffer);
        base_field::serialize_to_buffer(value.c1, buffer + sizeof(base_field));
    }

    static field2 serialize_from_buffer(uint8_t* buffer)
    {
        field2 result{ base_field::zero(), base_field::zero() };
        result.c0 = base_field::serialize_from_buffer(buffer);
        result.c1 = base_field::serialize_from_buffer(buffer + sizeof(base_field));

        return result;
    }

    friend std::ostream& operator<<(std::ostream& os, const field2& a)
    {
        os << a.c0 << " , " << a.c1;
        return os;
    }
};

} // namespace barretenberg

#include "field2_impl.hpp"