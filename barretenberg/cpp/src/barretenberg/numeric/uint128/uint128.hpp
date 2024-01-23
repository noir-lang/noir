#pragma once
#include <cstdint>
#include <iomanip>
#include <ostream>

#ifdef __i386__
#include "barretenberg/common/serialize.hpp"
#include <concepts>

namespace bb::numeric {

class alignas(32) uint128_t {
  public:
    uint32_t data[4]; // NOLINT

    constexpr uint128_t(const uint64_t a = 0)
        : data{ static_cast<uint32_t>(a), static_cast<uint32_t>(a >> 32), 0, 0 }
    {}

    constexpr uint128_t(const uint32_t a, const uint32_t b, const uint32_t c, const uint32_t d)
        : data{ a, b, c, d }
    {}

    constexpr uint128_t(const uint128_t& other)
        : data{ other.data[0], other.data[1], other.data[2], other.data[3] }
    {}
    constexpr uint128_t(uint128_t&& other) = default;

    static constexpr uint128_t from_uint64(const uint64_t a)
    {
        return { static_cast<uint32_t>(a), static_cast<uint32_t>(a >> 32), 0, 0 };
    }

    constexpr explicit operator uint64_t() { return (static_cast<uint64_t>(data[1]) << 32) + data[0]; }

    constexpr uint128_t& operator=(const uint128_t& other) = default;
    constexpr uint128_t& operator=(uint128_t&& other) = default;
    constexpr ~uint128_t() = default;
    explicit constexpr operator bool() const { return static_cast<bool>(data[0]); };

    template <std::integral T> explicit constexpr operator T() const { return static_cast<T>(data[0]); };

    [[nodiscard]] constexpr bool get_bit(uint64_t bit_index) const;
    [[nodiscard]] constexpr uint64_t get_msb() const;

    [[nodiscard]] constexpr uint128_t slice(uint64_t start, uint64_t end) const;
    [[nodiscard]] constexpr uint128_t pow(const uint128_t& exponent) const;

    constexpr uint128_t operator+(const uint128_t& other) const;
    constexpr uint128_t operator-(const uint128_t& other) const;
    constexpr uint128_t operator-() const;

    constexpr uint128_t operator*(const uint128_t& other) const;
    constexpr uint128_t operator/(const uint128_t& other) const;
    constexpr uint128_t operator%(const uint128_t& other) const;

    constexpr uint128_t operator>>(const uint128_t& other) const;
    constexpr uint128_t operator<<(const uint128_t& other) const;

    constexpr uint128_t operator&(const uint128_t& other) const;
    constexpr uint128_t operator^(const uint128_t& other) const;
    constexpr uint128_t operator|(const uint128_t& other) const;
    constexpr uint128_t operator~() const;

    constexpr bool operator==(const uint128_t& other) const;
    constexpr bool operator!=(const uint128_t& other) const;
    constexpr bool operator!() const;

    constexpr bool operator>(const uint128_t& other) const;
    constexpr bool operator<(const uint128_t& other) const;
    constexpr bool operator>=(const uint128_t& other) const;
    constexpr bool operator<=(const uint128_t& other) const;

    static constexpr size_t length() { return 128; }

    constexpr uint128_t& operator+=(const uint128_t& other)
    {
        *this = *this + other;
        return *this;
    };
    constexpr uint128_t& operator-=(const uint128_t& other)
    {
        *this = *this - other;
        return *this;
    };
    constexpr uint128_t& operator*=(const uint128_t& other)
    {
        *this = *this * other;
        return *this;
    };
    constexpr uint128_t& operator/=(const uint128_t& other)
    {
        *this = *this / other;
        return *this;
    };
    constexpr uint128_t& operator%=(const uint128_t& other)
    {
        *this = *this % other;
        return *this;
    };

    constexpr uint128_t& operator++()
    {
        *this += uint128_t(1);
        return *this;
    };
    constexpr uint128_t& operator--()
    {
        *this -= uint128_t(1);
        return *this;
    };

    constexpr uint128_t& operator&=(const uint128_t& other)
    {
        *this = *this & other;
        return *this;
    };
    constexpr uint128_t& operator^=(const uint128_t& other)
    {
        *this = *this ^ other;
        return *this;
    };
    constexpr uint128_t& operator|=(const uint128_t& other)
    {
        *this = *this | other;
        return *this;
    };

    constexpr uint128_t& operator>>=(const uint128_t& other)
    {
        *this = *this >> other;
        return *this;
    };
    constexpr uint128_t& operator<<=(const uint128_t& other)
    {
        *this = *this << other;
        return *this;
    };

    [[nodiscard]] constexpr std::pair<uint128_t, uint128_t> mul_extended(const uint128_t& other) const;

    [[nodiscard]] constexpr std::pair<uint128_t, uint128_t> divmod(const uint128_t& b) const;

  private:
    [[nodiscard]] static constexpr std::pair<uint32_t, uint32_t> mul_wide(uint32_t a, uint32_t b);
    [[nodiscard]] static constexpr std::pair<uint32_t, uint32_t> addc(uint32_t a, uint32_t b, uint32_t carry_in);
    [[nodiscard]] static constexpr uint32_t addc_discard_hi(uint32_t a, uint32_t b, uint32_t carry_in);
    [[nodiscard]] static constexpr uint32_t sbb_discard_hi(uint32_t a, uint32_t b, uint32_t borrow_in);

    [[nodiscard]] static constexpr std::pair<uint32_t, uint32_t> sbb(uint32_t a, uint32_t b, uint32_t borrow_in);
    [[nodiscard]] static constexpr uint32_t mac_discard_hi(uint32_t a, uint32_t b, uint32_t c, uint32_t carry_in);
    [[nodiscard]] static constexpr std::pair<uint32_t, uint32_t> mac(uint32_t a,
                                                                     uint32_t b,
                                                                     uint32_t c,
                                                                     uint32_t carry_in);
};

inline std::ostream& operator<<(std::ostream& os, uint128_t const& a)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << "0x" << std::setfill('0') << std::setw(8) << a.data[3] << std::setw(8) << a.data[2]
       << std::setw(8) << a.data[1] << std::setw(8) << a.data[0];
    os.flags(f);
    return os;
}

template <typename B> inline void read(B& it, uint128_t& value)
{
    using serialize::read;
    uint32_t a = 0;
    uint32_t b = 0;
    uint32_t c = 0;
    uint32_t d = 0;
    read(it, d);
    read(it, c);
    read(it, b);
    read(it, a);
    value = uint128_t(a, b, c, d);
}

template <typename B> inline void write(B& it, uint128_t const& value)
{
    using serialize::write;
    write(it, value.data[3]);
    write(it, value.data[2]);
    write(it, value.data[1]);
    write(it, value.data[0]);
}

} // namespace bb::numeric

#include "./uint128_impl.hpp"

// disable linter errors; we want to expose a global uint128_t type to mimic uint64_t, uint32_t etc
// NOLINTNEXTLINE(tidymisc-unused-using-decls, google-global-names-in-headers, misc-unused-using-decls)
using numeric::uint128_t;
#else
__extension__ using uint128_t = unsigned __int128;

namespace std {
// can ignore linter error for streaming operations, we need to add to std namespace to support printing this type!
// NOLINTNEXTLINE(cert-dcl58-cpp)
inline std::ostream& operator<<(std::ostream& os, uint128_t const& a)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << "0x" << std::setfill('0') << std::setw(16) << static_cast<uint64_t>(a >> 64) << std::setw(16)
       << static_cast<uint64_t>(a);
    os.flags(f);
    return os;
}
} // namespace std
#endif
