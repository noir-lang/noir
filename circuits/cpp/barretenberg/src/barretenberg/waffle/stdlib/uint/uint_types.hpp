#pragma once
#include "./uint.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer, typename Native> class uintW : public uint<Composer> {
  public:
    uintW()
        : uint<Composer>(sizeof(Native) * 8, static_cast<uint64_t>(0))
    {}

    uintW(Native other)
        : uint<Composer>(sizeof(Native) * 8, other)
    {}

    uintW(Composer* parent_context)
        : uint<Composer>(sizeof(Native) * 8, parent_context)
    {}

    uintW(const witness_t<Composer>& value)
        : uint<Composer>(sizeof(Native) * 8, value)
    {}

    uintW(Composer* parent_context, const Native value)
        : uint<Composer>(sizeof(Native) * 8, parent_context, value)
    {}

    uintW(Composer* parent_context, const std::array<bool_t<Composer>, sizeof(Native) * 8>& wires)
        : uint<Composer>(parent_context, std::vector<bool_t<Composer>>(wires.begin(), wires.end()))
    {}

    uintW(const field_t<Composer>& other)
        : uint<Composer>(sizeof(Native) * 8, other)
    {}

    uintW(const uint<Composer>& other)
        : uint<Composer>(other)
    {}

    uintW(const byte_array<Composer>& other)
        : uint<Composer>(other)
    {}

    uintW& operator=(const uintW& other)
    {
        uint<Composer>::operator=(other);
        return *this;
    }

    uintW operator+(const uintW& other) { return uint<Composer>::operator+(other); }
    uintW operator-(const uintW& other) { return uint<Composer>::operator-(other); };
    uintW operator*(const uintW& other) { return uint<Composer>::operator*(other); };
    uintW operator/(const uintW& other) { return uint<Composer>::operator/(other); };
    uintW operator%(const uintW& other) { return uint<Composer>::operator%(other); };
    uintW operator&(const uintW& other) { return uint<Composer>::operator&(other); };
    uintW operator|(const uintW& other) { return uint<Composer>::operator|(other); };
    uintW operator^(const uintW& other) { return uint<Composer>::operator^(other); };
    uintW operator~() { return uint<Composer>::operator~(); };

    uintW operator>>(const uint32_t const_shift) { return uint<Composer>::operator>>(const_shift); };
    uintW operator<<(const uint32_t const_shift) { return uint<Composer>::operator<<(const_shift); };

    uintW ror(const uint32_t const_rotation) { return uint<Composer>::ror(const_rotation); };
    uintW rol(const uint32_t const_rotation) { return uint<Composer>::rol(const_rotation); };

    /*
        uint32 operator++();
        uint32 operator--();
        uint32 operator+=(const uint32& other) { *this = operator+(other); };
        uint32 operator-=(const uint32& other) { *this = operator-(other); };
        uint32 operator*=(const uint32& other) { *this = operator*(other); };
        uint32 operator/=(const uint32& other) { *this = operator/(other); };
        uint32 operator%=(const uint32& other) { *this = operator%(other); };

        uint32 operator&=(const uint32& other) { *this = operator&(other); };
        uint32 operator^=(const uint32& other) { *this = operator^(other); };
        uint32 operator|=(const uint32& other) { *this = operator|(other); };

        uint32 operator>>=(const uint32& other) { *this = operator>>(other); };
        uint32 operator<<=(const uint32& other) { *this = operator<<(other); };
    */
    Native get_value() const { return static_cast<Native>(uint<Composer>::get_value()); }
};

template <typename Composer> using uint64 = uintW<Composer, uint64_t>;

template <typename Composer> using uint32 = uintW<Composer, uint32_t>;

template <typename Composer> using uint16 = uintW<Composer, uint16_t>;

template <typename Composer> using uint8 = uintW<Composer, uint8_t>;

} // namespace stdlib
} // namespace plonk
