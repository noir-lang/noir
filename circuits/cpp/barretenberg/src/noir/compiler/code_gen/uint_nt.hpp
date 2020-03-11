#pragma once
#include <numeric/uint256/uint256.hpp>
#include <stdlib/types/turbo.hpp>
#include <iomanip>

namespace noir {
namespace code_gen {

using namespace plonk::stdlib::types::turbo;

class uint_nt {
  public:
    uint_nt(size_t width, const uint64_t other)
        : __width(width)
        , uint8(uint256_t(other))
        , uint16(uint256_t(other))
        , uint32(uint256_t(other))
        , uint64(uint256_t(other))
    {}

    uint_nt(size_t width, const uint256_t other)
        : __width(width)
        , uint8(other)
        , uint16(other)
        , uint32(other)
        , uint64(other)
    {}


    uint_nt(uint64_t value)
        : __width(32)
        , uint8(uint8_t(value))
        , uint16(uint16_t(value))
        , uint32(uint32_t(value))
        , uint64(uint64_t(value))
    {}

    uint_nt(size_t width, Composer* parent_context)
        : __width(width)
        , uint8(parent_context, uint256_t(0))
        , uint16(parent_context, uint256_t(0))
        , uint32(parent_context, uint256_t(0))
        , uint64(parent_context, uint256_t(0))
    {}

    uint_nt(size_t width, const witness_ct& value)
        : __width(width)
    {
        switch (__width) {
        case 8: {
            uint8 = uint8_ct(value);
            break;
        }
        case 16: {
            uint16 = uint16_ct(value);
            break;
        }
        case 32: {
            uint32 = uint32_ct(value);
            break;
        }
        default: {
            uint64 = uint64_ct(value);
            break;
        }
        }
    }

    uint_nt(size_t width, Composer* parent_context, const uint64_t other)
        : __width(width)
        , uint8(parent_context, uint256_t(other))
        , uint16(parent_context, uint256_t(other))
        , uint32(parent_context, uint256_t(other))
        , uint64(parent_context, uint256_t(other))
    {}

    uint_nt(size_t width, const field_ct& value)
        : __width(width)
    {
        switch (__width) {
        case 8: {
            uint8 = uint8_ct(value);
            break;
        }
        case 16: {
            uint16 = uint16_ct(value);
            break;
        }
        case 32: {
            uint32 = uint32_ct(value);
            break;
        }
        default: {
            uint64 = uint64_ct(value);
            break;
        }
        }
    }

    uint_nt(Composer* parent_context, const std::vector<bool_ct>& wires)
        : __width(wires.size())
    {
        switch (__width) {
        case 8: {
            uint8 = uint8_ct(parent_context, wires);
            break;
        }
        case 16: {
            uint16 = uint16_ct(parent_context, wires);
            break;
        }
        case 32: {
            uint32 = uint32_ct(parent_context, wires);
            break;
        }
        default: {
            uint64 = uint64_ct(parent_context, wires);
            break;
        }
        }
    }

    template <size_t T>
    uint_nt(Composer* parent_context, const std::array<bool_ct, T>& wires)
        : __width(wires.size())
    {
        switch (__width) {
        case 8: {
            uint8 = uint8_ct(parent_context, wires);
            break;
        }
        case 16: {
            uint16 = uint16_ct(parent_context, wires);
            break;
        }
        case 32: {
            uint32 = uint32_ct(parent_context, wires);
            break;
        }
        default: {
            uint64 = uint64_ct(parent_context, wires);
            break;
        }
        }
    }

    uint_nt(const uint_nt& other)
        : __width(other.__width)
        , uint8(other.uint8)
        , uint16(other.uint16)
        , uint32(other.uint32)
        , uint64(other.uint64)
    {}

    uint_nt(uint_nt&& other)
        : __width(other.__width)
        , uint8(other.uint8)
        , uint16(other.uint16)
        , uint32(other.uint32)
        , uint64(other.uint64)
    {}

