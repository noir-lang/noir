#pragma once

#include "../common.hpp"
#include "../byte_array/byte_array.hpp"
#include "../field/field.hpp"
#include "../bool/bool.hpp"

#include "./uint.hpp"

#include <vector>
#include <iostream>

#include "../bool/bool.hpp"
#include "../byte_array/byte_array.hpp"
#include "../common.hpp"
#include "../int_utils.hpp"

namespace plonk {
namespace stdlib {
template <typename Composer> class uintNoir {
  public:
    uintNoir(size_t width, const uint64_t other)
        : __width(width)
        , uint8(uint256_t(other))
        , uint16(uint256_t(other))
        , uint32(uint256_t(other))
        , uint64(uint256_t(other))
    {}

    uintNoir(size_t width, const uint256_t other)
        : __width(width)
        , uint8(other)
        , uint16(other)
        , uint32(other)
        , uint64(other)
    {}


    uintNoir(uint64_t value)
        : __width(32)
        , uint8(uint8_t(value))
        , uint16(uint16_t(value))
        , uint32(uint32_t(value))
        , uint64(uint64_t(value))
    {}

    uintNoir(size_t width, Composer* parent_context)
        : __width(width)
        , uint8(parent_context, uint256_t(0))
        , uint16(parent_context, uint256_t(0))
        , uint32(parent_context, uint256_t(0))
        , uint64(parent_context, uint256_t(0))
    {}

    uintNoir(size_t width, const witness_t<Composer>& value)
        : __width(width)
    {
        switch (__width) {
        case 8: {
            uint8 = uint<Composer, uint8_t>(value);
            break;
        }
        case 16: {
            uint16 = uint<Composer, uint16_t>(value);
            break;
        }
        case 32: {
            uint32 = uint<Composer, uint32_t>(value);
            break;
        }
        default: {
            uint64 = uint<Composer, uint64_t>(value);
            break;
        }
        }
    }

    uintNoir(size_t width, Composer* parent_context, const uint64_t other)
        : __width(width)
        , uint8(parent_context, uint256_t(other))
        , uint16(parent_context, uint256_t(other))
        , uint32(parent_context, uint256_t(other))
        , uint64(parent_context, uint256_t(other))
    {}

    uintNoir(size_t width, const field_t<Composer>& value)
        : __width(width)
    {
        switch (__width) {
        case 8: {
            uint8 = uint<Composer, uint8_t>(value);
            break;
        }
        case 16: {
            uint16 = uint<Composer, uint16_t>(value);
            break;
        }
        case 32: {
            uint32 = uint<Composer, uint32_t>(value);
            break;
        }
        default: {
            uint64 = uint<Composer, uint64_t>(value);
            break;
        }
        }
    }

    uintNoir(Composer* parent_context, const std::vector<bool_t<Composer>>& wires)
        : __width(wires.size())
    {
        switch (__width) {
        case 8: {
            uint8 = uint<Composer, uint8_t>(parent_context, wires);
            break;
        }
        case 16: {
            uint16 = uint<Composer, uint16_t>(parent_context, wires);
            break;
        }
        case 32: {
            uint32 = uint<Composer, uint32_t>(parent_context, wires);
            break;
        }
        default: {
            uint64 = uint<Composer, uint64_t>(parent_context, wires);
            break;
        }
        }
    }

    template <size_t T>
    uintNoir(Composer* parent_context, const std::array<bool_t<Composer>, T>& wires)
        : __width(wires.size())
    {
        switch (__width) {
        case 8: {
            uint8 = uint<Composer, uint8_t>(parent_context, wires);
            break;
        }
        case 16: {
            uint16 = uint<Composer, uint16_t>(parent_context, wires);
            break;
        }
        case 32: {
            uint32 = uint<Composer, uint32_t>(parent_context, wires);
            break;
        }
        default: {
            uint64 = uint<Composer, uint64_t>(parent_context, wires);
            break;
        }
        }
    }

    uintNoir(const uintNoir& other)
        : __width(other.__width)
        , uint8(other.uint8)
        , uint16(other.uint16)
        , uint32(other.uint32)
        , uint64(other.uint64)
    {}

    uintNoir(uintNoir&& other)
        : __width(other.__width)
        , uint8(other.uint8)
        , uint16(other.uint16)
        , uint32(other.uint32)
        , uint64(other.uint64)
    {}

    uintNoir(const byte_array<Composer>& other)
        : __width(other.bits().size())
    {
        switch (__width) {
        case 8: {
            uint8 = uint<Composer, uint8_t>(other);
            break;
        }
        case 16: {
            uint16 = uint<Composer, uint16_t>(other);
            break;
        }
        case 32: {
            uint32 = uint<Composer, uint32_t>(other);
            break;
        }
        default: {
            uint64 = uint<Composer, uint64_t>(other);
            break;
        }
        }
    }

    // explicit uintNoir(char v)
    //     : __width(8)
    //     , uint8(uint256_t((uint64_t)v))
    // {}

    // explicit uintNoir(uint16_t v)
    //     : __width(16)
    //     , uint16(uint256_t((uint64_t)v))
    // {}

    // explicit uintNoir(uint32_t v)
    //     : __width(32)
    //     , uint32(uint256_t((uint64_t)v))
    // {}

    // explicit uintNoir(uint64_t v)
    //     : __width(64)
    //     , uint64(uint256_t((uint64_t)v))
    // {}

    // template <typename Native>
    // uintNoir(uint<Composer, Native> other)
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


    uintNoir(uint<Composer, uint8_t>&& other)
        : __width(8)
        , uint8(other) {}

    uintNoir(uint<Composer, uint16_t>&& other)
        : __width(16)
        , uint16(other) {}


    uintNoir(uint<Composer, uint32_t>&& other)
        : __width(32)
        , uint32(other) {}


    uintNoir(uint<Composer, uint64_t>&& other)
        : __width(64)
        , uint64(other) {}


    uintNoir(const uint<Composer, uint8_t>& other)
        : __width(8)
        , uint8(other) {}

    uintNoir(const uint<Composer, uint16_t>& other)
        : __width(16)
        , uint16(other) {}


    uintNoir(const uint<Composer, uint32_t>& other)
        : __width(32)
        , uint32(other) {}


    uintNoir(const uint<Composer, uint64_t>& other)
        : __width(64)
        , uint64(other) {}

    operator byte_array<Composer>()
    {
        switch (__width) {
        case 8: {
            return static_cast<byte_array<Composer>>(uint8);
            break;
        }
        case 16: {
            return static_cast<byte_array<Composer>>(uint16);
            break;
        }
        case 32: {
            return static_cast<byte_array<Composer>>(uint32);
            break;
        }
        default: {
            return static_cast<byte_array<Composer>>(uint64);
            break;
        }
        }
    }

    operator field_t<Composer>()
    {
        switch (__width) {
        case 8: {
            return static_cast<field_t<Composer>>(uint8);
            break;
        }
        case 16: {
            return static_cast<field_t<Composer>>(uint16);
            break;
        }
        case 32: {
            return static_cast<field_t<Composer>>(uint32);
            break;
        }
        default: {
            return static_cast<field_t<Composer>>(uint64);
            break;
        }
        }
    }

    uintNoir& operator=(const uintNoir& other)
    {
        __width = other.__width;
        uint8 = other.uint8;
        uint16 = other.uint16;
        uint32 = other.uint32;
        uint64 = other.uint64;
        return *this;
    }

    uintNoir operator+(const uintNoir& other) const
    {
        switch (__width) {
        case 8: {
            return uintNoir<Composer>(uint<Composer, uint8_t>(uint8 + other.uint8));
            break;
        }
        case 16: {
            return uintNoir<Composer>(uint<Composer, uint16_t>(uint16 + other.uint16));
            break;
        }
        case 32: {
            return uintNoir<Composer>(uint<Composer, uint32_t>(uint32 + other.uint32));
            break;
        }
        default: {
            return uintNoir<Composer>(uint<Composer, uint64_t>(uint64 + other.uint64));
            break;
        }
        }
    }


    uintNoir operator-(const uintNoir& other) const
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


    uintNoir operator*(const uintNoir& other) const
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

    uintNoir operator/(const uintNoir& other) const
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


    uintNoir operator%(const uintNoir& other) const
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


    uintNoir operator&(const uintNoir& other) const
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


    uintNoir operator|(const uintNoir& other) const
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


    uintNoir operator^(const uintNoir& other) const
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


    uintNoir operator~() const
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


    uintNoir operator>>(const size_t const_shift) const
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


    uintNoir operator<<(const size_t const_shift) const
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


    uintNoir ror(const size_t rot) const
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

    uintNoir rol(const size_t rot) const
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

    bool_t<Composer> operator>(const uintNoir& other) const
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


    bool_t<Composer> operator>=(const uintNoir& other) const
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

    bool_t<Composer> operator<(const uintNoir& other) const
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

    bool_t<Composer> operator<=(const uintNoir& other) const
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

    bool_t<Composer> operator==(const uintNoir& other) const
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


    bool_t<Composer> operator!=(const uintNoir& other) const
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

    bool_t<Composer> at(const size_t bit_index) const
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
    uintNoir operator++() { return operator+(uintNoir(width(), nullptr, 1)); };
    uintNoir operator--() { return operator-(uintNoir(width(), nullptr, 1)); };
    uintNoir operator+=(const uintNoir& other) { *this = operator+(other); return *this; };
    uintNoir operator-=(const uintNoir& other) { *this = operator-(other); return *this; };
    uintNoir operator*=(const uintNoir& other) { *this = operator*(other); return *this; };
    uintNoir operator/=(const uintNoir& other) { *this = operator/(other); return *this; };
    uintNoir operator%=(const uintNoir& other) { *this = operator%(other); return *this; };

    uintNoir operator&=(const uintNoir& other) { *this = operator&(other); return *this; };
    uintNoir operator^=(const uintNoir& other) { *this = operator^(other); return *this; };
    uintNoir operator|=(const uintNoir& other) { *this = operator|(other); return *this; };

    uintNoir operator>>=(const uint64_t const_shift) { *this = operator>>(const_shift); return *this; };
    uintNoir operator<<=(const uint64_t const_shift) { *this = operator<<(const_shift); return *this; };

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
    uint<Composer, uint8_t> uint8;
    uint<Composer, uint16_t> uint16;
    uint<Composer, uint32_t> uint32;
    uint<Composer, uint64_t> uint64;
};

template <typename T> inline std::ostream& operator<<(std::ostream& os, uintNoir<T> const& v)
{
    return os << v.get_value();
}

}
}