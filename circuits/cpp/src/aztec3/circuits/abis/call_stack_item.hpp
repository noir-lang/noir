#pragma once
#include "function_data.hpp"
#include "kernel_circuit_public_inputs.hpp"
#include "private_circuit_public_inputs.hpp"
#include "public_circuit_public_inputs.hpp"

#include "aztec3/circuits/abis/types.hpp"
#include "aztec3/utils/msgpack_derived_equals.hpp"
#include "aztec3/utils/msgpack_derived_output.hpp"
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
    // for schema serialization
    void msgpack_schema(auto& packer) const
    {
        packer.pack_with_name(PrivatePublic<NCT>::schema_name + std::string("CallStackItem"), *this);  // NOLINT
    }
    boolean operator==(CallContext<NCT> const& other) const
    {
        // we can't use =default with a custom boolean, but we can use a msgpack-derived utility
        return utils::msgpack_derived_equals<boolean>(*this, other);
    };

    template <typename Builder>
    CallStackItem<CircuitTypes<Builder>, PrivatePublic> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        CallStackItem<CircuitTypes<Builder>, PrivatePublic> call_stack_item = {
            to_ct(contract_address),
            function_data.to_circuit_type(builder),
            public_inputs.to_circuit_type(builder),
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
        fr call_stack_item_hash = NCT::hash(inputs, GeneratorIndex::CALL_STACK_ITEM);

        return call_stack_item_hash;
    }
};

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
