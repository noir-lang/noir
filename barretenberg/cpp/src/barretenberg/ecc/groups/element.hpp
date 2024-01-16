#pragma once

#include "affine_element.hpp"
#include "barretenberg/common/compiler_hints.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "wnaf.hpp"
#include <array>
#include <random>
#include <vector>

namespace bb::group_elements {

/**
 * @brief element class. Implements ecc group arithmetic using Jacobian coordinates
 * See https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
 *
 * Note: Currently subgroup checks are NOT IMPLEMENTED
 * Our current Plonk implementation uses G1 points that have a cofactor of 1.
 * All G2 points are precomputed (generator [1]_2 and trusted setup point [x]_2).
 * Explicitly assume precomputed points are valid members of the prime-order subgroup for G2.
 * @tparam Fq prime field the curve is defined over
 * @tparam Fr prime field whose characteristic equals the size of the prime-order elliptic curve subgroup
 * @tparam Params curve parameters
 */
template <class Fq, class Fr, class Params> class alignas(32) element {
  public:
    static constexpr Fq curve_b = Params::b;

    element() noexcept = default;

    constexpr element(const Fq& a, const Fq& b, const Fq& c) noexcept;
    constexpr element(const element& other) noexcept;
    constexpr element(element&& other) noexcept;
    constexpr element(const affine_element<Fq, Fr, Params>& other) noexcept;
    constexpr ~element() noexcept = default;

    static constexpr element one() noexcept { return { Params::one_x, Params::one_y, Fq::one() }; };
    static constexpr element zero() noexcept
    {
        element zero;
        zero.self_set_infinity();
        return zero;
    };

    constexpr element& operator=(const element& other) noexcept;
    constexpr element& operator=(element&& other) noexcept;

    constexpr operator affine_element<Fq, Fr, Params>() const noexcept;

    static element random_element(numeric::random::Engine* engine = nullptr) noexcept;

    constexpr element dbl() const noexcept;
    constexpr void self_dbl() noexcept;
    constexpr void self_mixed_add_or_sub(const affine_element<Fq, Fr, Params>& other, uint64_t predicate) noexcept;

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

    element operator*(const Fr& exponent) const noexcept;
    element operator*=(const Fr& exponent) noexcept;

    // If you end up implementing this, congrats, you've solved the DL problem!
    // P.S. This is a joke, don't even attempt! 😂
    // constexpr Fr operator/(const element& other) noexcept {}

    constexpr element normalize() const noexcept;
    static element infinity();
    BBERG_INLINE constexpr element set_infinity() const noexcept;
    BBERG_INLINE constexpr void self_set_infinity() noexcept;
    [[nodiscard]] BBERG_INLINE constexpr bool is_point_at_infinity() const noexcept;
    [[nodiscard]] BBERG_INLINE constexpr bool on_curve() const noexcept;
    BBERG_INLINE constexpr bool operator==(const element& other) const noexcept;

    static void batch_normalize(element* elements, size_t num_elements) noexcept;
    static void batch_affine_add(const std::span<affine_element<Fq, Fr, Params>>& first_group,
                                 const std::span<affine_element<Fq, Fr, Params>>& second_group,
                                 const std::span<affine_element<Fq, Fr, Params>>& results) noexcept;
    static std::vector<affine_element<Fq, Fr, Params>> batch_mul_with_endomorphism(
        const std::span<affine_element<Fq, Fr, Params>>& points, const Fr& exponent) noexcept;

    Fq x;
    Fq y;
    Fq z;

  private:
    element mul_without_endomorphism(const Fr& exponent) const noexcept;
    element mul_with_endomorphism(const Fr& exponent) const noexcept;

    template <typename = typename std::enable_if<Params::can_hash_to_curve>>
    static element random_coordinates_on_curve(numeric::random::Engine* engine = nullptr) noexcept;
    // {
    //     bool found_one = false;
    //     Fq yy;
    //     Fq x;
    //     Fq y;
    //     Fq t0;
    //     while (!found_one) {
    //         x = Fq::random_element(engine);
    //         yy = x.sqr() * x + Params::b;
    //         if constexpr (Params::has_a) {
    //             yy += (x * Params::a);
    //         }
    //         y = yy.sqrt();
    //         t0 = y.sqr();
    //         found_one = (yy == t0);
    //     }
    //     return { x, y, Fq::one() };
    // }
    // for serialization: update with new fields
    MSGPACK_FIELDS(x, y, z);

    static void conditional_negate_affine(const affine_element<Fq, Fr, Params>& in,
                                          affine_element<Fq, Fr, Params>& out,
                                          uint64_t predicate) noexcept;

    friend std::ostream& operator<<(std::ostream& os, const element& a)
    {
        os << "{ " << a.x << ", " << a.y << ", " << a.z << " }";
        return os;
    }
};

template <class Fq, class Fr, class Params> std::ostream& operator<<(std::ostream& os, element<Fq, Fr, Params> const& e)
{
    return os << "x:" << e.x << " y:" << e.y << " z:" << e.z;
}

// constexpr element<Fq, Fr, Params>::one = element<Fq, Fr, Params>{ Params::one_x, Params::one_y, Fq::one() };
// constexpr element<Fq, Fr, Params>::point_at_infinity = one.set_infinity();
// constexpr element<Fq, Fr, Params>::curve_b = Params::b;
} // namespace bb::group_elements

#include "./element_impl.hpp"

template <class Fq, class Fr, class Params>
bb::group_elements::affine_element<Fq, Fr, Params> operator*(
    const bb::group_elements::affine_element<Fq, Fr, Params>& base, const Fr& exponent) noexcept
{
    return bb::group_elements::affine_element<Fq, Fr, Params>(bb::group_elements::element(base) * exponent);
}

template <class Fq, class Fr, class Params>
bb::group_elements::affine_element<Fq, Fr, Params> operator*(const bb::group_elements::element<Fq, Fr, Params>& base,
                                                             const Fr& exponent) noexcept
{
    return (bb::group_elements::element(base) * exponent);
}
