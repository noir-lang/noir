#pragma once

#include "call_context_reconciliation_data.hpp"
#include "../call_stack_item.hpp"
#include "../read_request_membership_witness.hpp"
#include "../types.hpp"

#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PrivateCallData {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;
    using VK = typename NCT::VK;

    CallStackItem<NCT, PrivateTypes> call_stack_item{};

    std::array<CallStackItem<NCT, PrivateTypes>, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> private_call_stack_preimages{};

    // std::array<CallStackItem<NCT, CallType::Public>, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>
    // public_call_stack_preimages;

    NativeTypes::Proof proof{};  // TODO: how to express proof as native/circuit type when it gets used as a buffer?
    std::shared_ptr<VK> vk;

    MembershipWitness<NCT, FUNCTION_TREE_HEIGHT> function_leaf_membership_witness{};
    MembershipWitness<NCT, CONTRACT_TREE_HEIGHT> contract_leaf_membership_witness{};

    std::array<ReadRequestMembershipWitness<NCT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL>
        read_request_membership_witnesses{};

    fr portal_contract_address = 0;  // an ETH address
    fr acir_hash = 0;

    boolean operator==(PrivateCallData<NCT> const& other) const
    {
        // WARNING: proof skipped!
        return call_stack_item == other.call_stack_item &&
               private_call_stack_preimages == other.private_call_stack_preimages && vk == other.vk &&
               function_leaf_membership_witness == other.function_leaf_membership_witness &&
               contract_leaf_membership_witness == other.contract_leaf_membership_witness &&
               read_request_membership_witnesses == other.read_request_membership_witnesses &&
               portal_contract_address == other.portal_contract_address && acir_hash == other.acir_hash;
    };

    // WARNING: the `proof` does NOT get converted! (because the current implementation of `verify_proof` takes a proof
    // of native bytes; any conversion to circuit types happens within the `verify_proof` function)
    template <typename Builder> PrivateCallData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        typedef CircuitTypes<Builder> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        PrivateCallData<CircuitTypes<Builder>> data = {
            to_circuit_type(call_stack_item),

            map(private_call_stack_preimages, to_circuit_type),

            proof,  // Notice: not converted! Stays as native. This is because of how the verify_proof function
                    // currently works.
            CT::VK::from_witness(&builder, vk),

            to_circuit_type(function_leaf_membership_witness),
            to_circuit_type(contract_leaf_membership_witness),

            aztec3::utils::types::to_ct<Builder, ReadRequestMembershipWitness<CT, PRIVATE_DATA_TREE_HEIGHT>>(
                builder, read_request_membership_witnesses),

            to_ct(portal_contract_address),
            to_ct(acir_hash),
        };

        return data;
    };
};  // namespace aztec3::circuits::abis::private_kernel

template <typename NCT> void read(uint8_t const*& it, PrivateCallData<NCT>& obj)
{
    using serialize::read;

    read(it, obj.call_stack_item);
    read(it, obj.private_call_stack_preimages);
    read(it, obj.proof);
    read(it, obj.vk);
    read(it, obj.function_leaf_membership_witness);
    read(it, obj.contract_leaf_membership_witness);
    read(it, obj.read_request_membership_witnesses);
    read(it, obj.portal_contract_address);
    read(it, obj.acir_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PrivateCallData<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.call_stack_item);
    write(buf, obj.private_call_stack_preimages);
    write(buf, obj.proof);
    write(buf, *obj.vk);
    write(buf, obj.function_leaf_membership_witness);
    write(buf, obj.contract_leaf_membership_witness);
    write(buf, obj.read_request_membership_witnesses);
    write(buf, obj.portal_contract_address);
    write(buf, obj.acir_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PrivateCallData<NCT> const& obj)
{
    return os << "call_stack_item:\n"
              << obj.call_stack_item << "\n"
              << "private_call_stack_preimages:\n"
              << obj.private_call_stack_preimages << "\n"
              << "proof:\n"
              << obj.proof << "\n"
              << "vk:\n"
              << *(obj.vk) << "\n"
              << "function_leaf_membership_witness:\n"
              << obj.function_leaf_membership_witness << "\n"
              << "contract_leaf_membership_witness:\n"
              << obj.contract_leaf_membership_witness << "\n"
              << "read_request_membership_witnesses:\n"
              << obj.read_request_membership_witnesses << "\n"
              << "portal_contract_address: " << obj.portal_contract_address << "\n"
              << "acir_hash: " << obj.acir_hash << "\n";
}

}  // namespace aztec3::circuits::abis::private_kernel
