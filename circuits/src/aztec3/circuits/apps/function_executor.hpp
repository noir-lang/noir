// #pragma once
// #include <common/container.hpp>
// #include <aztec3/constants.hpp>
// #include <stdlib/types/convert.hpp>
// #include <aztec3/circuits/abis/function_signature.hpp>
// #include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
// #include "private_state_note.hpp"
// #include "private_state_var.hpp"
// #include "function.hpp"
// #include "l1_function_interface.hpp"
// #include "oracle_wrapper.hpp"

// namespace aztec3::circuits::apps {

// using plonk::stdlib::witness_t;
// using plonk::stdlib::types::CircuitTypes;
// using NT = plonk::stdlib::types::NativeTypes;
// using aztec3::circuits::abis::FunctionSignature;
// using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

// template <typename Composer> class FunctionExecutor {
//     typedef CircuitTypes<Composer> CT;
//     typedef typename CT::fr fr;

//   public:
//     Composer& composer;
//     OracleWrapperInterface<Composer>& oracle;
//     ContractFactory<Composer>& contract_factory;
//     PrivateStateFactory<Composer>& private_state_factory;
//     OptionalPrivateCircuitPublicInputs<CT> private_circuit_public_inputs;
//     // UnpackedData<CT> unpacked_data;

//     FunctionExecutor<Composer>(ContractFactory<Composer>& contract_factory)
//         : composer(contract_factory.composer)
//         , oracle(contract_factory.oracle)
//         , contract_factory(contract_factory)
//         , private_state_factory(contract_factory.private_state_factory)
//         , private_circuit_public_inputs(OptionalPrivateCircuitPublicInputs<CT>::create())
//     {
//         private_circuit_public_inputs.call_context = oracle.get_call_context();
//     }

//     void finalise()
//     {
//         private_state_factory.finalise();
//         private_circuit_public_inputs.set_commitments(private_state_factory.commitments);
//         private_circuit_public_inputs.set_nullifiers(private_state_factory.nullifiers);
//         private_circuit_public_inputs.set_public(composer);
//     };
// };

// } // namespace aztec3::circuits::apps