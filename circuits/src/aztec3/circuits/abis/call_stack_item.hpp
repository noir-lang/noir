#pragma once
#include "function_signature.hpp"
#include "call_context.hpp"
#include "private_circuit_public_inputs.hpp"
#include "public_circuit_public_inputs.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::conditional;
using std::is_same;

enum class CallType {
    Public,
    Private,
};

template <typename NCT, CallType call_type> struct CallStackItem {
    typedef typename NCT::boolean boolean;
    typedef typename NCT::fr fr;

    template <typename T>
    using PublicInputs = typename std::
        conditional<call_type == CallType::Public, PublicCircuitPublicInputs<T>, PrivateCircuitPublicInputs<T>>::type;

    FunctionSignature<NCT> function_signature;
    PublicInputs<NCT> public_inputs; // TODO: can we just do args?
    CallContext<NCT> call_context;
    boolean is_delegate_call = false;
    boolean is_static_call = false;

    bool operator==(CallStackItem<NCT, call_type> const&) const = default;

    template <typename T> static CallStackItem<NCT, call_type> empty()
    {
        return { FunctionSignature<NCT>::empty(), PublicInputs<NCT>::empty(), CallContext<NCT>::empty(), 0, 0 };
    };

    template <typename Composer>
    CallStackItem<CircuitTypes<Composer>, call_type> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        CallStackItem<CircuitTypes<Composer>, call_type> call_stack_item = {
            function_signature.to_circuit_type(composer),
            public_inputs.to_circuit_type(composer),
            call_context.to_circuit_type(composer),
            to_ct(is_delegate_call),
            to_ct(is_static_call),
        };

        return call_stack_item;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            function_signature.hash(), public_inputs.hash(), call_context.hash(),
            fr(is_delegate_call),      fr(is_static_call),
        };

        return NCT::compress(inputs, GeneratorIndex::CALL_STACK_ITEM);
    }
};

template <typename NCT, CallType call_type>
void read(uint8_t const*& it, CallStackItem<NCT, call_type>& call_stack_item)
{
    using serialize::read;

    read(it, call_stack_item.function_signature);
    read(it, call_stack_item.public_inputs_hash);
    read(it, call_stack_item.call_context);
    read(it, call_stack_item.is_delegate_call);
    read(it, call_stack_item.is_callback);
};

template <typename NCT, CallType call_type>
void write(std::vector<uint8_t>& buf, CallStackItem<NCT, call_type> const& call_stack_item)
{
    using serialize::write;

    write(buf, call_stack_item.function_signature);
    write(buf, call_stack_item.public_inputs_hash);
    write(buf, call_stack_item.call_context);
    write(buf, call_stack_item.is_delegate_call);
    write(buf, call_stack_item.is_static_call);
};

template <typename NCT, CallType call_type>
std::ostream& operator<<(std::ostream& os, CallStackItem<NCT, call_type> const& call_stack_item)
{
    return os << "function_signature: " << call_stack_item.function_signature << "\n"
              << "public_inputs: " << call_stack_item.public_inputs << "\n"
              << "call_context: " << call_stack_item.call_context << "\n"
              << "is_delegate_call: " << call_stack_item.is_delegate_call << "\n"
              << "is_static_call: " << call_stack_item.is_static_call << "\n";
}

} // namespace aztec3::circuits::abis