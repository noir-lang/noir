#pragma once
#include "../bool/bool.hpp"
#include "../byte_array/byte_array.hpp"
#include "../composers/composers_fwd.hpp"
#include "../field/field.hpp"
#include "../plookup/plookup.hpp"

#include "./plookup/uint.hpp"

#include <honk/composer/standard_honk_composer.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/composer/ultra_composer.hpp>

namespace plonk {
namespace stdlib {

/**
 * @brief A standard library fixed-width unsigned integer type. Useful, e.g., for hashing.
 * Use safe_uint instead if looking to represent integer arithmetic inside of Fr.
 *
 * @tparam Composer The type of the composer context.
 * @tparam Native   One of the native uint types uintN_t, where N = 8, 16, 32, 64.
 *
 * @details All arithmetic operations in Native is done modulo 2**N, and the same is true for
 * uint<Composer, Native>.
 */

template <typename Composer, typename Native> class uint {
    // We only instantiate for widths 8, 16, 32 and 64, and only those widths were audited.
    static_assert((sizeof(Native) == 1) || (sizeof(Native) == 2) || (sizeof(Native) == 4) || (sizeof(Native) == 8));

  public:
    static constexpr size_t width = sizeof(Native) * 8;

    uint(const witness_t<Composer>& other);
    uint(const field_t<Composer>& other);
    uint(const uint256_t& value = 0);
    uint(Composer* composer, const uint256_t& value = 0);
    uint(const byte_array<Composer>& other);
    // constructors used for bit arrays
    uint(Composer* parent_context, const std::vector<bool_t<Composer>>& wires);
    uint(Composer* parent_context, const std::array<bool_t<Composer>, width>& wires);

    uint(const Native v)
        : uint(static_cast<uint256_t>(v))
    {}

    std::vector<uint32_t> constrain_accumulators(Composer* ctx,
                                                 const uint32_t witness_index,
                                                 const size_t num_bits,
                                                 std::string const& msg) const;

    static constexpr size_t num_accumulators()
    {
        return (width + Composer::UINT_LOG2_BASE - 1) / Composer::UINT_LOG2_BASE;
    }

    uint(const uint& other);
    uint(uint&& other);

    uint& operator=(const uint& other);
    uint& operator=(uint&& other);

    explicit operator byte_array<Composer>() const;
    explicit operator field_t<Composer>() const;

    uint operator+(const uint& other) const;
    uint operator-(const uint& other) const;
    uint operator*(const uint& other) const;
    uint operator/(const uint& other) const;
    uint operator%(const uint& other) const;

    uint operator&(const uint& other) const;
    uint operator^(const uint& other) const;
    uint operator|(const uint& other) const;
    uint operator~() const;

    uint operator>>(const size_t shift) const;
    uint operator<<(const size_t shift) const;

    uint ror(const size_t target_rotation) const;
    uint rol(const size_t target_rotation) const;
    uint ror(const uint256_t target_rotation) const { return ror(static_cast<size_t>(target_rotation.data[0])); }
    uint rol(const uint256_t target_rotation) const { return rol(static_cast<size_t>(target_rotation.data[0])); }

    bool_t<Composer> operator>(const uint& other) const;
    bool_t<Composer> operator<(const uint& other) const;
    bool_t<Composer> operator>=(const uint& other) const;
    bool_t<Composer> operator<=(const uint& other) const;
    bool_t<Composer> operator==(const uint& other) const;
    bool_t<Composer> operator!=(const uint& other) const;
    bool_t<Composer> operator!() const;

    uint operator+=(const uint& other)
    {
        *this = operator+(other);
        return *this;
    }
    uint operator-=(const uint& other)
    {
        *this = operator-(other);
        return *this;
    }
    uint operator*=(const uint& other)
    {
        *this = operator*(other);
        return *this;
    }
    uint operator/=(const uint& other)
    {
        *this = operator/(other);
        return *this;
    }
    uint operator%=(const uint& other)
    {
        *this = operator%(other);
        return *this;
    }

    uint operator&=(const uint& other)
    {
        *this = operator&(other);
        return *this;
    }
    uint operator^=(const uint& other)
    {
        *this = operator^(other);
        return *this;
    }
    uint operator|=(const uint& other)
    {
        *this = operator|(other);
        return *this;
    }

    uint operator>>=(const size_t shift)
    {
        *this = operator>>(shift);
        return *this;
    }
    uint operator<<=(const size_t shift)
    {
        *this = operator<<(shift);
        return *this;
    }

    uint256_t get_mask() const { return MASK; };

    uint normalize() const;

    uint256_t get_value() const;

    bool is_constant() const { return witness_index == IS_CONSTANT; }
    Composer* get_context() const { return context; }

    bool_t<Composer> at(const size_t bit_index) const;

    size_t get_width() const { return width; }

    uint32_t get_witness_index() const { return witness_index; }

    uint256_t get_additive_constant() const { return additive_constant; }

  protected:
    Composer* context;

    enum WitnessStatus { OK, NOT_NORMALIZED, WEAK_NORMALIZED };

    mutable uint256_t additive_constant;
    mutable WitnessStatus witness_status;
    mutable std::vector<uint32_t> accumulators;
    mutable uint32_t witness_index;

    static constexpr uint256_t CIRCUIT_UINT_MAX_PLUS_ONE = (uint256_t(1) << width);
    static constexpr uint256_t MASK = CIRCUIT_UINT_MAX_PLUS_ONE - 1;

  private:
    enum LogicOp {
        AND,
        XOR,
    };

    std::pair<uint, uint> divmod(const uint& other) const;
    uint logic_operator(const uint& other, const LogicOp op_type) const;
    uint weak_normalize() const;

    uint256_t get_unbounded_value() const;
};

template <typename T, typename w> inline std::ostream& operator<<(std::ostream& os, uint<T, w> const& v)
{
    return os << v.get_value();
}

template <typename ComposerContext>
using uint8 = typename std::conditional<ComposerContext::type == waffle::ComposerType::PLOOKUP,
                                        uint_plookup<ComposerContext, uint8_t>,
                                        uint<ComposerContext, uint8_t>>::type;
template <typename ComposerContext>
using uint16 = typename std::conditional<ComposerContext::type == waffle::ComposerType::PLOOKUP,
                                         uint_plookup<ComposerContext, uint16_t>,
                                         uint<ComposerContext, uint16_t>>::type;
template <typename ComposerContext>
using uint32 = typename std::conditional<ComposerContext::type == waffle::ComposerType::PLOOKUP,
                                         uint_plookup<ComposerContext, uint32_t>,
                                         uint<ComposerContext, uint32_t>>::type;
template <typename ComposerContext>
using uint64 = typename std::conditional<ComposerContext::type == waffle::ComposerType::PLOOKUP,
                                         uint_plookup<ComposerContext, uint64_t>,
                                         uint<ComposerContext, uint64_t>>::type;

EXTERN_STDLIB_BASIC_TYPE_VA(uint, uint8_t);
EXTERN_STDLIB_BASIC_TYPE_VA(uint, uint16_t);
EXTERN_STDLIB_BASIC_TYPE_VA(uint, uint32_t);
EXTERN_STDLIB_BASIC_TYPE_VA(uint, uint64_t);

} // namespace stdlib
} // namespace plonk