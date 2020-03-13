#pragma once

#include <stdint.h>

namespace barretenberg {
template <class base, class T> constexpr field2<base, T> field2<base, T>::operator*(const field2& other) const noexcept
{
    base t1 = c0 * other.c0;
    base t2 = c1 * other.c1;
    base t3 = c0 + c1;
    base t4 = other.c0 + other.c1;

    return { t1 - t2, t3 * t4 - (t1 + t2) };
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator+(const field2& other) const noexcept
{
    return { c0 + other.c0, c1 + other.c1 };
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator-(const field2& other) const noexcept
{
    return { c0 - other.c0, c1 - other.c1 };
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator-() const noexcept
{
    return { -c0, -c1 };
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator/(const field2& other) const noexcept
{
    return operator*(other.invert());
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator*=(const field2& other) noexcept
{
    *this = operator*(other);
    return *this;
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator+=(const field2& other) noexcept
{
    *this = operator+(other);
    return *this;
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator-=(const field2& other) noexcept
{
    *this = operator-(other);
    return *this;
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::operator/=(const field2& other) noexcept
{
    *this = operator/(other);
    return *this;
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::sqr() const noexcept
{
    base t1 = (c0 * c1);
    return { (c0 + c1) * (c0 - c1), t1 + t1 };
}

template <class base, class T> constexpr void field2<base, T>::self_sqr() noexcept
{
    *this = sqr();
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::to_montgomery_form() const noexcept
{
    return { c0.to_montgomery_form(), c1.to_montgomery_form() };
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::from_montgomery_form() const noexcept
{
    return { c0.from_montgomery_form(), c1.from_montgomery_form() };
}

template <class base, class T> constexpr void field2<base, T>::self_to_montgomery_form() noexcept
{
    c0.self_to_montgomery_form();
    c1.self_to_montgomery_form();
}

template <class base, class T> constexpr void field2<base, T>::self_from_montgomery_form() noexcept
{
    c0.self_from_montgomery_form();
    c1.self_from_montgomery_form();
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::reduce_once() const noexcept
{
    return *this;
    // return { c0.reduce_once(), c1.reduce_once() };
}

template <class base, class T> constexpr void field2<base, T>::self_reduce_once() noexcept
{
    // c0.self_reduce_once();
    // c1.self_reduce_once();
}

template <class base, class T> constexpr void field2<base, T>::self_neg() noexcept
{
    c0.self_neg();
    c1.self_neg();
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::invert() const noexcept
{
    base t3 = (c0.sqr() + c1.sqr()).invert();
    return { c0 * t3, -(c1 * t3) };
}

template <class base, class T>
constexpr void field2<base, T>::self_conditional_negate(const uint64_t predicate) noexcept
{
    *this = predicate ? -(*this) : *this;
}

template <class base, class T> constexpr void field2<base, T>::self_set_msb() noexcept
{
    c0.data[3] = 0ULL | (1ULL << 63ULL);
}

template <class base, class T> constexpr bool field2<base, T>::is_msb_set() const noexcept
{
    return (c0.data[3] >> 63ULL) == 1ULL;
}

template <class base, class T> constexpr uint64_t field2<base, T>::is_msb_set_word() const noexcept
{
    return (c0.data[3] >> 63ULL);
}

template <class base, class T> constexpr bool field2<base, T>::is_zero() const noexcept
{
    return (c0.is_zero() && c1.is_zero());
}

template <class base, class T> constexpr bool field2<base, T>::operator==(const field2& other) const noexcept
{
    return (c0 == other.c0) && (c1 == other.c1);
}

template <class base, class T> constexpr field2<base, T> field2<base, T>::frobenius_map() const noexcept
{
    return { c0, -c1 };
}

template <class base, class T> constexpr void field2<base, T>::self_frobenius_map() noexcept
{
    c1.self_neg();
}

template <class base, class T> field2<base, T> field2<base, T>::random_element(numeric::random::Engine* engine)
{
    return { base::random_element(engine), base::random_element(engine) };
}
} // namespace barretenberg