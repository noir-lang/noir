#pragma once

#include "call_context_reconciliation_data.hpp"
#include "../call_stack_item.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PrivateCallData {
    typedef typename NCT::address address;
    typedef typename NCT::fr fr;
    typedef typename NCT::VK VK;

    CallStackItem<NCT, CallType::Private> call_stack_item;
    CallContextReconciliationData<NCT> call_context_reconciliation_data;

    NativeTypes::Proof proof; // TODO: how to express proof as native/circuit type when it gets used as a buffer?
    std::shared_ptr<VK> vk;
    std::array<fr, VK_TREE_HEIGHT> vk_path;

    fr contract_tree_root;
    fr contract_leaf_index;
    std::array<fr, CONTRACT_TREE_HEIGHT> contract_path;

    fr portal_contract_address; // an ETH address

    // WARNING: the `proof` does NOT get converted!
    template <typename Composer> PrivateCallData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        typedef CircuitTypes<Composer> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        PrivateCallData<CircuitTypes<Composer>> data = {
            call_stack_item.to_circuit_type(composer),
            call_context_reconciliation_data.to_circuit_type(composer),

            proof, // Notice: not converted! Stays as native. This is because of how the verify_proof function currently
                   // works.
            CT::VK::from_witness(&composer, vk),
            to_ct(vk_path),

            to_ct(contract_tree_root),
            to_ct(contract_leaf_index),
            to_ct(contract_path),

            to_ct(portal_contract_address),
        };

        return data;
    };
};

} // namespace aztec3::circuits::abis::private_kernel