    uint_nt(const byte_array_ct& other)
        : __width(other.bits().size())
    {
        switch (__width) {
        case 8: {
            uint8 = uint8_ct(other);
            break;
        }
        case 16: {
            uint16 = uint16_ct(other);
            break;
        }
        case 32: {
            uint32 = uint32_ct(other);
            break;
        }
        default: {
            uint64 = uint64_ct(other);
            break;
        }
        }
    }

    // explicit uint_nt(char v)
    //     : __width(8)
    //     , uint8(uint256_t((uint64_t)v))
    // {}

    // explicit uint_nt(uint16_t v)
    //     : __width(16)
    //     , uint16(uint256_t((uint64_t)v))
    // {}

    // explicit uint_nt(uint32_t v)
    //     : __width(32)
    //     , uint32(uint256_t((uint64_t)v))
    // {}

    // explicit uint_nt(uint64_t v)
    //     : __width(64)
    //     , uint64(uint256_t((uint64_t)v))
    // {}

    // template <typename Native>
    // uint_nt(uint<Composer, Native> other)
    //     : __width(sizeof(Native) * 8)
    // {
    //     switch (__width) {
    //     case 8: {
    //         uint8 = other;
    //         break;
    //     }
    //     case 16: {
    //         uint16 = other;
    //         break;
    //     }
    //     case 32: {
    //         uint32 = other;
    //         break;
    //     }
    //     default: {
    //         uint64 = other;
    //         break;
    //     }
    //     }
    // }


    uint_nt(uint8_ct&& other)
        : __width(8)
        , uint8(other) {}

    uint_nt(uint16_ct&& other)
        : __width(16)
        , uint16(other) {}


    uint_nt(uint32_ct&& other)
        : __width(32)
        , uint32(other) {}


    uint_nt(uint64_ct&& other)
        : __width(64)
        , uint64(other) {}


    uint_nt(const uint8_ct& other)
        : __width(8)
        , uint8(other) {}

    uint_nt(const uint16_ct& other)
        : __width(16)
        , uint16(other) {}


    uint_nt(const uint32_ct& other)
        : __width(32)
        , uint32(other) {}


    uint_nt(const uint64_ct& other)
        : __width(64)
        , uint64(other) {}

    operator byte_array_ct()
    {
        switch (__width) {
        case 8: {
            return static_cast<byte_array_ct>(uint8);
            break;
        }
        case 16: {
            return static_cast<byte_array_ct>(uint16);
            break;
        }
        case 32: {
            return static_cast<byte_array_ct>(uint32);
            break;
        }
        default: {
            return static_cast<byte_array_ct>(uint64);
            break;
        }
        }
    }

    operator field_ct()
    {
        switch (__width) {
        case 8: {
            return static_cast<field_ct>(uint8);
            break;
        }
        case 16: {
            return static_cast<field_ct>(uint16);
            break;
        }
        case 32: {
            return static_cast<field_ct>(uint32);
            break;
        }
        default: {
            return static_cast<field_ct>(uint64);
            break;
        }
        }
    }

    uint_nt& operator=(const uint_nt& other)
    {
        __width = other.__width;
        uint8 = other.uint8;
        uint16 = other.uint16;
        uint32 = other.uint32;
        uint64 = other.uint64;
        return *this;
    }

    uint_nt operator+(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint_nt(uint8_ct(uint8 + other.uint8));
            break;
        }
        case 16: {
            return uint_nt(uint16_ct(uint16 + other.uint16));
            break;
        }
        case 32: {
            return uint_nt(uint32_ct(uint32 + other.uint32));
            break;
        }
        default: {
            return uint_nt(uint64_ct(uint64 + other.uint64));
            break;
        }
        }
    }


