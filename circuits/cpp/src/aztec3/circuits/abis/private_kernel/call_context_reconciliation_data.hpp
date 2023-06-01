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
//     std::array<CallContext<NCT>, PRIVATE_CALL_STACK_LENGTH> private_call_contexts;
//     std::array<fr, PRIVATE_CALL_STACK_LENGTH> private_counterparts;

//     std::array<CallContext<NCT>, PUBLIC_CALL_STACK_LENGTH> public_call_contexts;
//     std::array<fr, PUBLIC_CALL_STACK_LENGTH> public_counterparts;

//     // std::array<fr, CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH> contract_deployment_call_stack;

//     std::array<CallContext<NCT>, NEW_L2_TO_L1_MSGS_LENGTH> l1_call_contexts;
//     std::array<fr, NEW_L2_TO_L1_MSGS_LENGTH> l1_counterparts; // TODO: this is probably wrong.

//     template <typename Composer>
//     CallContextReconciliationData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
//     {
//         static_assert((std::is_same<NativeTypes, NCT>::value));

//         // Capture the composer:
//         auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
//         auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

//         CallContextReconciliationData<CircuitTypes<Composer>> data = {

//             map(private_call_contexts, to_circuit_type), to_ct(private_counterparts),

//             map(public_call_contexts, to_circuit_type),  to_ct(public_counterparts),

//             map(l1_call_contexts, to_circuit_type),      to_ct(l1_counterparts),
//         };

//         return data;
//     };
// };

// } // namespace aztec3::circuits::abis::private_kernel