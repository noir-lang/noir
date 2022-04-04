#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct CallbackStackItem {
    typedef typename NCT::address address;
    typedef typename NCT::fr fr;

    address callback_public_key;
    fr success_callback_call_hash;
    fr failure_callback_call_hash;
    fr success_result_arg_map_acc;

    bool operator==(CallbackStackItem<NCT> const&) const = default;

    static CallbackStackItem<NCT> empty() { return { 0, 0, 0, 0 }; };

    template <typename Composer> CallbackStackItem<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        CallbackStackItem<CircuitTypes<Composer>> callback_stack_item = {
            to_ct(callback_public_key),
            to_ct(success_callback_call_hash),
            to_ct(failure_callback_call_hash),
            to_ct(success_result_arg_map_acc),
        };

        return callback_stack_item;
    };

    template <typename Composer> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        callback_public_key.to_field().assert_is_zero();
        success_callback_call_hash.assert_is_zero();
        failure_callback_call_hash.assert_is_zero();
        success_result_arg_map_acc.assert_is_zero();
    }

    template <typename Composer> void set_public()
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        callback_public_key.to_field().set_public();
        success_callback_call_hash.set_public();
        failure_callback_call_hash.set_public();
        success_result_arg_map_acc.set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, CallbackStackItem<NCT>& callback_stack_item)
{
    using serialize::read;

    read(it, callback_stack_item.callback_public_key);
    read(it, callback_stack_item.success_callback_call_hash);
    read(it, callback_stack_item.failure_callback_call_hash);
    read(it, callback_stack_item.success_result_arg_map_acc);
    read(it, callback_stack_item.is_callback);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, CallbackStackItem<NCT> const& callback_stack_item)
{
    using serialize::write;

    write(buf, callback_stack_item.callback_public_key);
    write(buf, callback_stack_item.success_callback_call_hash);
    write(buf, callback_stack_item.failure_callback_call_hash);
    write(buf, callback_stack_item.success_result_arg_map_acc);
    write(buf, callback_stack_item.is_static_call);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, CallbackStackItem<NCT> const& callback_stack_item)
{
    return os << "callback_public_key: " << callback_stack_item.callback_public_key << "\n"
              << "success_callback_call_hash: " << callback_stack_item.success_callback_call_hash << "\n"
              << "failure_callback_call_hash: " << callback_stack_item.failure_callback_call_hash << "\n"
              << "success_result_arg_map_acc: " << callback_stack_item.success_result_arg_map_acc << "\n";
}

} // namespace aztec3::circuits::abis