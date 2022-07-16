// #include "contract.hpp"
// #include "function1.hpp"
// #include <aztec3/circuits/apps/private_state_note.hpp>
// #include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

// namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

// using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

// OptionalPrivateCircuitPublicInputs<NT> function1(
//     Composer& composer, OracleWrapper& oracle, NT::fr const& _a, NT::fr const& _b, NT::fr const& _c)
// {
//     CT::fr a = to_ct(composer, _a);
//     CT::fr b = to_ct(composer, _b);
//     CT::fr c = to_ct(composer, _c);

//     CT::address msg_sender = oracle.get_msg_sender();

//     auto env = init(composer, oracle);

//     auto& x = env.get_private_state("x");

//     x.add({
//         .value = a,
//         .owner_address = msg_sender,
//         .creator_address = msg_sender,
//         .memo = 0,
//     });

//     auto function2 = env.get_function("function2");

//     auto result = function2.call(a, b, c);

//     auto& public_inputs = env.private_circuit_public_inputs;

//     public_inputs.custom_inputs[0] = a;
//     public_inputs.custom_inputs[1] = b;
//     public_inputs.custom_inputs[2] = c;

//     public_inputs.private_call_stack[0] = ...

//                                           env.finalise();

//     info("public inputs: ", public_inputs);

//     return public_inputs.to_native_type<Composer>();
//     // TODO: also return note preimages and nullifier preimages.
// };

// } // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call