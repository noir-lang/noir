#pragma once
#include "function_signature.hpp"
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
    FunctionSignature<NCT> function_signature;
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
            to_ct(call_stack_item_hash),    function_signature.to_circuit_type(composer),
            to_ct(emitted_events),          to_ct(vk_hash),
            to_ct(portal_contract_address), to_ct(pay_fee_from_l1),
            to_ct(pay_fee_from_public_l2),  to_ct(called_from_l1),
            to_ct(called_from_public_l2),
        };

        return data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        call_stack_item_hash.set_public();
        function_signature.set_public();
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