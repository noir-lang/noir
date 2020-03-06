#pragma once

#include <iostream>

#include "../common.hpp"
#include <ostream>

namespace waffle
{
    class StandardComposer;
    class MiMCComposer;
    class TurboComposer;
}

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

    bool get_value() const { return witness_bool ^ witness_inverted; }

    bool is_constant() const { return witness_index == static_cast<uint32_t>(-1); }

    bool_t normalize() const;

    mutable ComposerContext* context = nullptr;
    mutable bool witness_bool = false;
    mutable bool witness_inverted = false;
    mutable uint32_t witness_index = static_cast<uint32_t>(-1);
};

template <typename T> inline std::ostream& operator<<(std::ostream& os, bool_t<T> const& v)
{
    return os << v.get_value();
}

extern template class bool_t<waffle::StandardComposer>;
extern template class bool_t<waffle::MiMCComposer>;
extern template class bool_t<waffle::TurboComposer>;

} // namespace stdlib
} // namespace plonk
