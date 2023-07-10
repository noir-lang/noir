// #pragma once

// #include "../call_context.hpp"

// #include "aztec3/utils/types/native_types.hpp"
// #include "aztec3/utils/types/circuit_types.hpp"
// #include "aztec3/utils/types/convert.hpp"

// #include <barretenberg/barretenberg.hpp>

// namespace aztec3::circuits::abis::private_kernel {

// using plonk::stdlib::witness_t;
// using aztec3::utils::types::CircuitTypes;
// using aztec3::utils::types::NativeTypes;
// using std::is_same;

// template <typename NCT> struct CallContextReconciliationData {
//     typedef typename NCT::fr fr;

//     /**
//      * This class needs an explanation...
//      *
//      */
//     std::array<CallContext<NCT>, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> private_call_contexts;
//     std::array<fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> private_counterparts;

//     std::array<CallContext<NCT>, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> public_call_contexts;
//     std::array<fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> public_counterparts;

//     // std::array<fr, CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH> contract_deployment_call_stack;

//     std::array<CallContext<NCT>, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> l1_call_contexts;
//     std::array<fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> l1_counterparts; // TODO: this is probably wrong.

//     template <typename Builder>
//     CallContextReconciliationData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
//     {
//         static_assert((std::is_same<NativeTypes, NCT>::value));

//         // Capture the circuit builder:
//         auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
//         auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

//         CallContextReconciliationData<CircuitTypes<Builder>> data = {

//             map(private_call_contexts, to_circuit_type), to_ct(private_counterparts),

//             map(public_call_contexts, to_circuit_type),  to_ct(public_counterparts),

//             map(l1_call_contexts, to_circuit_type),      to_ct(l1_counterparts),
//         };

//         return data;
//     };
// };

// } // namespace aztec3::circuits::abis::private_kernel