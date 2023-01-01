// #include "contract.hpp"
// #include "function1.hpp"
// #include <aztec3/circuits/apps/private_state_note.hpp>
// #include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

// namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

// using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

// OptionalPrivateCircuitPublicInputs<NT> function1(FunctionExecutionContext<Composer>& exec_ctx,
//                                                  NT::fr const& _d,
//                                                  NT::fr const& _e,
//                                                  NT::fr const& _f)
// {
//     CT::fr d = to_ct(composer, _d);
//     CT::fr e = to_ct(composer, _e);
//     CT::fr f = to_ct(composer, _f);

//     CT::address msg_sender = oracle.get_msg_sender();

//     auto env = init(composer, oracle);

//     auto& y = env.get_private_state_var("y");

//     auto product = d * e * f;

//     y.add({
//         .value = product,
//         .owner = msg_sender,
//         .creator_address = msg_sender,
//         .memo = 0,
//     });

//     auto& public_inputs = env.private_circuit_public_inputs;

//     public_inputs.args[0] = d;
//     public_inputs.args[1] = e;
//     public_inputs.args[2] = f;

//     public_inputs.return_values[0] = product;

//     env.finalise();

//     info("public inputs: ", public_inputs);

//     return public_inputs.to_native_type<Composer>();
//     // TODO: also return note preimages and nullifier preimages.
// };

// } // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call