#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::apps::notes {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT, typename V> struct DefaultSingletonPrivateNotePreimage {
    using fr = typename NCT::fr;
    using grumpkin_point = typename NCT::grumpkin_point;
    using address = typename NCT::address;
    using boolean = typename NCT::boolean;

    // No custom constructors so that designated initializers can be used (for readability of test circuits).

    std::optional<V> value;
    std::optional<address> owner;

    std::optional<fr> salt;
    std::optional<fr> nonce;

    // For serialization, update with new fields
    MSGPACK_FIELDS(value, owner, salt, nonce);

    bool operator==(DefaultSingletonPrivateNotePreimage<NCT, V> const&) const = default;

    template <typename Builder> auto to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        // Depending on whether the _circuit_ type version of `V` is from the stdlib, or some custom type, the
        // conversion method will be different.
        const bool has_to_circuit_type = requires(V v) { v.to_circuit_type(); };
        const bool has_to_ct = requires(V v) { to_ct(v); };

        // To avoid messy template arguments in the calling code, we use a lambda function with `auto` return type to
        // avoid explicitly having to state the circuit type for `V`.
        auto circuit_value = [&]() -> auto {
            if constexpr (has_to_circuit_type) {
                return value.to_circuit_type();
            } else if (has_to_ct) {
                return to_ct(value);
            } else {
                throw_or_abort("Can't convert Value to circuit type");
            }
        }();

        // When this method is called, this class must be templated over native types. We can avoid templating over the
        // circuit types (for the return values) (in order to make the calling syntax cleaner) with the below `decltype`
        // deduction of the _circuit_ version of template type `V`.
        DefaultSingletonPrivateNotePreimage<CircuitTypes<Builder>, typename decltype(circuit_value)::value_type>
            preimage = {
                circuit_value,
                to_ct(owner),
                to_ct(salt),
                to_ct(nonce),
            };

        return preimage;
    };

    template <typename Builder> auto to_native_type() const
    {
        static_assert(!std::is_same<NativeTypes, NCT>::value);

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        // See `to_circuit_type()` for explanation of this code.
        const bool has_to_native_type = requires(V v) { v.to_native_type(); };
        const bool has_to_nt = requires(V v) { to_nt(v); };

        auto native_value = [&]() -> auto {
            if constexpr (has_to_native_type) {
                return value.to_native_type();
            } else if (has_to_nt) {
                return to_nt(value);
            } else {
                throw_or_abort("Can't convert Value to native type");
            }
        }();

        DefaultSingletonPrivateNotePreimage<NativeTypes, typename decltype(native_value)::value_type> preimage = {
            native_value,
            to_nt(owner),
            to_nt(salt),
            to_nt(nonce),
        };

        return preimage;
    };
};

}  // namespace aztec3::circuits::apps::notes
