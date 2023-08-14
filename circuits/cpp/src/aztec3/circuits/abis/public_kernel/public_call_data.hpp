#pragma once

#include "../call_stack_item.hpp"
#include "../types.hpp"

#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis::public_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PublicCallData {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;
    using VK = typename NCT::VK;

    CallStackItem<NCT, PublicTypes> call_stack_item{};

    std::array<CallStackItem<NCT, PublicTypes>, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> public_call_stack_preimages{};

    NativeTypes::Proof proof{};  // TODO: how to express proof as native/circuit type when it gets used as a buffer?

    fr portal_contract_address = 0;  // an ETH address
    fr bytecode_hash = 0;

    // for serialization, update with new fields
    MSGPACK_FIELDS(call_stack_item, public_call_stack_preimages, proof, portal_contract_address, bytecode_hash);
    boolean operator==(PublicCallData<NCT> const& other) const
    {
        // WARNING: proof skipped!
        return call_stack_item == other.call_stack_item &&
               public_call_stack_preimages == other.public_call_stack_preimages &&
               portal_contract_address == other.portal_contract_address && bytecode_hash == other.bytecode_hash;
    };

    // WARNING: the `proof` does NOT get converted! (because the current implementation of `verify_proof` takes a proof
    // of native bytes; any conversion to circuit types happens within the `verify_proof` function)
    template <typename Builder> PublicCallData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        // typedef CircuitTypes<Builder> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        PublicCallData<CircuitTypes<Builder>> data = {
            call_stack_item.to_circuit_type(builder),

            map(public_call_stack_preimages, to_circuit_type),

            proof,  // Notice: not converted! Stays as native. This is because of how the verify_proof function
                    // currently works.
            // CT::VK::from_witness(&builder, vk),

            // to_circuit_type(function_leaf_membership_witness),
            // to_circuit_type(contract_leaf_membership_witness),

            to_ct(portal_contract_address),
            to_ct(bytecode_hash),
        };

        return data;
    };
};

}  // namespace aztec3::circuits::abis::public_kernel
