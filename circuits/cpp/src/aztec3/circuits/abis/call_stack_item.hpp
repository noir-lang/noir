#pragma once
#include "function_data.hpp"
#include "kernel_circuit_public_inputs.hpp"
#include "private_circuit_public_inputs.hpp"
#include "public_circuit_public_inputs.hpp"

#include "aztec3/circuits/abis/types.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT, template <class> typename PrivatePublic> struct CallStackItem {
    using address = typename NCT::address;
    using boolean = typename NCT::boolean;
    using fr = typename NCT::fr;

    // This is the _actual_ contract address relating to where this function's code resides in the
    // contract tree. Regardless of whether this is a call or delegatecall, this
    // `contract_address` _does not change_. Amongst other things, it's used as a lookup for
    // getting the correct code from the tree. There is a separate `storage_contract_address`
    // within a CallStackItem which varies depending on whether this is a call or delegatecall.
    address contract_address = 0;
    FunctionData<NCT> function_data{};
    typename PrivatePublic<NCT>::AppCircuitPublicInputs public_inputs{};
    // True if this call stack item represents a request to execute a function rather than a
    // fulfilled execution. Used when enqueuing calls from private to public functions.
    boolean is_execution_request = false;

    // for serialization, update with new fields
    MSGPACK_FIELDS(contract_address, function_data, public_inputs, is_execution_request);
    boolean operator==(CallContext<NCT> const& other) const
    {
        return contract_address == other.contract_address && function_data == other.function_data &&
               public_inputs == other.public_inputs && is_execution_request == other.is_execution_request;
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
            to_ct(is_execution_request),
        };

        return call_stack_item;
    };

    fr hash() const
    {
        const std::vector<fr> inputs = {
            contract_address.to_field(),
            function_data.hash(),
            public_inputs.hash(),
        };

        // NOLINTNEXTLINE(misc-const-correctness)
        fr call_stack_item_hash = NCT::compress(inputs, GeneratorIndex::CALL_STACK_ITEM);

        return call_stack_item_hash;
    }
};  // namespace aztec3::circuits::abis

template <typename NCT, template <class> typename PrivatePublic>
void read(uint8_t const*& it, CallStackItem<NCT, PrivatePublic>& call_stack_item)
{
    using serialize::read;

    read(it, call_stack_item.contract_address);
    read(it, call_stack_item.function_data);
    read(it, call_stack_item.public_inputs);
    read(it, call_stack_item.is_execution_request);
};

template <typename NCT, template <class> typename PrivatePublic>
void write(std::vector<uint8_t>& buf, CallStackItem<NCT, PrivatePublic> const& call_stack_item)
{
    using serialize::write;

    write(buf, call_stack_item.contract_address);
    write(buf, call_stack_item.function_data);
    write(buf, call_stack_item.public_inputs);
    write(buf, call_stack_item.is_execution_request);
};

template <typename NCT, template <class> typename PrivatePublic>
std::ostream& operator<<(std::ostream& os, CallStackItem<NCT, PrivatePublic> const& call_stack_item)
{
    return os << "contract_address: " << call_stack_item.contract_address << "\n"
              << "function_data: " << call_stack_item.function_data << "\n"
              << "public_inputs: " << call_stack_item.public_inputs << "\n"
              << "is_execution_request: " << call_stack_item.is_execution_request << "\n";
}

// Returns a copy of this call stack item where all result-related fields are zeroed out.
inline CallStackItem<NativeTypes, PublicTypes> as_execution_request(
    CallStackItem<NativeTypes, PublicTypes> const& call_stack_item)
{
    return {
        .contract_address = call_stack_item.contract_address,
        .function_data = call_stack_item.function_data,
        .public_inputs = {
            .call_context = call_stack_item.public_inputs.call_context,
            .args_hash = call_stack_item.public_inputs.args_hash,
        },
        .is_execution_request = call_stack_item.is_execution_request,
    };
};

// Returns the hash of a call stack item, or if the call stack item represents an execution request,
// zeroes out all fields but those related to the request (contract, function data, call context, args)
// and then hashes the item. Implemented only for native types for now.
inline fr get_call_stack_item_hash(abis::CallStackItem<NativeTypes, PublicTypes> const& call_stack_item)
{
    auto const& preimage =
        call_stack_item.is_execution_request ? as_execution_request(call_stack_item) : call_stack_item;
    return preimage.hash();
}

}  // namespace aztec3::circuits::abis