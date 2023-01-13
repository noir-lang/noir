#pragma once
#include "function_signature.hpp"
#include "private_circuit_public_inputs.hpp"
#include "public_circuit_public_inputs.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <stdlib/types/native_types.hpp>

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
    typedef typename NCT::address address;
    typedef typename NCT::boolean boolean;
    typedef typename NCT::fr fr;

    template <typename T>
    using PublicInputs = typename std::
        conditional<call_type == CallType::Public, PublicCircuitPublicInputs<T>, PrivateCircuitPublicInputs<T>>::type;

    address contract_address;
    FunctionSignature<NCT> function_signature;
    PublicInputs<NCT> public_inputs;

    bool operator==(CallStackItem<NCT, call_type> const&) const = default;

    template <typename T> static CallStackItem<NCT, call_type> empty()
    {
        return { 0, FunctionSignature<NCT>::empty(), PublicInputs<NCT>::empty() };
    };

    template <typename Composer>
    CallStackItem<CircuitTypes<Composer>, call_type> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        CallStackItem<CircuitTypes<Composer>, call_type> call_stack_item = {
            to_ct(contract_address),
            function_signature.to_circuit_type(composer),
            public_inputs.to_circuit_type(composer),
        };

        return call_stack_item;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            contract_address.to_field(),
            function_signature.hash(),
            public_inputs.hash(),
        };

        fr call_stack_item_hash = NCT::compress(inputs, GeneratorIndex::CALL_STACK_ITEM);

        return call_stack_item_hash;
    }
}; // namespace aztec3::circuits::abis

template <typename NCT, CallType call_type>
void read(uint8_t const*& it, CallStackItem<NCT, call_type>& call_stack_item)
{
    using serialize::read;

    read(it, call_stack_item.contract_address);
    read(it, call_stack_item.function_signature);
    read(it, call_stack_item.public_inputs_hash);
};

template <typename NCT, CallType call_type>
void write(std::vector<uint8_t>& buf, CallStackItem<NCT, call_type> const& call_stack_item)
{
    using serialize::write;

    write(buf, call_stack_item.contract_address);
    write(buf, call_stack_item.function_signature);
    write(buf, call_stack_item.public_inputs_hash);
};

template <typename NCT, CallType call_type>
std::ostream& operator<<(std::ostream& os, CallStackItem<NCT, call_type> const& call_stack_item)
{
    return os << "contract_address: " << call_stack_item.contract_address << "\n"
              << "function_signature: " << call_stack_item.function_signature << "\n"
              << "public_inputs: " << call_stack_item.public_inputs << "\n";
}

} // namespace aztec3::circuits::abis