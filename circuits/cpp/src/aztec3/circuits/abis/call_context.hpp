#pragma once

#include <barretenberg/crypto/generators/generator_data.hpp>
#include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/constants.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> struct CallContext {
    typedef typename NCT::address address;
    typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    address msg_sender;
    address storage_contract_address;
    address tx_origin = msg_sender;

    boolean is_delegate_call;
    boolean is_static_call;
    boolean is_contract_deployment;

    boolean operator==(CallContext<NCT> const& other) const
    {
        return msg_sender == other.msg_sender && storage_contract_address == other.storage_contract_address &&
               tx_origin == other.tx_origin && is_delegate_call == other.is_delegate_call &&
               is_static_call == other.is_static_call && is_contract_deployment == other.is_contract_deployment;
    };

    static CallContext<NCT> empty() { return { 0, 0, 0, 0, 0, 0 }; };

    template <typename Composer> CallContext<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        CallContext<CircuitTypes<Composer>> call_context = {
            to_ct(msg_sender),       to_ct(storage_contract_address), to_ct(tx_origin),
            to_ct(is_delegate_call), to_ct(is_static_call),           to_ct(is_contract_deployment),

        };

        return call_context;
    };

    template <typename Composer> CallContext<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        CallContext<NativeTypes> call_context = {
            to_nt(msg_sender),       to_nt(storage_contract_address), to_nt(tx_origin),
            to_nt(is_delegate_call), to_nt(is_static_call),           to_nt(is_contract_deployment),
        };

        return call_context;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            msg_sender.to_field(), storage_contract_address.to_field(), tx_origin.to_field(), fr(is_delegate_call),
            fr(is_static_call),    fr(is_contract_deployment),
        };

        return NCT::compress(inputs, GeneratorIndex::CALL_CONTEXT);
    }

    template <typename Composer> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        msg_sender.to_field().assert_is_zero();
        storage_contract_address.to_field().assert_is_zero();
        tx_origin.to_field().assert_is_zero();
        fr(is_delegate_call).assert_is_zero();
        fr(is_static_call).assert_is_zero();
        fr(is_contract_deployment).assert_is_zero();
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        msg_sender.to_field().set_public();
        storage_contract_address.to_field().set_public();
        tx_origin.to_field().set_public();
        fr(is_delegate_call).set_public();
        fr(is_static_call).set_public();
        fr(is_contract_deployment).set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, CallContext<NCT>& call_context)
{
    using serialize::read;

    read(it, call_context.msg_sender);
    read(it, call_context.storage_contract_address);
    read(it, call_context.tx_origin);
    read(it, call_context.is_delegate_call);
    read(it, call_context.is_static_call);
    read(it, call_context.is_contract_deployment);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, CallContext<NCT> const& call_context)
{
    using serialize::write;

    write(buf, call_context.msg_sender);
    write(buf, call_context.storage_contract_address);
    write(buf, call_context.tx_origin);
    write(buf, call_context.is_delegate_call);
    write(buf, call_context.is_static_call);
    write(buf, call_context.is_contract_deployment);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, CallContext<NCT> const& call_context)
{
    return os << "msg_sender: " << call_context.msg_sender << "\n"
              << "storage_contract_address: " << call_context.storage_contract_address << "\n"
              << "tx_origin: " << call_context.tx_origin << "\n"
              << "is_delegate_call: " << call_context.is_delegate_call << "\n"
              << "is_static_call: " << call_context.is_static_call << "\n"
              << "is_contract_deployment: " << call_context.is_contract_deployment << "\n";
}

} // namespace aztec3::circuits::abis