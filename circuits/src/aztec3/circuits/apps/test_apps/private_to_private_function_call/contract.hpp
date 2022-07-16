// #pragma once
// #include "init.hpp"
// #include <aztec3/circuits/apps/contract_factory.hpp>
// #include <aztec3/circuits/apps/function.hpp>

// namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

// inline ContractFactory<Composer> init(Composer& composer, OracleWrapper& oracle)
// {
//     ContractFactory<Composer> contract(composer, oracle, "priv_to_priv_function_call");

//     contract.new_private_state("x");
//     contract.new_private_state("y");

//     // Solely used for assigning vk indices.
//     contract.set_functions({
//         { .name = "function1", .is_private = true },
//         { .name = "function2", .is_private = true },
//     });

//     return contract;
// }

// } // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call