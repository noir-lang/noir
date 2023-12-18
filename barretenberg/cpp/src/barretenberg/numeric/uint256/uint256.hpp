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

#include "../uint128/uint128.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include <concepts>
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <sstream>

namespace numeric {

class alignas(32) uint256_t {
  public:
    constexpr uint256_t(const uint64_t a = 0) noexcept
        : data{ a, 0, 0, 0 }
    {}

    constexpr uint256_t(const uint64_t a, const uint64_t b, const uint64_t c, const uint64_t d) noexcept
        : data{ a, b, c, d }
    {}

    constexpr uint256_t(const uint256_t& other) noexcept
        : data{ other.data[0], other.data[1], other.data[2], other.data[3] }
    {}
    constexpr uint256_t(uint256_t&& other) noexcept = default;

    explicit constexpr uint256_t(std::string input) noexcept
    {
        /* Quick and dirty conversion from a single character to its hex equivelent */
        constexpr auto HexCharToInt = [](uint8_t Input) {
            bool valid =
                (Input >= 'a' && Input <= 'f') || (Input >= 'A' && Input <= 'F') || (Input >= '0' && Input <= '9');
            if (!valid) {
                throw_or_abort("Error, uint256 constructed from string_view with invalid hex parameter");
            }
            uint8_t res =
                ((Input >= 'a') && (Input <= 'f'))   ? (Input - (static_cast<uint8_t>('a') - static_cast<uint8_t>(10)))
                : ((Input >= 'A') && (Input <= 'F')) ? (Input - (static_cast<uint8_t>('A') - static_cast<uint8_t>(10)))
                : ((Input >= '0') && (Input <= '9')) ? (Input - static_cast<uint8_t>('0'))
                                                     : 0;
            return res;
        };

        std::array<uint64_t, 4> limbs{ 0, 0, 0, 0 };
        size_t start_index = 0;
        if (input.size() == 66 && input[0] == '0' && input[1] == 'x') {
            start_index = 2;
        } else if (input.size() != 64) {
            throw_or_abort("Error, uint256 constructed from string_view with invalid length");
        }
        for (size_t j = 0; j < 4; ++j) {

            const size_t limb_index = start_index + j * 16;
            for (size_t i = 0; i < 8; ++i) {
                const size_t byte_index = limb_index + (i * 2);
                uint8_t nibble_hi = HexCharToInt(static_cast<uint8_t>(input[byte_index]));
                uint8_t nibble_lo = HexCharToInt(static_cast<uint8_t>(input[byte_index + 1]));
                uint8_t byte = static_cast<uint8_t>((nibble_hi * 16) + nibble_lo);
                limbs[j] <<= 8;
                limbs[j] += byte;
            }
        }
        data[0] = limbs[3];
        data[1] = limbs[2];
        data[2] = limbs[1];
        data[3] = limbs[0];
    }

    static constexpr uint256_t from_uint128(const uint128_t a) noexcept
    {
        return { static_cast<uint64_t>(a), static_cast<uint64_t>(a >> 64), 0, 0 };
    }

    constexpr explicit operator uint128_t() { return (static_cast<uint128_t>(data[1]) << 64) + data[0]; }

    constexpr uint256_t& operator=(const uint256_t& other) noexcept = default;
    constexpr uint256_t& operator=(uint256_t&& other) noexcept = default;
    constexpr ~uint256_t() noexcept = default;

    explicit constexpr operator bool() const { return static_cast<bool>(data[0]); };

    template <std::integral T> explicit constexpr operator T() const { return static_cast<T>(data[0]); };

    [[nodiscard]] constexpr bool get_bit(uint64_t bit_index) const;
    [[nodiscard]] constexpr uint64_t get_msb() const;

    [[nodiscard]] constexpr uint256_t slice(uint64_t start, uint64_t end) const;
    [[nodiscard]] constexpr uint256_t pow(const uint256_t& exponent) const;

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

    [[nodiscard]] constexpr std::pair<uint256_t, uint256_t> mul_extended(const uint256_t& other) const;

    uint64_t data[4]; // NOLINT

    [[nodiscard]] constexpr std::pair<uint256_t, uint256_t> divmod(const uint256_t& b) const;

  private:
    [[nodiscard]] static constexpr std::pair<uint64_t, uint64_t> mul_wide(uint64_t a, uint64_t b);
    [[nodiscard]] static constexpr std::pair<uint64_t, uint64_t> addc(uint64_t a, uint64_t b, uint64_t carry_in);
    [[nodiscard]] static constexpr uint64_t addc_discard_hi(uint64_t a, uint64_t b, uint64_t carry_in);
    [[nodiscard]] static constexpr uint64_t sbb_discard_hi(uint64_t a, uint64_t b, uint64_t borrow_in);
    [[nodiscard]] static constexpr std::pair<uint64_t, uint64_t> sbb(uint64_t a, uint64_t b, uint64_t borrow_in);
    [[nodiscard]] static constexpr uint64_t mac_discard_hi(uint64_t a, uint64_t b, uint64_t c, uint64_t carry_in);
    [[nodiscard]] static constexpr std::pair<uint64_t, uint64_t> mac(uint64_t a,
                                                                     uint64_t b,
                                                                     uint64_t c,
                                                                     uint64_t carry_in);
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
    uint64_t a = 0;
    uint64_t b = 0;
    uint64_t c = 0;
    uint64_t d = 0;
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

// disable linter errors; we want to expose a global uint256_t type to mimic uint64_t, uint32_t etc
// NOLINTNEXTLINE(tidymisc-unused-using-decls, google-global-names-in-headers, misc-unused-using-decls)
using numeric::uint256_t;
