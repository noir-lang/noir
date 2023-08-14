#pragma once
#include "function_data.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct OptionallyRevealedData {
    using address = typename NCT::address;
    using boolean = typename NCT::boolean;
    using fr = typename NCT::fr;

    fr call_stack_item_hash = 0;
    FunctionData<NCT> function_data{};
    fr vk_hash = 0;
    address portal_contract_address = 0;
    boolean pay_fee_from_l1 = false;
    boolean pay_fee_from_public_l2 = false;
    boolean called_from_l1 = false;
    boolean called_from_public_l2 = false;

    // for serialization: update up with new fields
    MSGPACK_FIELDS(call_stack_item_hash,
                   function_data,
                   vk_hash,
                   portal_contract_address,
                   pay_fee_from_l1,
                   pay_fee_from_public_l2,
                   called_from_l1,
                   called_from_public_l2);
    boolean operator==(OptionallyRevealedData<NCT> const& other) const
    {
        return call_stack_item_hash == other.call_stack_item_hash && function_data == other.function_data &&
               vk_hash == other.vk_hash && portal_contract_address == other.portal_contract_address &&
               pay_fee_from_l1 == other.pay_fee_from_l1 && pay_fee_from_public_l2 == other.pay_fee_from_public_l2 &&
               called_from_l1 == other.called_from_l1 && called_from_public_l2 == other.called_from_public_l2;
    };

    template <typename Builder> OptionallyRevealedData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        OptionallyRevealedData<CircuitTypes<Builder>> data = {
            to_ct(call_stack_item_hash),
            function_data.to_circuit_type(builder),
            to_ct(vk_hash),
            to_ct(portal_contract_address),
            to_ct(pay_fee_from_l1),
            to_ct(pay_fee_from_public_l2),
            to_ct(called_from_l1),
            to_ct(called_from_public_l2),
        };

        return data;
    };

    template <typename Builder> OptionallyRevealedData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        OptionallyRevealedData<NativeTypes> data = {
            to_nt(call_stack_item_hash),    to_native_type(function_data), to_nt(vk_hash),
            to_nt(portal_contract_address), to_nt(pay_fee_from_l1),        to_nt(pay_fee_from_public_l2),
            to_nt(called_from_l1),          to_nt(called_from_public_l2),
        };

        return data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        call_stack_item_hash.set_public();
        function_data.set_public();
        vk_hash.set_public();
        portal_contract_address.to_field().set_public();
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

}  // namespace aztec3::circuits::abis
