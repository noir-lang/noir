#pragma once

#include "affine_element.hpp"
#include "wnaf.hpp"
#include <array>
#include <vector>
#include <common/inline.hpp>
#include <numeric/random/engine.hpp>
#include <numeric/uint256/uint256.hpp>
#include <random>

namespace barretenberg {
namespace group_elements {

template <class Fq, class Fr, class Params> class alignas(32) element {
  public:
    static constexpr Fq curve_b = Params::b;

    element() noexcept {}

    constexpr element(const Fq& a, const Fq& b, const Fq& c) noexcept;
    constexpr element(const element& other) noexcept;
    constexpr element(element&& other) noexcept;
    constexpr element(const affine_element<Fq, Fr, Params>& other) noexcept;

    constexpr element& operator=(const element& other) noexcept;
    constexpr element& operator=(element&& other) noexcept;

    constexpr operator affine_element<Fq, Fr, Params>() const noexcept;

    static element random_element(numeric::random::Engine* engine = nullptr) noexcept;

    constexpr element dbl() const noexcept;
    constexpr void self_dbl() noexcept;
    constexpr void self_mixed_add_or_sub(const affine_element<Fq, Fr, Params>& other,
                                                      const uint64_t predicate) noexcept;

    constexpr element operator+(const element& other) const noexcept;
    constexpr element operator+(const affine_element<Fq, Fr, Params>& other) const noexcept;
    constexpr element operator+=(const element& other) noexcept;
    constexpr element operator+=(const affine_element<Fq, Fr, Params>& other) noexcept;

    constexpr element operator-(const element& other) const noexcept;
    constexpr element operator-(const affine_element<Fq, Fr, Params>& other) const noexcept;
    constexpr element operator-() const noexcept;
    constexpr element operator-=(const element& other) noexcept;
    constexpr element operator-=(const affine_element<Fq, Fr, Params>& other) noexcept;

    friend constexpr element operator+(const affine_element<Fq, Fr, Params>& left, const element& right) noexcept
    {
        return right + left;
    }
    friend constexpr element operator-(const affine_element<Fq, Fr, Params>& left, const element& right) noexcept
    {
        return -right + left;
    }

    element operator*(const Fr& other) const noexcept;

    element operator*=(const Fr& other) noexcept;

    // constexpr Fr operator/(const element& other) noexcept {} TODO: this one seems harder than the others...

    constexpr element normalize() const noexcept;
    BBERG_INLINE constexpr element set_infinity() const noexcept;
    BBERG_INLINE constexpr void self_set_infinity() noexcept;
    BBERG_INLINE constexpr bool is_point_at_infinity() const noexcept;
    BBERG_INLINE constexpr bool on_curve() const noexcept;
    BBERG_INLINE constexpr bool operator==(const element& other) const noexcept;

    static void batch_normalize(element* elements, const size_t num_elements) noexcept;

    Fq x;
    Fq y;
    Fq z;

  private:
    element mul_without_endomorphism(const Fr& exponent) const noexcept;
    element mul_with_endomorphism(const Fr& exponent) const noexcept;

    template <typename = typename std::enable_if<Params::can_hash_to_curve>>
    static element random_coordinates_on_curve(numeric::random::Engine* engine = nullptr) noexcept
    {
        bool found_one = false;
        Fq yy;
        Fq x;
        Fq y;
        Fq t0;
        while (!found_one) {
            x = Fq::random_element(engine);
            yy = x.sqr() * x + Params::b;
            if constexpr (Params::has_a) {
                yy += (x * Params::a);
            }
            y = yy.sqrt();
            t0 = y.sqr();
            found_one = (yy == t0);
        }
        return { x, y, Fq::one() };
    }

    static void conditional_negate_affine(const affine_element<Fq, Fr, Params>& in,
                                          affine_element<Fq, Fr, Params>& out,
                                          const uint64_t predicate) noexcept;
};

template <class Fq, class Fr, class Params>
std::ostream& operator<<(std::ostream& os, element<Fq, Fr, Params> const& e) {
    return os << "x:" << e.x << " y:" << e.y << " z:" << e.z;
}

// constexpr element<Fq, Fr, Params>::one = element<Fq, Fr, Params>{ Params::one_x, Params::one_y, Fq::one() };
// constexpr element<Fq, Fr, Params>::point_at_infinity = one.set_infinity();
// constexpr element<Fq, Fr, Params>::curve_b = Params::b;
} // namespace group_elements
} // namespace barretenberg

#include "./element_impl.hpp"

template <class Fq, class Fr, class Params>
barretenberg::group_elements::affine_element<Fq, Fr, Params> operator*(
    const barretenberg::group_elements::affine_element<Fq, Fr, Params>& base, const Fr& exponent) noexcept
{
    return barretenberg::group_elements::affine_element<Fq, Fr, Params>(barretenberg::group_elements::element(base) *
                                                                        exponent);
}

template <class Fq, class Fr, class Params>
barretenberg::group_elements::affine_element<Fq, Fr, Params> operator*(
    const barretenberg::group_elements::element<Fq, Fr, Params>& base, const Fr& exponent) noexcept
{
    return (barretenberg::group_elements::element(base) * exponent);
}