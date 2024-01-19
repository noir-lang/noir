#pragma once
#include "../bool/bool.hpp"
#include "../circuit_builders/circuit_builders.hpp"
#include "../circuit_builders/circuit_builders_fwd.hpp"
#include "../field/field.hpp"
#include "../witness/witness.hpp"
#include "barretenberg/common/assert.hpp"
#include <functional>

// The purpose of this class is to enable positive integer operations without a risk of overflow.
// Despite the name, it is *not* a "safe" version of the uint class - as operations are positive integer
// operations, and not modulo 2^t for some t, as they are in the uint class.

namespace bb::plonk {
namespace stdlib {

template <typename Builder> class safe_uint_t {
  private:
    typedef field_t<Builder> field_ct;
    typedef bool_t<Builder> bool_ct;
    // this constructor is private since we only want the operators to be able to define a positive int without a range
    // check.
    safe_uint_t(field_ct const& value, uint256_t current_max, size_t safety)
        : value(value)
        , current_max(current_max)
    {
        ASSERT(safety == IS_UNSAFE);
        if (current_max > MAX_VALUE) // For optimal efficiency this should only be checked while testing a circuit
        {
            throw_or_abort("exceeded modulus in safe_uint class");
        }
    }

  public:
    // The following constant should be small enough that any thing with this bitnum is smaller than the modulus
    static constexpr size_t MAX_BIT_NUM = bb::fr::modulus.get_msb();
    static constexpr uint256_t MAX_VALUE = bb::fr::modulus - 1;
    static constexpr size_t IS_UNSAFE = 143; // weird constant to make it hard to use accidentally
    // Make sure our uint256 values don't wrap  - add_two function sums three of these
    static_assert((uint512_t)MAX_VALUE * 3 < (uint512_t)1 << 256);
    field_ct value;
    uint256_t current_max;

    safe_uint_t()
        : value(0)
        , current_max(0)
    {}

    safe_uint_t(field_ct const& value, size_t bit_num, std::string const& description = "unknown")
        : value(value)
    {
        ASSERT(bit_num <= MAX_BIT_NUM);
        this->value.create_range_constraint(bit_num, format("safe_uint_t range constraint failure: ", description));
        current_max = ((uint256_t)1 << bit_num) - 1;
    }

    // When initialzing a constant, we can set the max value to the constant itself (rather than the usually larger
    // 2^n-1)
    safe_uint_t(const bb::fr& const_value)
        : value(const_value)
        , current_max(const_value)
    {}

    // When initialzing a constant, we can set the max value to the constant itself (rather than the usually larger
    // 2^n-1)
    safe_uint_t(const uint256_t& const_value)
        : value(bb::fr(const_value))
        , current_max(bb::fr(const_value))
    {}
    safe_uint_t(const unsigned int& const_value)
        : value(bb::fr(const_value))
        , current_max(bb::fr(const_value))
    {}

    safe_uint_t(const safe_uint_t& other)
        : value(other.value)
        , current_max(other.current_max)
    {}

    static safe_uint_t<Builder> create_constant_witness(Builder* parent_context, bb::fr const& value)

    {
        witness_t<Builder> out(parent_context, value);
        parent_context->assert_equal_constant(out.witness_index, value);
        return safe_uint_t(value, uint256_t(value), IS_UNSAFE);
    }

    // We take advantage of the range constraint already being applied in the bool constructor and don't make a
    // redundant one.
    safe_uint_t(const bool_ct& other)
        : value(other)
        , current_max(1)
    {}

    explicit operator bool_ct() { return bool_ct(value); }
    static safe_uint_t from_witness_index(Builder* parent_context, const uint32_t witness_index);

    // Subtraction when you have a pre-determined bound on the difference size
    safe_uint_t subtract(const safe_uint_t& other,
                         const size_t difference_bit_size,
                         std::string const& description = "") const;

    safe_uint_t operator-(const safe_uint_t& other) const;

    // division when you have a pre-determined bound on the sizes of the quotient and remainder
    safe_uint_t divide(
        const safe_uint_t& other,
        const size_t quotient_bit_size,
        const size_t remainder_bit_size,
        std::string const& description = "",
        const std::function<std::pair<uint256_t, uint256_t>(uint256_t, uint256_t)>& get_quotient =
            [](uint256_t val, uint256_t divisor) {
                return std::make_pair((uint256_t)(val / (uint256_t)divisor), (uint256_t)(val % (uint256_t)divisor));
            }) const;

    // Potentially less efficient than divide function - bounds remainder and quotient by max of this
    safe_uint_t operator/(const safe_uint_t& other) const;

    safe_uint_t add_two(const safe_uint_t& add_a, const safe_uint_t& add_b) const
    {
        ASSERT(current_max + add_a.current_max + add_b.current_max <= MAX_VALUE && "Exceeded modulus in add_two");
        auto new_val = value.add_two(add_a.value, add_b.value);
        auto new_max = current_max + add_a.current_max + add_b.current_max;
        return safe_uint_t(new_val, new_max, IS_UNSAFE);
    }

    safe_uint_t madd(const safe_uint_t& to_mul, const safe_uint_t& to_add) const
    {
        ASSERT((uint512_t)current_max * (uint512_t)to_mul.current_max + (uint512_t)to_add.current_max <= MAX_VALUE &&
               "Exceeded modulus in madd");
        auto new_val = value.madd(to_mul.value, to_add.value);
        auto new_max = current_max * to_mul.current_max + to_add.current_max;
        return safe_uint_t(new_val, new_max, IS_UNSAFE);
    }

    safe_uint_t& operator=(const safe_uint_t& other)
    {
        value = other.value;
        current_max = other.current_max;
        return *this;
    }

    safe_uint_t& operator=(safe_uint_t&& other)
    {
        value = other.value;
        current_max = other.current_max;
        return *this;
    }

    safe_uint_t operator+=(const safe_uint_t& other)
    {
        *this = *this + other;
        return *this;
    }

    safe_uint_t operator*=(const safe_uint_t& other)
    {
        *this = *this * other;
        return *this;
    }

    std::array<safe_uint_t<Builder>, 3> slice(const uint8_t msb, const uint8_t lsb) const;
    void set_public() const { value.set_public(); }
    operator field_ct() { return value; }
    operator field_ct() const { return value; }
    safe_uint_t operator+(const safe_uint_t& other) const;
    safe_uint_t operator*(const safe_uint_t& other) const;
    bool_ct operator==(const safe_uint_t& other) const;
    bool_ct operator!=(const safe_uint_t& other) const;

    /**
     * normalize returns a safe_uint_t element where `multiplicative_constant = 1` and `additive_constant = 0`
     * i.e. the value is defined entirely by the builder variable that `witness_index` points to
     * If the witness_index is ever needed, `normalize` should be called first
     *
     * Will cost 1 constraint if the field element is not already normalized (or is constant)
     **/
    safe_uint_t normalize() const;

    bb::fr get_value() const;

    Builder* get_context() const { return value.context; }

    /**
     * is_zero will return a bool_ct, and add constraints that enforce its correctness
     * N.B. If you want to ENFORCE that a safe_uint_t object is zero, use `assert_is_zero`
     **/
    bool_ct is_zero() const;

    void assert_equal(const safe_uint_t& rhs, std::string const& msg = "safe_uint_t::assert_equal") const
    {
        this->value.assert_equal(rhs.value, msg);
    }
    void assert_is_not_zero(std::string const& msg = "safe_uint_t::assert_is_not_zero") const;
    void assert_is_zero(std::string const& msg = "safe_uint_t::assert_is_zero") const;
    bool is_constant() const { return value.is_constant(); }

    static safe_uint_t conditional_assign(const bool_ct& predicate, const safe_uint_t& lhs, const safe_uint_t& rhs)
    {
        auto new_val = (lhs.value - rhs.value).madd(predicate, rhs.value);
        auto new_max = lhs.current_max > rhs.current_max ? lhs.current_max : rhs.current_max;
        return safe_uint_t(new_val, new_max, IS_UNSAFE);
    }

    uint32_t get_witness_index() const { return value.get_witness_index(); }
};

template <typename Builder> inline std::ostream& operator<<(std::ostream& os, safe_uint_t<Builder> const& v)
{
    return os << v.value;
}
} // namespace stdlib
} // namespace bb::plonk
