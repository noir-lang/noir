#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct CallContext {
    typedef typename NCT::address address;
    typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    address msg_sender;
    address storage_contract_address;

    boolean is_delegate_call;
    boolean is_static_call;

    boolean is_callback;

    boolean is_fee_payment;
    boolean pay_fee_from_l1;
    boolean pay_fee_from_public_l2;
    boolean called_from_l1;
    boolean called_from_public_l2;

    boolean operator==(CallContext<NCT> const& other) const
    {
        return msg_sender == other.msg_sender && storage_contract_address == other.storage_contract_address;
    };

    static CallContext<NCT> empty() { return { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 }; };

    template <typename Composer> CallContext<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        CallContext<CircuitTypes<Composer>> call_context = {
            to_ct(msg_sender),       to_ct(storage_contract_address),
            to_ct(is_delegate_call), to_ct(is_static_call),
            to_ct(is_callback),      to_ct(is_fee_payment),
            to_ct(pay_fee_from_l1),  to_ct(pay_fee_from_public_l2),
            to_ct(called_from_l1),   to_ct(called_from_public_l2),

        };

        return call_context;
    };

    template <typename Composer> CallContext<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };

        CallContext<NativeTypes> call_context = {
            to_nt(msg_sender),       to_nt(storage_contract_address),
            to_nt(is_delegate_call), to_nt(is_static_call),
            to_nt(is_callback),      to_nt(is_fee_payment),
            to_nt(pay_fee_from_l1),  to_nt(pay_fee_from_public_l2),
            to_nt(called_from_l1),   to_nt(called_from_public_l2),
        };

        return call_context;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            msg_sender.to_field(), storage_contract_address.to_field(),
            fr(is_delegate_call),  fr(is_static_call),
            fr(is_callback),       fr(is_fee_payment),
            fr(pay_fee_from_l1),   fr(pay_fee_from_public_l2),
            fr(called_from_l1),    fr(called_from_public_l2),
        };

        return NCT::compress(inputs, GeneratorIndex::CALL_CONTEXT);
    }

    template <typename Composer> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        msg_sender.to_field().assert_is_zero();
        storage_contract_address.to_field().assert_is_zero();
        fr(is_delegate_call).assert_is_zero();
        fr(is_static_call).assert_is_zero();
        fr(is_callback).assert_is_zero();
        fr(is_fee_payment).assert_is_zero();
        fr(pay_fee_from_l1).assert_is_zero();
        fr(pay_fee_from_public_l2).assert_is_zero();
        fr(called_from_l1).assert_is_zero();
        fr(called_from_public_l2).assert_is_zero();
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        msg_sender.to_field().set_public();
        storage_contract_address.to_field().set_public();
        fr(is_delegate_call).set_public();
        fr(is_static_call).set_public();
        fr(is_callback).set_public();
        fr(is_fee_payment).set_public();
        fr(pay_fee_from_l1).set_public();
        fr(pay_fee_from_public_l2).set_public();
        fr(called_from_l1).set_public();
        fr(called_from_public_l2).set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, CallContext<NCT>& call_context)
{
    using serialize::read;

    read(it, call_context.msg_sender);
    read(it, call_context.storage_contract_address);
    read(it, call_context.is_delegate_call);
    read(it, call_context.is_static_call);
    read(it, call_context.is_callback);
    read(it, call_context.is_fee_payment);
    read(it, call_context.pay_fee_from_l1);
    read(it, call_context.pay_fee_from_public_l2);
    read(it, call_context.called_from_l1);
    read(it, call_context.called_from_public_l2);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, CallContext<NCT> const& call_context)
{
    using serialize::write;

    write(buf, call_context.msg_sender);
    write(buf, call_context.storage_contract_address);
    write(buf, call_context.is_delegate_call);
    write(buf, call_context.is_static_call);
    write(buf, call_context.is_callback);
    write(buf, call_context.is_fee_payment);
    write(buf, call_context.pay_fee_from_l1);
    write(buf, call_context.pay_fee_from_public_l2);
    write(buf, call_context.called_from_l1);
    write(buf, call_context.called_from_public_l2);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, CallContext<NCT> const& call_context)
{
    return os << "msg_sender: " << call_context.msg_sender << "\n"
              << "storage_contract_address: " << call_context.storage_contract_address << "\n"
              << "storage_contract_address: " << call_context.storage_contract_address << "\n"
              << "is_delegate_call: " << call_context.is_delegate_call << "\n"
              << "is_static_call: " << call_context.is_static_call << "\n"
              << "is_callback: " << call_context.is_callback << "\n"
              << "is_fee_payment: " << call_context.is_fee_payment << "\n"
              << "pay_fee_from_l1: " << call_context.pay_fee_from_l1 << "\n"
              << "pay_fee_from_public_l2: " << call_context.pay_fee_from_public_l2 << "\n"
              << "called_from_l1: " << call_context.called_from_l1 << "\n"
              << "called_from_public_l2: " << call_context.called_from_public_l2 << "\n";
}

} // namespace aztec3::circuits::abis