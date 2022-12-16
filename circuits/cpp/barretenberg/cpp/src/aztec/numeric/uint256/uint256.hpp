/**
 * uint256_t
 * Copyright Aztec 2020
 *
 * An unsigned 256 bit integer type.
 *
 * Constructor and all methods are constexpr.
 * Ideally, uint256_t should be able to be treated like any other literal type.
 *
 * Not optimized for performance, this code doesn't touch any of our hot paths when constructing PLONK proofs.
 **/
#pragma once

#include <cstdint>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <common/serialize.hpp>
#include "../uint128/uint128.hpp"

namespace numeric {

class alignas(32) uint256_t {
  public:
    constexpr uint256_t(const uint64_t a = 0)
        : data{ a, 0, 0, 0 }
    {}

    constexpr uint256_t(const uint64_t a, const uint64_t b, const uint64_t c, const uint64_t d)
        : data{ a, b, c, d }
    {}

    constexpr uint256_t(const uint256_t& other)
        : data{ other.data[0], other.data[1], other.data[2], other.data[3] }
    {}

    explicit uint256_t(std::string const& str)
    {
        for (int i = 0; i < 4; ++i) {
            std::stringstream ss;
            ss << std::hex << str.substr(size_t(i) * 16, 16);
            ss >> data[3 - i];
        }
    }

    static constexpr uint256_t from_uint128(const uint128_t a)
    {
        return uint256_t(static_cast<uint64_t>(a), static_cast<uint64_t>(a >> 64), 0, 0);
    }

    constexpr explicit operator uint128_t() { return (uint128_t(data[1]) << 64) + data[0]; }

    constexpr uint256_t& operator=(const uint256_t& other) = default;

    explicit constexpr operator bool() const { return static_cast<bool>(data[0]); };

    template <typename T> explicit constexpr operator T() const { return static_cast<T>(data[0]); };

    constexpr bool get_bit(const uint64_t bit_index) const;
    constexpr uint64_t get_msb() const;

    constexpr uint256_t slice(const uint64_t start, const uint64_t end) const;
    constexpr uint256_t pow(const uint256_t& exponent) const;

    constexpr uint256_t operator+(const uint256_t& other) const;
    constexpr uint256_t operator-(const uint256_t& other) const;
    constexpr uint256_t operator-() const;

    constexpr uint256_t operator*(const uint256_t& other) const;
    constexpr uint256_t operator/(const uint256_t& other) const;
    constexpr uint256_t operator%(const uint256_t& other) const;

    constexpr uint256_t operator>>(const uint256_t& other) const;
    constexpr uint256_t operator<<(const uint256_t& other) const;

    constexpr uint256_t operator&(const uint256_t& other) const;
    constexpr uint256_t operator^(const uint256_t& other) const;
    constexpr uint256_t operator|(const uint256_t& other) const;
    constexpr uint256_t operator~() const;

    constexpr bool operator==(const uint256_t& other) const;
    constexpr bool operator!=(const uint256_t& other) const;
    constexpr bool operator!() const;

    constexpr bool operator>(const uint256_t& other) const;
    constexpr bool operator<(const uint256_t& other) const;
    constexpr bool operator>=(const uint256_t& other) const;
    constexpr bool operator<=(const uint256_t& other) const;

    static constexpr size_t length() { return 256; }

    constexpr uint256_t& operator+=(const uint256_t& other)
    {
        *this = *this + other;
        return *this;
    };
    constexpr uint256_t& operator-=(const uint256_t& other)
    {
        *this = *this - other;
        return *this;
    };
    constexpr uint256_t& operator*=(const uint256_t& other)
    {
        *this = *this * other;
        return *this;
    };
    constexpr uint256_t& operator/=(const uint256_t& other)
    {
        *this = *this / other;
        return *this;
    };
    constexpr uint256_t& operator%=(const uint256_t& other)
    {
        *this = *this % other;
        return *this;
    };

    constexpr uint256_t& operator++()
    {
        *this += uint256_t(1);
        return *this;
    };
    constexpr uint256_t& operator--()
    {
        *this -= uint256_t(1);
        return *this;
    };

    constexpr uint256_t& operator&=(const uint256_t& other)
    {
        *this = *this & other;
        return *this;
    };
    constexpr uint256_t& operator^=(const uint256_t& other)
    {
        *this = *this ^ other;
        return *this;
    };
    constexpr uint256_t& operator|=(const uint256_t& other)
    {
        *this = *this | other;
        return *this;
    };

    constexpr uint256_t& operator>>=(const uint256_t& other)
    {
        *this = *this >> other;
        return *this;
    };
    constexpr uint256_t& operator<<=(const uint256_t& other)
    {
        *this = *this << other;
        return *this;
    };

    constexpr std::pair<uint256_t, uint256_t> mul_extended(const uint256_t& other) const;

    uint64_t data[4];

    constexpr std::pair<uint256_t, uint256_t> divmod(const uint256_t& b) const;

  private:
    constexpr std::pair<uint64_t, uint64_t> mul_wide(const uint64_t a, const uint64_t b) const;
    constexpr std::pair<uint64_t, uint64_t> addc(const uint64_t a, const uint64_t b, const uint64_t carry_in) const;
    constexpr uint64_t addc_discard_hi(const uint64_t a, const uint64_t b, const uint64_t carry_in) const;
    constexpr uint64_t sbb_discard_hi(const uint64_t a, const uint64_t b, const uint64_t borrow_in) const;

    constexpr std::pair<uint64_t, uint64_t> sbb(const uint64_t a, const uint64_t b, const uint64_t borrow_in) const;
    constexpr uint64_t mac_discard_hi(const uint64_t a,
                                      const uint64_t b,
                                      const uint64_t c,
                                      const uint64_t carry_in) const;
    constexpr std::pair<uint64_t, uint64_t> mac(const uint64_t a,
                                                const uint64_t b,
                                                const uint64_t c,
                                                const uint64_t carry_in) const;
};

inline std::ostream& operator<<(std::ostream& os, uint256_t const& a)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << "0x" << std::setfill('0') << std::setw(16) << a.data[3] << std::setw(16) << a.data[2]
       << std::setw(16) << a.data[1] << std::setw(16) << a.data[0];
    os.flags(f);
    return os;
}

template <typename B> inline void read(B& it, uint256_t& value)
{
    using serialize::read;
    uint64_t a, b, c, d;
    read(it, d);
    read(it, c);
    read(it, b);
    read(it, a);
    value = uint256_t(a, b, c, d);
}

template <typename B> inline void write(B& it, uint256_t const& value)
{
    using serialize::write;
    write(it, value.data[3]);
    write(it, value.data[2]);
    write(it, value.data[1]);
    write(it, value.data[0]);
}

} // namespace numeric

#include "./uint256_impl.hpp"

using numeric::uint256_t;