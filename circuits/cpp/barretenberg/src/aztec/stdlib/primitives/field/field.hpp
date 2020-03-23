#pragma once
#include "../composers/composers_fwd.hpp"
#include "../witness/witness.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class bool_t;
template <typename ComposerContext> class byte_array;

template <typename ComposerContext> class field_t {
  public:
    field_t(ComposerContext* parent_context = nullptr);
    field_t(ComposerContext* parent_context, const barretenberg::fr& value);

    constexpr field_t(const int value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(static_cast<uint64_t>(value));
        multiplicative_constant = barretenberg::fr(0);
        witness_index = static_cast<uint32_t>(-1);
    }
    constexpr field_t(const uint64_t value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = static_cast<uint32_t>(-1);
    }
    constexpr field_t(const barretenberg::fr& value)
        : context(nullptr)
        , additive_constant(value)
        , multiplicative_constant(barretenberg::fr(1))
        , witness_index(static_cast<uint32_t>(-1))
    {}

    field_t(const witness_t<ComposerContext>& value);
    field_t(const field_t& other);
    field_t(field_t&& other);
    field_t(byte_array<ComposerContext> const& other);

    field_t(const bool_t<ComposerContext>& other);

    static field_t from_witness_index(ComposerContext* parent_context, const uint32_t witness_index);
    operator bool_t<ComposerContext>();
    operator byte_array<ComposerContext>() const;

    field_t& operator=(const field_t& other);
    field_t& operator=(field_t&& other);

    field_t operator+(const field_t& other) const;
    field_t operator-(const field_t& other) const;
    field_t operator*(const field_t& other) const;
    field_t operator/(const field_t& other) const;

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

    static constexpr field_t coset_generator(const size_t generator_idx)
    {
        return field_t(barretenberg::fr::coset_generator(generator_idx));
    }

    constexpr field_t operator-() const
    {
        if (std::is_constant_evaluated()) {
            field_t result(0);
            result.multiplicative_constant = -multiplicative_constant;
            result.additive_constant = -additive_constant;
            return result;
        }
        field_t result(*this);
        // if (witness_index == UINT32_MAX) {
        //     result.additive_constant -= result.additive_constant;
        // } else {
        result.multiplicative_constant = -result.multiplicative_constant;
        result.additive_constant = -result.additive_constant;
        //}
        return result;
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

    field_t normalize() const;

    barretenberg::fr get_value() const;

    bool_t<ComposerContext> is_zero();
    void assert_not_zero();
    bool is_constant() const { return witness_index == static_cast<uint32_t>(-1); }

    mutable ComposerContext* context = nullptr;
    mutable barretenberg::fr additive_constant;
    mutable barretenberg::fr multiplicative_constant;
    mutable uint32_t witness_index = static_cast<uint32_t>(-1);
};

template <typename ComposerContext> inline std::ostream& operator<<(std::ostream& os, field_t<ComposerContext> const& v)
{
    return os << v.get_value();
}

EXTERN_STDLIB_TYPE(field_t);

} // namespace stdlib
} // namespace plonk
