#pragma once
#include "../composers/composers_fwd.hpp"
#include "../witness/witness.hpp"
#include <common/assert.hpp>

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class bool_t;

template <typename ComposerContext> class field_t {
  public:
    field_t(ComposerContext* parent_context = nullptr);
    field_t(ComposerContext* parent_context, const barretenberg::fr& value);

    field_t(const int value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(const unsigned long long value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(const unsigned long value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(const barretenberg::fr& value)
        : context(nullptr)
        , additive_constant(value)
        , multiplicative_constant(barretenberg::fr(1))
        , witness_index(IS_CONSTANT)
    {}

    field_t(const witness_t<ComposerContext>& value);

    field_t(const field_t& other)
        : context(other.context)
        , additive_constant(other.additive_constant)
        , multiplicative_constant(other.multiplicative_constant)
        , witness_index(other.witness_index)
    {}

    field_t(field_t&& other)
        : context(other.context)
        , additive_constant(other.additive_constant)
        , multiplicative_constant(other.multiplicative_constant)
        , witness_index(other.witness_index)
    {}

    field_t(const bool_t<ComposerContext>& other);

    static constexpr bool is_composite = false;
    static constexpr uint256_t modulus = barretenberg::fr::modulus;

    static field_t from_witness_index(ComposerContext* parent_context, const uint32_t witness_index);

    explicit operator bool_t<ComposerContext>();

    field_t& operator=(const field_t& other)
    {
        additive_constant = other.additive_constant;
        multiplicative_constant = other.multiplicative_constant;
        witness_index = other.witness_index;
        context = (other.context == nullptr ? nullptr : other.context);
        return *this;
    }

    field_t& operator=(field_t&& other)
    {
        additive_constant = other.additive_constant;
        multiplicative_constant = other.multiplicative_constant;
        witness_index = other.witness_index;
        context = (other.context == nullptr ? nullptr : other.context);
        return *this;
    }

    field_t operator+(const field_t& other) const;
    field_t operator-(const field_t& other) const;
    field_t operator*(const field_t& other) const;
    field_t operator/(const field_t& other) const;

    field_t sqr() const { return operator*(*this); }

    field_t operator+=(const field_t& other)
    {
        *this = *this + other;
        return *this;
    }
    field_t operator-=(const field_t& other)
    {
        *this = *this - other;
        return *this;
    }
    field_t operator*=(const field_t& other)
    {
        *this = *this * other;
        return *this;
    }
    field_t operator/=(const field_t& other)
    {
        *this = *this / other;
        return *this;
    }

    field_t invert() const { return (field_t(1) / field_t(*this)).normalize(); }

    static field_t coset_generator(const size_t generator_idx)
    {
        return field_t(barretenberg::fr::coset_generator(generator_idx));
    }

    static field_t external_coset_generator() { return field_t(barretenberg::fr::external_coset_generator()); }

    field_t operator-() const
    {
        field_t result(*this);
        result.multiplicative_constant = -multiplicative_constant;
        result.additive_constant = -additive_constant;

        return result;
    }

    field_t conditional_negate(const bool_t<ComposerContext>& predicate) const;

    void assert_equal(const field_t& rhs) const
    {
        const field_t lhs = *this;
        ComposerContext* ctx = lhs.get_context() ? lhs.get_context() : rhs.get_context();

        if (lhs.witness_index == UINT32_MAX && rhs.witness_index == UINT32_MAX) {
            ASSERT(lhs.get_value() == rhs.get_value());
        } else if (lhs.witness_index == UINT32_MAX) {
            field_t right = rhs.normalize();
            ctx->assert_equal_constant(right.witness_index, lhs.get_value());
        } else if (rhs.witness_index == UINT32_MAX) {
            field_t left = lhs.normalize();
            ctx->assert_equal_constant(left.witness_index, rhs.get_value());
        } else {
            field_t left = lhs.normalize();
            field_t right = rhs.normalize();
            ctx->assert_equal(left.witness_index, right.witness_index);
        }
    }

    static std::array<field_t, 4> preprocess_two_bit_table(const field_t& T0,
                                                           const field_t& T1,
                                                           const field_t& T2,
                                                           const field_t& T3);
    static field_t select_from_two_bit_table(const std::array<field_t, 4>& table,
                                             const bool_t<ComposerContext>& t1,
                                             const bool_t<ComposerContext>& t0);

    static std::array<field_t, 8> preprocess_three_bit_table(const field_t& T0,
                                                             const field_t& T1,
                                                             const field_t& T2,
                                                             const field_t& T3,
                                                             const field_t& T4,
                                                             const field_t& T5,
                                                             const field_t& T6,
                                                             const field_t& T7);
    static field_t select_from_three_bit_table(const std::array<field_t, 8>& table,
                                               const bool_t<ComposerContext>& t2,
                                               const bool_t<ComposerContext>& t1,
                                               const bool_t<ComposerContext>& t0);

    static void evaluate_polynomial_identity(const field_t& a, const field_t& b, const field_t& c, const field_t& d);

    field_t madd(const field_t& to_mul, const field_t& to_add) const;
    field_t add_two(const field_t& add_a, const field_t& add_b) const;
    bool_t<ComposerContext> operator==(const field_t& other) const;
    bool_t<ComposerContext> operator!=(const field_t& other) const;

    field_t normalize() const;

    barretenberg::fr get_value() const;

    ComposerContext* get_context() const { return context; }

    bool_t<ComposerContext> is_zero() const;
    void assert_is_not_zero();
    void assert_is_zero();
    bool is_constant() const { return witness_index == IS_CONSTANT; }

    uint32_t get_witness_index() const { return witness_index; }

    mutable ComposerContext* context = nullptr;
    mutable barretenberg::fr additive_constant;
    mutable barretenberg::fr multiplicative_constant;
    mutable uint32_t witness_index = IS_CONSTANT;
};

template <typename ComposerContext> inline std::ostream& operator<<(std::ostream& os, field_t<ComposerContext> const& v)
{
    return os << v.get_value();
}

EXTERN_STDLIB_TYPE(field_t);

} // namespace stdlib
} // namespace plonk