    uint_nt operator-(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 - other.uint8;
            break;
        }
        case 16: {
            return uint16 - other.uint16;
            break;
        }
        case 32: {
            return uint32 - other.uint32;
            break;
        }
        default: {
            return uint64 - other.uint64;
            break;
        }
        }
    }


    uint_nt operator*(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 * other.uint8;
            break;
        }
        case 16: {
            return uint16 * other.uint16;
            break;
        }
        case 32: {
            return uint32 * other.uint32;
            break;
        }
        default: {
            return uint64 * other.uint64;
            break;
        }
        }
    }

    uint_nt operator/(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 / other.uint8;
            break;
        }
        case 16: {
            return uint16 / other.uint16;
            break;
        }
        case 32: {
            return uint32 / other.uint32;
            break;
        }
        default: {
            return uint64 / other.uint64;
            break;
        }
        }
    }


    uint_nt operator%(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 % other.uint8;
            break;
        }
        case 16: {
            return uint16 % other.uint16;
            break;
        }
        case 32: {
            return uint32 % other.uint32;
            break;
        }
        default: {
            return uint64 % other.uint64;
            break;
        }
        }
    }


    uint_nt operator&(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 & other.uint8;
            break;
        }
        case 16: {
            return uint16 & other.uint16;
            break;
        }
        case 32: {
            return uint32 & other.uint32;
            break;
        }
        default: {
            return uint64 & other.uint64;
            break;
        }
        }
    }


    uint_nt operator|(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 | other.uint8;
            break;
        }
        case 16: {
            return uint16 | other.uint16;
            break;
        }
        case 32: {
            return uint32 | other.uint32;
            break;
        }
        default: {
            return uint64 | other.uint64;
            break;
        }
        }
    }


    uint_nt operator^(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 ^ other.uint8;
            break;
        }
        case 16: {
            return uint16 ^ other.uint16;
            break;
        }
        case 32: {
            return uint32 ^ other.uint32;
            break;
        }
        default: {
            return uint64 ^ other.uint64;
            break;
        }
        }
    }


    uint_nt operator~() const
    {
        switch (__width) {
        case 8: {
            return ~uint8;
            break;
        }
        case 16: {
            return ~uint16;
            break;
        }
        case 32: {
            return ~uint32;
            break;
        }
        default: {
            return ~uint64;
            break;
        }
        }
    }


    uint_nt operator>>(const size_t const_shift) const
    {
        switch (__width) {
        case 8: {
            return uint8 >> const_shift;
            break;
        }
        case 16: {
            return uint16 >> const_shift;
            break;
        }
        case 32: {
            return uint32 >> const_shift;
            break;
        }
        default: {
            return uint64 >> const_shift;
            break;
        }
        }
    }


    uint_nt operator<<(const size_t const_shift) const
    {
        switch (__width) {
        case 8: {
            return uint8 << const_shift;
            break;
        }
        case 16: {
            return uint16 << const_shift;
            break;
        }
        case 32: {
            return uint32 << const_shift;
            break;
        }
        default: {
            return uint64 << const_shift;
            break;
        }
        }
    }


    uint_nt ror(const size_t rot) const
    {
        switch (__width) {
        case 8: {
            return uint8.ror(rot);
            break;
        }
        case 16: {
            return uint16.ror(rot);
            break;
        }
        case 32: {
            return uint32.ror(rot);
            break;
        }
        default: {
            return uint64.ror(rot);
            break;
        }
        }
    }

    uint_nt rol(const size_t rot) const
    {
        switch (__width) {
        case 8: {
            return uint8.rol(rot);
            break;
        }
        case 16: {
            return uint16.rol(rot);
            break;
        }
        case 32: {
            return uint32.rol(rot);
            break;
        }
        default: {
            return uint64.rol(rot);
            break;
        }
        }
    }

    bool_ct operator>(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 > other.uint8;
            break;
        }
        case 16: {
            return uint16 > other.uint16;
            break;
        }
        case 32: {
            return uint32 > other.uint32;
            break;
        }
        default: {
            return uint64 > other.uint64;
            break;
        }
        }
    }


    bool_ct operator>=(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 >= other.uint8;
            break;
        }
        case 16: {
            return uint16 >= other.uint16;
            break;
        }
        case 32: {
            return uint32 >= other.uint32;
            break;
        }
        default: {
            return uint64 >= other.uint64;
            break;
        }
        }
    }

    bool_ct operator<(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 < other.uint8;
            break;
        }
        case 16: {
            return uint16 < other.uint16;
            break;
        }
        case 32: {
            return uint32 < other.uint32;
            break;
        }
        default: {
            return uint64 < other.uint64;
            break;
        }
        }
    }

    bool_ct operator<=(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 <= other.uint8;
            break;
        }
        case 16: {
            return uint16 <= other.uint16;
            break;
        }
        case 32: {
            return uint32 <= other.uint32;
            break;
        }
        default: {
            return uint64 <= other.uint64;
            break;
        }
        }
    }

    bool_ct operator==(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 == other.uint8;
            break;
        }
        case 16: {
            return uint16 == other.uint16;
            break;
        }
        case 32: {
            return uint32 == other.uint32;
            break;
        }
        default: {
            return uint64 == other.uint64;
            break;
        }
        }
    }


    bool_ct operator!=(const uint_nt& other) const
    {
        switch (__width) {
        case 8: {
            return uint8 != other.uint8;
            break;
        }
        case 16: {
            return uint16 != other.uint16;
            break;
        }
        case 32: {
            return uint32 != other.uint32;
            break;
        }
        default: {
            return uint64 != other.uint64;
            break;
        }
        }
    }

    bool_ct at(const size_t bit_index) const
    {
        switch (__width) {
        case 8: {
            return uint8.at(bit_index);
            break;
        }
        case 16: {
            return uint16.at(bit_index);
            break;
        }
        case 32: {
            return uint32.at(bit_index);
            break;
        }
        default: {
            return uint64.at(bit_index);
            break;
        }
        }
    }
    uint_nt operator++() { return operator+(uint_nt(width(), nullptr, 1)); };
    uint_nt operator--() { return operator-(uint_nt(width(), nullptr, 1)); };
    uint_nt operator+=(const uint_nt& other) { *this = operator+(other); return *this; };
    uint_nt operator-=(const uint_nt& other) { *this = operator-(other); return *this; };
    uint_nt operator*=(const uint_nt& other) { *this = operator*(other); return *this; };
    uint_nt operator/=(const uint_nt& other) { *this = operator/(other); return *this; };
    uint_nt operator%=(const uint_nt& other) { *this = operator%(other); return *this; };

    uint_nt operator&=(const uint_nt& other) { *this = operator&(other); return *this; };
    uint_nt operator^=(const uint_nt& other) { *this = operator^(other); return *this; };
    uint_nt operator|=(const uint_nt& other) { *this = operator|(other); return *this; };

    uint_nt operator>>=(const uint64_t const_shift) { *this = operator>>(const_shift); return *this; };
    uint_nt operator<<=(const uint64_t const_shift) { *this = operator<<(const_shift); return *this; };

    uint256_t get_value() const
    {
        switch (__width) {
        case 8: {
            return uint8.get_value().data[0];
            break;
        }
        case 16: {
            return uint16.get_value().data[0];
            break;
        }
        case 32: {
            return uint32.get_value().data[0];
            break;
        }
        default: {
            return uint64.get_value().data[0];
            break;
        }
        }
    }

    size_t width() const { return __width; }

    bool is_constant() const
    {
        switch (__width) {
        case 8: {
            return uint8.is_constant();
            break;
        }
        case 16: {
            return uint16.is_constant();
            break;
        }
        case 32: {
            return uint32.is_constant();
            break;
        }
        default: {
            return uint64.is_constant();
            break;
        }
        }
    }
  private:
    size_t __width;
  public:
    uint8_ct uint8;
    uint16_ct uint16;
    uint32_ct uint32;
    uint64_ct uint64;
};

inline std::ostream& operator<<(std::ostream& os, uint_nt const& v)
{
    auto flags = os.flags();
    os << std::hex << std::setfill('0');
    auto value =  v.get_value();
    switch(v.width()) {
        case 8: os << std::setw(2) << (int)static_cast<uint8_t>(value); break;
        case 16: os << std::setw(4) << static_cast<uint16_t>(value); break;
        case 32: os << std::setw(8) << static_cast<uint32_t>(value); break;
        case 64: os << std::setw(16) << static_cast<uint64_t>(value); break;
        default: os << v.get_value(); break;
    }
    os.flags(flags);
    return os;
}

}
}