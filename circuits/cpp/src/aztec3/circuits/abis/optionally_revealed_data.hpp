#pragma once
#include "function_data.hpp"

#include "aztec3/utils/array.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::zero_array;
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

    template <typename Composer>
    OptionallyRevealedData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        OptionallyRevealedData<CircuitTypes<Composer>> data = {
            to_ct(call_stack_item_hash),
            function_data.to_circuit_type(composer),
            to_ct(vk_hash),
            to_ct(portal_contract_address),
            to_ct(pay_fee_from_l1),
            to_ct(pay_fee_from_public_l2),
            to_ct(called_from_l1),
            to_ct(called_from_public_l2),
        };

        return data;
    };

    template <typename Composer> OptionallyRevealedData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

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

template <typename NCT> void read(uint8_t const*& it, OptionallyRevealedData<NCT>& data)
{
    using serialize::read;

    read(it, data.call_stack_item_hash);
    read(it, data.function_data);
    read(it, data.vk_hash);
    read(it, data.portal_contract_address);
    read(it, data.pay_fee_from_l1);
    read(it, data.pay_fee_from_public_l2);
    read(it, data.called_from_l1);
    read(it, data.called_from_public_l2);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, OptionallyRevealedData<NCT> const& data)
{
    using serialize::write;

    write(buf, data.call_stack_item_hash);
    write(buf, data.function_data);
    write(buf, data.vk_hash);
    write(buf, data.portal_contract_address);
    write(buf, data.pay_fee_from_l1);
    write(buf, data.pay_fee_from_public_l2);
    write(buf, data.called_from_l1);
    write(buf, data.called_from_public_l2);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, OptionallyRevealedData<NCT> const& data)
{
    return os << "call_stack_item_hash: " << data.call_stack_item_hash << "\n"
              << "function_data:\n"
              << data.function_data << "\n"
              << "vk_hash: " << data.vk_hash << "\n"
              << "portal_contract_address: " << data.portal_contract_address << "\n"
              << "pay_fee_from_l1: " << data.pay_fee_from_l1 << "\n"
              << "pay_fee_from_public_l2: " << data.pay_fee_from_public_l2 << "\n"
              << "called_from_l1: " << data.called_from_l1 << "\n"
              << "called_from_public_l2: " << data.called_from_public_l2 << "\n";
}

}  // namespace aztec3::circuits::abis