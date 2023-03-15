#pragma once
#include "function_data.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct OptionallyRevealedData {
    typedef typename NCT::boolean boolean;
    typedef typename NCT::fr fr;

    fr call_stack_item_hash;
    FunctionData<NCT> function_data;
    std::array<fr, EMITTED_EVENTS_LENGTH> emitted_events;
    fr vk_hash;
    fr portal_contract_address; // an ETH address
    boolean pay_fee_from_l1;
    boolean pay_fee_from_public_l2;
    boolean called_from_l1;
    boolean called_from_public_l2;

    template <typename Composer>
    OptionallyRevealedData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        OptionallyRevealedData<CircuitTypes<Composer>> data = {
            to_ct(call_stack_item_hash),    function_data.to_circuit_type(composer),
            to_ct(emitted_events),          to_ct(vk_hash),
            to_ct(portal_contract_address), to_ct(pay_fee_from_l1),
            to_ct(pay_fee_from_public_l2),  to_ct(called_from_l1),
            to_ct(called_from_public_l2),
        };

        return data;
    };

    template <typename Composer> OptionallyRevealedData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        OptionallyRevealedData<NativeTypes> data = {
            to_nt(call_stack_item_hash),    to_native_type(function_data),
            to_nt(emitted_events),          to_nt(vk_hash),
            to_nt(portal_contract_address), to_nt(pay_fee_from_l1),
            to_nt(pay_fee_from_public_l2),  to_nt(called_from_l1),
            to_nt(called_from_public_l2),
        };

        return data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        call_stack_item_hash.set_public();
        function_data.set_public();
        set_array_public(emitted_events);
        vk_hash.set_public();
        portal_contract_address.set_public();
        fr(pay_fee_from_l1).set_public();
        fr(pay_fee_from_public_l2).set_public();
        fr(called_from_l1).set_public();
        fr(called_from_public_l2).set_public();
    }

    template <typename T, size_t SIZE> void set_array_public(std::array<T, SIZE>& arr)
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));
        for (T& e : arr) {
            fr(e).set_public();
        }
    }
};

} // namespace aztec3::circuits::abis