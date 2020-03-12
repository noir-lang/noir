/**
 * uint512_t
 * Copyright Aztec 2020
 *
 * An unsigned 512 bit integer type.
 *
 * Constructor and all methods are constexpr.
 * Ideally, uint512_t should be able to be treated like any other literal type.
 *
 * Not optimized for performance, this code doesn't touch any of our hot paths when constructing PLONK proofs
 **/
#pragma once

#include <cstdint>
#include <iomanip>
#include <iostream>

#include "../uint256/uint256.hpp"

class uint512_t {
  public:
    constexpr uint512_t(const uint64_t data = 0)
        : lo{ data, 0, 0, 0 }
        , hi(0)
    {}

    constexpr uint512_t(const uint256_t input_lo)
        : lo(input_lo)
        , hi(0)
    {}

    constexpr uint512_t(const uint256_t input_lo, const uint256_t input_hi)
        : lo(input_lo)
        , hi(input_hi)
    {}

    constexpr uint512_t(const uint512_t& other)
        : lo(other.lo)
        , hi(other.hi)
    {}

    constexpr uint512_t& operator=(const uint512_t& other) = default;

    explicit constexpr operator bool() const { return static_cast<bool>(lo.data[0]); };

    template<typename T>
    explicit constexpr operator T() const { return static_cast<T>(lo.data[0]); };

    constexpr bool get_bit(const uint64_t bit_index) const;
    constexpr uint64_t get_msb() const;

    constexpr uint512_t operator+(const uint512_t& other) const;
    constexpr uint512_t operator-(const uint512_t& other) const;
    constexpr uint512_t operator-() const;

    constexpr uint512_t operator*(const uint512_t& other) const;
    constexpr uint512_t operator/(const uint512_t& other) const;
    constexpr uint512_t operator%(const uint512_t& other) const;

    constexpr uint512_t operator>>(const uint256_t& other) const;
    constexpr uint512_t operator<<(const uint256_t& other) const;

    constexpr uint512_t operator&(const uint512_t& other) const;
    constexpr uint512_t operator^(const uint512_t& other) const;
    constexpr uint512_t operator|(const uint512_t& other) const;
    constexpr uint512_t operator~() const;

    constexpr bool operator==(const uint512_t& other) const;
    constexpr bool operator!=(const uint512_t& other) const;
    constexpr bool operator!() const;

    constexpr bool operator>(const uint512_t& other) const;
    constexpr bool operator<(const uint512_t& other) const;
    constexpr bool operator>=(const uint512_t& other) const;
    constexpr bool operator<=(const uint512_t& other) const;

    constexpr uint512_t& operator+=(const uint512_t& other)
    {
        *this = *this + other;
        return *this;
    };
    constexpr uint512_t& operator-=(const uint512_t& other)
    {
        *this = *this - other;
        return *this;
    };
    constexpr uint512_t& operator*=(const uint512_t& other)
    {
        *this = *this * other;
        return *this;
    };
    constexpr uint512_t& operator/=(const uint512_t& other)
    {
        *this = *this / other;
        return *this;
    };
    constexpr uint512_t& operator%=(const uint512_t& other)
    {
        *this = *this % other;
        return *this;
    };

    constexpr uint512_t& operator++()
    {
        *this += uint512_t(1);
        return *this;
    };
    constexpr uint512_t& operator--()
    {
        *this -= uint512_t(1);
        return *this;
    };

    constexpr uint512_t& operator&=(const uint512_t& other)
    {
        *this = *this & other;
        return *this;
    };
    constexpr uint512_t& operator^=(const uint512_t& other)
    {
        *this = *this ^ other;
        return *this;
    };
    constexpr uint512_t& operator|=(const uint512_t& other)
    {
        *this = *this | other;
        return *this;
    };

    constexpr uint512_t& operator>>=(const uint256_t& other)
    {
        *this = *this >> other;
        return *this;
    };
    constexpr uint512_t& operator<<=(const uint256_t& other)
    {
        *this = *this << other;
        return *this;
    };

    constexpr uint512_t invmod(const uint512_t& modulus) const;

    uint256_t lo;
    uint256_t hi;

  private:
    constexpr std::pair<uint512_t, uint512_t> divmod(const uint512_t& b) const;
};

#include "./uint512_impl.hpp"

inline std::ostream& operator<<(std::ostream& os, uint512_t const& a)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << "0x" << std::setfill('0') << std::setw(16) << a.hi.data[3] << std::setw(16) << a.hi.data[2]
       << std::setw(16) << a.hi.data[1] << std::setw(16) << a.hi.data[0] << std::setw(16) << a.lo.data[3]
       << std::setw(16) << a.lo.data[2] << std::setw(16) << a.lo.data[1] << std::setw(16) << a.lo.data[0];
    os.flags(f);
    return os;
}