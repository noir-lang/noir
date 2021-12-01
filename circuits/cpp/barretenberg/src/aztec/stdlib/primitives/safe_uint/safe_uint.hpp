#pragma once
#include <functional>
#include "../composers/composers_fwd.hpp"
#include "../witness/witness.hpp"
#include "../bool/bool.hpp"
#include <common/assert.hpp>
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class bool_t;
template <typename ComposerContext> class field_t;

template <typename ComposerContext> class safe_uint_t {
  public:
    // The following constant should be small enough that any thing with this bitnum is smaller than the modulus
    static constexpr size_t MAX_BIT_NUM = fr::modulus.get_msb();
    static constexpr uint256_t MAX_VALUE = fr::modulus - 1;
    static constexpr size_t IS_UNSAFE = 143; // weird constant to make it hard to use accidentally
    // Make sure our uint256 values don't wrap  - add_two function sums three of these
    static_assert((uint512_t)MAX_VALUE * 3 < (uint512_t)1 << 256);
    field_t<ComposerContext> value;
    uint256_t current_max;

    safe_uint_t()
        : value(0)
        , current_max(0)
    {}

    safe_uint_t(field_t<ComposerContext> const& value, size_t bit_num, std::string const& description = "unknown")
        : value(value)
    {

        ASSERT(bit_num <= MAX_BIT_NUM);
        this->value.create_range_constraint(bit_num, format("safe_uint_t range constraint failure: ", description));
        current_max = ((uint256_t)1 << bit_num) - 1;
    }

    // When initialzing a constant, we can set the max value to the constant itself (rather than the usually larger
    // 2^n-1)
    safe_uint_t(const barretenberg::fr& const_value)
        : value(const_value)
        , current_max(const_value)
    {}

    // When initialzing a constant, we can set the max value to the constant itself (rather than the usually larger
    // 2^n-1)
    safe_uint_t(const uint256_t& const_value)
        : value(barretenberg::fr(const_value))
        , current_max(barretenberg::fr(const_value))
    {}
    safe_uint_t(const unsigned int& const_value)
        : value(barretenberg::fr(const_value))
        , current_max(barretenberg::fr(const_value))
    {}

    safe_uint_t(const safe_uint_t& other)
        : value(other.value)
        , current_max(other.current_max)
    {}

    static safe_uint_t<ComposerContext> create_constant_witness(ComposerContext* parent_context, fr const& value)

    {
        witness_t<ComposerContext> out(parent_context, value);
        parent_context->assert_equal_constant(out.witness_index, value);
        return safe_uint_t(value, uint256_t(value), IS_UNSAFE);
    }

    // We take advantage of the range constraint already being applied in the bool constructor and don't make a
    // redundant one.
    safe_uint_t(const bool_t<ComposerContext>& other)
        : value(other)
        , current_max(1)
    {}

    explicit operator bool_t<ComposerContext>() { return bool_t<ComposerContext>(value); }
    static safe_uint_t from_witness_index(ComposerContext* parent_context, const uint32_t witness_index);

    // Subtraction when you have a pre-determined bound on the difference size
    safe_uint_t subtract(const safe_uint_t& other, const size_t difference_bit_size) const
    {
        ASSERT(difference_bit_size <= MAX_BIT_NUM);
        field_t<ComposerContext> difference_val = this->value - other.value;
        safe_uint_t<ComposerContext> difference(difference_val, difference_bit_size);
        // This checks the subtraction is correct for integers without any wraps
        if (difference.current_max + other.current_max > MAX_VALUE)
            throw_or_abort("maximum value exceeded in positive_int subtract");
        return difference;
    }

    safe_uint_t operator-(const safe_uint_t& other) const
    {
        field_t<ComposerContext> difference_val = this->value - other.value;
        safe_uint_t<ComposerContext> difference(difference_val, (size_t)(current_max.get_msb() + 1));
        // This checks the subtraction is correct for integers without any wraps
        if (difference.current_max + other.current_max > MAX_VALUE)
            throw_or_abort("maximum value exceeded in positive_int minus operator");
        return difference;
    }

    // division when you have a pre-determined bound on the sizes of the quotient and remainder
    safe_uint_t divide(
        const safe_uint_t& other,
        const size_t quotient_bit_size,
        const size_t remainder_bit_size,
        const std::function<std::pair<uint256_t, uint256_t>(uint256_t, uint256_t)>& get_quotient =
            [](uint256_t val, uint256_t divisor) {
                return std::make_pair((uint256_t)(val / (uint256_t)divisor), (uint256_t)(val % (uint256_t)divisor));
            })
    {
        ASSERT(quotient_bit_size <= MAX_BIT_NUM);
        ASSERT(remainder_bit_size <= MAX_BIT_NUM);
        uint256_t val = this->value.get_value();
        auto [quotient_val, remainder_val] = get_quotient(val, (uint256_t)other.value.get_value());
        field_t<ComposerContext> quotient_field(witness_t(value.context, quotient_val));
        field_t<ComposerContext> remainder_field(witness_t(value.context, remainder_val));
        safe_uint_t<ComposerContext> quotient(quotient_field, quotient_bit_size);
        safe_uint_t<ComposerContext> remainder(remainder_field, remainder_bit_size);
        // This line implicitly checks we are not overflowing
        safe_uint_t int_val = quotient * other + remainder;
        this->assert_equal(int_val);

        return quotient;
    }

    // Potentially less efficient than divide function - bounds remainder and quotient by max of this
    safe_uint_t operator/(const safe_uint_t& other) const
    {
        uint256_t val = this->value.get_value();
        auto quotient_val = (uint256_t)(val / (uint256_t)other.value.get_value());
        auto remainder_val = (uint256_t)(val % (uint256_t)other.value.get_value());
        field_t<ComposerContext> quotient_field(witness_t(value.context, quotient_val));
        field_t<ComposerContext> remainder_field(witness_t(value.context, remainder_val));
        safe_uint_t<ComposerContext> quotient(quotient_field, (size_t)(current_max.get_msb() + 1));
        safe_uint_t<ComposerContext> remainder(remainder_field, (size_t)(current_max.get_msb() + 1));
        // This line implicitly checks we are not overflowing
        safe_uint_t int_val = quotient * other + remainder;
        this->assert_equal(int_val);

        return quotient;
    }
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

    std::array<safe_uint_t<ComposerContext>, 3> slice(const uint8_t msb, const uint8_t lsb) const;
    void set_public() const { value.set_public(); }
    operator field_t<ComposerContext>() { return value; }
    safe_uint_t operator+(const safe_uint_t& other) const;
    safe_uint_t operator*(const safe_uint_t& other) const;
    bool_t<ComposerContext> operator==(const safe_uint_t& other) const;
    bool_t<ComposerContext> operator!=(const safe_uint_t& other) const;

    /**
     * normalize returns a safe_uint_t element where `multiplicative_constant = 1` and `additive_constant = 0`
     * i.e. the value is defined entirely by the composer variable that `witness_index` points to
     * If the witness_index is ever needed, `normalize` should be called first
     *
     * Will cost 1 constraint if the field element is not already normalized (or is constant)
     **/
    safe_uint_t normalize() const;

    barretenberg::fr get_value() const;

    ComposerContext* get_context() const { return value.context; }

    /**
     * is_zero will return a bool_t, and add constraints that enforce its correctness
     * N.B. If you want to ENFORCE that a safe_uint_t object is zero, use `assert_is_zero`
     **/
    bool_t<ComposerContext> is_zero() const;

    void assert_equal(const safe_uint_t& rhs, std::string const& msg = "safe_uint_t::assert_equal") const
    {
        this->value.assert_equal(rhs.value, msg);
    }
    void assert_is_not_zero(std::string const& msg = "safe_uint_t::assert_is_not_zero") const;
    void assert_is_zero(std::string const& msg = "safe_uint_t::assert_is_zero") const;
    bool is_constant() const { return value.is_constant(); }

    static safe_uint_t conditional_assign(const bool_t<ComposerContext>& predicate,
                                          const safe_uint_t& lhs,
                                          const safe_uint_t& rhs)
    {
        auto new_val = (lhs.value - rhs.value).madd(predicate, rhs.value);
        auto new_max = lhs.current_max > rhs.current_max ? lhs.current_max : rhs.current_max;
        return safe_uint_t(new_val, new_max, IS_UNSAFE);
    }

    uint32_t get_witness_index() const { return value.get_witness_index(); }

  private:
    // this constructor is private since we only want the operators to be able to define a positive int without a range
    // check.
    safe_uint_t(field_t<ComposerContext> const& value, uint256_t current_max, size_t safety)
        : value(value)
        , current_max(current_max)
    {
        ASSERT(safety == IS_UNSAFE);
        if (current_max > MAX_VALUE) // For optimal efficiency this should only be checked while testing a circuit
        {
            throw_or_abort("exceeded modulus in positive_int class");
        }
    }
};

template <typename ComposerContext>
inline std::ostream& operator<<(std::ostream& os, safe_uint_t<ComposerContext> const& v)
{
    return os << v.value;
}

EXTERN_STDLIB_TYPE(safe_uint_t);

} // namespace stdlib
} // namespace plonk
