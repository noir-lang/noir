#pragma once
#include "../composers/composers_fwd.hpp"
#include "../witness/witness.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class bool_t {
  public:
    bool_t(const bool value = false);
    bool_t(ComposerContext* parent_context);
    bool_t(ComposerContext* parent_context, const bool value);
    bool_t(const witness_t<ComposerContext>& value);
    bool_t(const bool_t& other);
    bool_t(bool_t&& other);

    bool_t& operator=(const bool other);
    bool_t& operator=(const witness_t<ComposerContext>& other);
    bool_t& operator=(const bool_t& other);
    bool_t& operator=(bool_t&& other);
    // field_t& operator=(const barretenberg::fr &value);

    // bitwise operations
    bool_t operator&(const bool_t& other) const;
    bool_t operator|(const bool_t& other) const;
    bool_t operator^(const bool_t& other) const;
    bool_t operator!() const;

    // equality checks
    bool_t operator==(const bool_t& other) const;

    bool_t operator!=(const bool_t& other) const;

    // misc bool ops
    bool_t operator~() const { return operator!(); }

    bool_t operator&&(const bool_t& other) const;

    bool_t operator||(const bool_t& other) const;

    // self ops
    void operator|=(const bool_t& other) { *this = operator|(other); }

    void operator&=(const bool_t& other) { *this = operator&(other); }

    void operator^=(const bool_t& other) { *this = operator^(other); }

    void assert_equal(const bool_t& rhs, std::string const& msg = "bool_t::assert_equal") const
    {
        const bool_t lhs = *this;
        ComposerContext* ctx = lhs.get_context() ? lhs.get_context() : rhs.get_context();

        if (lhs.witness_index == IS_CONSTANT && rhs.witness_index == IS_CONSTANT) {
            ASSERT(lhs.get_value() == rhs.get_value());
        } else if (lhs.witness_index == IS_CONSTANT) {
            bool_t right = rhs.normalize();
            ctx->assert_equal_constant(right.witness_index, lhs.get_value(), msg);
        } else if (rhs.witness_index == IS_CONSTANT) {
            bool_t left = lhs.normalize();
            ctx->assert_equal_constant(left.witness_index, rhs.get_value(), msg);
        } else {
            bool_t left = lhs.normalize();
            bool_t right = rhs.normalize();
            ctx->assert_equal(left.witness_index, right.witness_index, msg);
        }
    }

    bool get_value() const { return witness_bool ^ witness_inverted; }

    bool is_constant() const { return witness_index == static_cast<uint32_t>(-1); }

    bool_t normalize() const;

    ComposerContext* get_context() const { return context; }

    mutable ComposerContext* context = nullptr;
    mutable bool witness_bool = false;
    mutable bool witness_inverted = false;
    mutable uint32_t witness_index = static_cast<uint32_t>(-1);
};

template <typename T> inline std::ostream& operator<<(std::ostream& os, bool_t<T> const& v)
{
    return os << v.get_value();
}

EXTERN_STDLIB_TYPE(bool_t);

} // namespace stdlib
} // namespace plonk
