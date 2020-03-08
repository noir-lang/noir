#pragma once
#include <crypto/keccak/keccak.hpp>

namespace barretenberg {
namespace group_elements {
template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(const Fq& a, const Fq& b) noexcept
    : x(a)
    , y(b)
{}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(const affine_element& other) noexcept
    : x(other.x)
    , y(other.y)
{}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(affine_element&& other) noexcept
    : x(other.x)
    , y(other.y)
{}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(const uint256_t& compressed) noexcept
{
    uint256_t x_coordinate = compressed;
    x_coordinate.data[3] = x_coordinate.data[3] & (~0x8000000000000000ULL);
    bool y_bit = compressed.get_bit(255);

    x = Fq(x_coordinate);
    y = (x.sqr() * x + T::b).sqrt();

    if (y.from_montgomery_form().get_bit(0) != y_bit) {
        y = -y;
    }
}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>& affine_element<Fq, Fr, T>::operator=(const affine_element& other) noexcept
{
    x = other.x;
    y = other.y;
    return *this;
}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>& affine_element<Fq, Fr, T>::operator=(affine_element&& other) noexcept
{
    x = other.x;
    y = other.y;
    return *this;
}

template <class Fq, class Fr, class T> constexpr affine_element<Fq, Fr, T>::operator uint256_t() const noexcept
{
    uint256_t out(x);
    if (y.from_montgomery_form().get_bit(0)) {
        out.data[3] = out.data[3] | 0x8000000000000000ULL;
    }
    return out;
}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::set_infinity() const noexcept
{
    affine_element result(*this);
    result.self_set_infinity();
    return result;
}

template <class Fq, class Fr, class T> constexpr void affine_element<Fq, Fr, T>::self_set_infinity() noexcept
{
    y.self_set_msb();
}

template <class Fq, class Fr, class T> constexpr bool affine_element<Fq, Fr, T>::is_point_at_infinity() const noexcept
{
    return (y.is_msb_set());
}

template <class Fq, class Fr, class T> constexpr bool affine_element<Fq, Fr, T>::on_curve() const noexcept
{
    if (is_point_at_infinity()) {
        return false;
    }
    Fq xxx = x.sqr() * x + T::b;
    Fq yy = y.sqr();
    return (xxx == yy);
}

template <class Fq, class Fr, class T>
constexpr bool affine_element<Fq, Fr, T>::operator==(const affine_element& other) const noexcept
{
    bool both_infinity = is_point_at_infinity() && other.is_point_at_infinity();
    return both_infinity || ((x == other.x) && (y == other.y));
}

template <class Fq, class Fr, class T>
affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::hash_to_curve(const uint64_t seed) noexcept
{
    static_assert(T::can_hash_to_curve == true);

    Fq input(seed, 0, 0, 0);
    keccak256 c = hash_field_element((uint64_t*)&input.data[0]);
    uint256_t compressed{ c.word64s[0], c.word64s[1], c.word64s[2], c.word64s[3] };
    return affine_element<Fq, Fr, T>(compressed);
}
} // namespace group_elements
} // namespace barretenberg