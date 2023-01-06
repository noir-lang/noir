// #pragma once

// #include "init.hpp"

// #include <aztec3/circuits/apps/contract.hpp>
// #include <aztec3/circuits/apps/function_declaration.hpp>
// #include <aztec3/circuits/apps/function_execution_context.hpp>

// namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

// inline Contract init_contract_1(FunctionExecutionContext& exec_ctx)
// {
//     Contract contract(exec_ctx, "contract_1");

//     contract.declare_state_var("x");
//     contract.declare_state_var("y");

//     // Solely used for assigning vk indices.
//     contract.set_functions({
//         { .name = "function_1_1", .is_private = true },
//     });

//     return contract;
// }

// inline Contract init_contract_2(FunctionExecutionContext& exec_ctx)
// {
//     Contract contract(exec_ctx, "contract_2");

//     contract.declare_state_var("x");
//     contract.declare_state_var("y");

//     // Solely used for assigning vk indices.
//     contract.set_functions({
//         { .name = "function_2_1", .is_private = true },
//     });

//     return contract;
// }

// } // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call