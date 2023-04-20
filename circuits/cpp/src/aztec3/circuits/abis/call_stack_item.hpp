#pragma once
#include "function_data.hpp"
#include "private_circuit_public_inputs.hpp"
#include "public_circuit_public_inputs.hpp"
#include "kernel_circuit_public_inputs.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::conditional;
using std::is_same;

template <typename NCT, template <class> typename PrivatePublic> struct CallStackItem {
    typedef typename NCT::address address;
    typedef typename NCT::boolean boolean;
    typedef typename NCT::fr fr;

    // This is the _actual_ contract address relating to where this function's code resides in the
    // contract tree. Regardless of whether this is a call or delegatecall, this
    // `contract_address` _does not change_. Amongst other things, it's used as a lookup for
    // getting the correct code from the tree. There is a separate `storage_contract_address`
    // within a CallStackItem which varies depending on whether this is a call or delegatecall.
    address contract_address = 0;
    FunctionData<NCT> function_data{};
    typename PrivatePublic<NCT>::AppCircuitPublicInputs public_inputs{};

    boolean operator==(CallContext<NCT> const& other) const
    {
        return contract_address == other.contract_address && function_data == other.function_data &&
               public_inputs == other.public_inputs;
    };

    template <typename Composer>
    CallStackItem<CircuitTypes<Composer>, PrivatePublic> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        CallStackItem<CircuitTypes<Composer>, PrivatePublic> call_stack_item = {
            to_ct(contract_address),
            function_data.to_circuit_type(composer),
            public_inputs.to_circuit_type(composer),
        };

        return call_stack_item;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            contract_address.to_field(),
            function_data.hash(),
            public_inputs.hash(),
        };

        fr call_stack_item_hash = NCT::compress(inputs, GeneratorIndex::CALL_STACK_ITEM);

        return call_stack_item_hash;
    }
}; // namespace aztec3::circuits::abis

template <typename NCT, template <class> typename PrivatePublic>
void read(uint8_t const*& it, CallStackItem<NCT, PrivatePublic>& call_stack_item)
{
    using serialize::read;

    read(it, call_stack_item.contract_address);
    read(it, call_stack_item.function_data);
    read(it, call_stack_item.public_inputs);
};

template <typename NCT, template <class> typename PrivatePublic>
void write(std::vector<uint8_t>& buf, CallStackItem<NCT, PrivatePublic> const& call_stack_item)
{
    using serialize::write;

    write(buf, call_stack_item.contract_address);
    write(buf, call_stack_item.function_data);
    write(buf, call_stack_item.public_inputs);
};

template <typename NCT, template <class> typename PrivatePublic>
std::ostream& operator<<(std::ostream& os, CallStackItem<NCT, PrivatePublic> const& call_stack_item)
{
    return os << "contract_address: " << call_stack_item.contract_address << "\n"
              << "function_data: " << call_stack_item.function_data << "\n"
              << "public_inputs: " << call_stack_item.public_inputs << "\n";
}

} // namespace aztec3::circuits::abis