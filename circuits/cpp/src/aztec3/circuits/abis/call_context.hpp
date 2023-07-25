#pragma once

#include "aztec3/constants.hpp"
#include "aztec3/utils/msgpack_derived_equals.hpp"
#include "aztec3/utils/msgpack_derived_output.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct CallContext {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    address msg_sender = 0;
    address storage_contract_address = 0;
    fr portal_contract_address = 0;

    boolean is_delegate_call = false;
    boolean is_static_call = false;
    boolean is_contract_deployment = false;

    // for serialization, update with new fields
    MSGPACK_FIELDS(msg_sender,
                   storage_contract_address,
                   portal_contract_address,
                   is_delegate_call,
                   is_static_call,
                   is_contract_deployment);
    boolean operator==(CallContext<NCT> const& other) const
    {
        // we can't use =default with a custom boolean, but we can use a msgpack-derived utility
        return utils::msgpack_derived_equals<boolean>(*this, other);
    };

    template <typename Builder> CallContext<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        CallContext<CircuitTypes<Builder>> call_context = {
            to_ct(msg_sender),       to_ct(storage_contract_address), to_ct(portal_contract_address),
            to_ct(is_delegate_call), to_ct(is_static_call),           to_ct(is_contract_deployment),

        };

        return call_context;
    };

    template <typename Builder> CallContext<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        CallContext<NativeTypes> call_context = {
            to_nt(msg_sender),       to_nt(storage_contract_address), to_nt(portal_contract_address),
            to_nt(is_delegate_call), to_nt(is_static_call),           to_nt(is_contract_deployment),
        };

        return call_context;
    };

    fr hash() const
    {
        std::vector<fr> const inputs = {
            msg_sender.to_field(), storage_contract_address.to_field(), portal_contract_address, fr(is_delegate_call),
            fr(is_static_call),    fr(is_contract_deployment),
        };

        return NCT::hash(inputs, GeneratorIndex::CALL_CONTEXT);
    }

    template <typename Builder> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        msg_sender.to_field().assert_is_zero();
        storage_contract_address.to_field().assert_is_zero();
        portal_contract_address.assert_is_zero();
        fr(is_delegate_call).assert_is_zero();
        fr(is_static_call).assert_is_zero();
        fr(is_contract_deployment).assert_is_zero();
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        msg_sender.to_field().set_public();
        storage_contract_address.to_field().set_public();
        portal_contract_address.set_public();
        fr(is_delegate_call).set_public();
        fr(is_static_call).set_public();
        fr(is_contract_deployment).set_public();
    }
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, CallContext<NCT> const& call_context)
{
    utils::msgpack_derived_output(os, call_context);
    return os;
}

}  // namespace aztec3::circuits::abis
