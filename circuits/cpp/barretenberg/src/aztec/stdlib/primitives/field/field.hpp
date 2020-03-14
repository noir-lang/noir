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
    field_t(const uint64_t value);
    field_t(const witness_t<ComposerContext>& value);
    field_t(const field_t& other);
    field_t(field_t&& other);
    field_t(byte_array<ComposerContext> const& other);

    field_t(const bool_t<ComposerContext>& other);
    operator bool_t<ComposerContext>();
    operator byte_array<ComposerContext>() const;

    field_t& operator=(const field_t& other);
    field_t& operator=(field_t&& other);

    field_t operator+(const field_t& other) const;
    field_t operator-(const field_t& other) const;
    field_t operator*(const field_t& other) const;
    field_t operator/(const field_t& other) const;

    bool_t<ComposerContext> operator==(const field_t& other) const;

    field_t normalize() const;

    barretenberg::fr get_value() const;

    bool_t<ComposerContext> is_zero();
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
