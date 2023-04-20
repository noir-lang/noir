#pragma once

#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"
#include "../call_stack_item.hpp"
#include "../types.hpp"

#include <barretenberg/common/map.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::public_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct PublicCallData {
    typedef typename NCT::address address;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;
    typedef typename NCT::VK VK;

    CallStackItem<NCT, PublicTypes> call_stack_item{};

    std::array<CallStackItem<NCT, PublicTypes>, PUBLIC_CALL_STACK_LENGTH> public_call_stack_preimages{};

    NativeTypes::Proof proof{}; // TODO: how to express proof as native/circuit type when it gets used as a buffer?

    fr portal_contract_address = 0; // an ETH address
    fr bytecode_hash = 0;

    boolean operator==(PublicCallData<NCT> const& other) const
    {
        // WARNING: proof skipped!
        return call_stack_item == other.call_stack_item &&
               public_call_stack_preimages == other.public_call_stack_preimages &&
               portal_contract_address == other.portal_contract_address && bytecode_hash == other.bytecode_hash;
    };

    // WARNING: the `proof` does NOT get converted! (because the current implementation of `verify_proof` takes a proof
    // of native bytes; any conversion to circuit types happens within the `verify_proof` function)
    template <typename Composer> PublicCallData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        // typedef CircuitTypes<Composer> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        PublicCallData<CircuitTypes<Composer>> data = {
            call_stack_item.to_circuit_type(composer),

            map(public_call_stack_preimages, to_circuit_type),

            proof, // Notice: not converted! Stays as native. This is because of how the verify_proof function
                   // currently works.
            // CT::VK::from_witness(&composer, vk),

            // to_circuit_type(function_leaf_membership_witness),
            // to_circuit_type(contract_leaf_membership_witness),

            to_ct(portal_contract_address),
            to_ct(bytecode_hash),
        };

        return data;
    };
};

template <typename NCT> void read(uint8_t const*& it, PublicCallData<NCT>& obj)
{
    using serialize::read;

    read(it, obj.call_stack_item);
    read(it, obj.private_call_stack_preimages);
    read(it, obj.proof);
    read(it, obj.vk);
    read(it, obj.function_leaf_membership_witness);
    read(it, obj.contract_leaf_membership_witness);
    read(it, obj.portal_contract_address);
    read(it, obj.acir_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PublicCallData<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.call_stack_item);
    write(buf, obj.private_call_stack_preimages);
    write(buf, obj.proof);
    write(buf, *obj.vk);
    write(buf, obj.function_leaf_membership_witness);
    write(buf, obj.contract_leaf_membership_witness);
    write(buf, obj.portal_contract_address);
    write(buf, obj.acir_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PublicCallData<NCT> const& obj)
{
    return os << "call_stack_item:\n"
              << obj.call_stack_item << "\n"
              << "public_call_stack_preimages:\n"
              << obj.public_call_stack_preimages << "\n"
              << "proof:\n"
              << obj.proof
              << "\n"
              //<< "vk:\n"
              //<< *(obj.vk) << "\n"
              //<< "function_leaf_membership_witness:\n"
              //<< obj.function_leaf_membership_witness << "\n"
              //<< "contract_leaf_membership_witness:\n"
              //<< obj.contract_leaf_membership_witness << "\n"
              << "portal_contract_address: " << obj.portal_contract_address << "\n"
              << "bytecode_hash: " << obj.bytecode_hash << "\n";
}

} // namespace aztec3::circuits::abis::public_kernel