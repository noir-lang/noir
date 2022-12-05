#pragma once
#include <iomanip>
#include <ostream>

#ifdef __i386__
#include <cstdint>
#include <common/serialize.hpp>

namespace numeric {

class alignas(32) uint128_t {
  public:
    uint32_t data[4];

    constexpr uint128_t(const uint64_t a = 0)
        : data{ static_cast<uint32_t>(a), static_cast<uint32_t>(a >> 32), 0, 0 }
    {}

    constexpr uint128_t(const uint32_t a, const uint32_t b, const uint32_t c, const uint32_t d)
        : data{ a, b, c, d }
    {}

    constexpr uint128_t(const uint128_t& other)
        : data{ other.data[0], other.data[1], other.data[2], other.data[3] }
    {}

    static constexpr uint128_t from_uint64(const uint64_t a)
    {
        return uint128_t(static_cast<uint32_t>(a), static_cast<uint32_t>(a >> 32), 0, 0);
    }

    constexpr explicit operator uint64_t() { return (uint64_t(data[1]) << 32) + data[0]; }

    constexpr uint128_t& operator=(const uint128_t& other) = default;

    explicit constexpr operator bool() const { return static_cast<bool>(data[0]); };

    template <typename T> explicit constexpr operator T() const { return static_cast<T>(data[0]); };

    constexpr bool get_bit(const uint64_t bit_index) const;
    constexpr uint64_t get_msb() const;

    constexpr uint128_t slice(const uint64_t start, const uint64_t end) const;
    constexpr uint128_t pow(const uint128_t& exponent) const;

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

    constexpr std::pair<uint128_t, uint128_t> mul_extended(const uint128_t& other) const;

    constexpr std::pair<uint128_t, uint128_t> divmod(const uint128_t& b) const;

  private:
    constexpr std::pair<uint32_t, uint32_t> mul_wide(const uint32_t a, const uint32_t b) const;
    constexpr std::pair<uint32_t, uint32_t> addc(const uint32_t a, const uint32_t b, const uint32_t carry_in) const;
    constexpr uint32_t addc_discard_hi(const uint32_t a, const uint32_t b, const uint32_t carry_in) const;
    constexpr uint32_t sbb_discard_hi(const uint32_t a, const uint32_t b, const uint32_t borrow_in) const;

    constexpr std::pair<uint32_t, uint32_t> sbb(const uint32_t a, const uint32_t b, const uint32_t borrow_in) const;
    constexpr uint32_t mac_discard_hi(const uint32_t a,
                                      const uint32_t b,
                                      const uint32_t c,
                                      const uint32_t carry_in) const;
    constexpr std::pair<uint32_t, uint32_t> mac(const uint32_t a,
                                                const uint32_t b,
                                                const uint32_t c,
                                                const uint32_t carry_in) const;
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
    uint32_t a, b, c, d;
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

} // namespace numeric

#include "./uint128_impl.hpp"

using numeric::uint128_t;
#else
__extension__ using uint128_t = unsigned __int128;

namespace std {
inline std::ostream& operator<<(std::ostream& os, uint128_t const& a)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << "0x" << std::setfill('0') << std::setw(16) << (uint64_t)(a >> 64) << std::setw(16) << (uint64_t)a;
    os.flags(f);
    return os;
}
} // namespace std
#endif