#pragma once
#include <aztec3/circuits/apps/contract_factory.hpp>
#include <aztec3/circuits/apps/function_declaration.hpp>
#include <aztec3/circuits/apps/function_executor.hpp>
#include "init.hpp"

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

inline Contract<Composer> init_contract(FunctionExecutionContext<Composer>& exec_ctx)
{
    Contract<Composer> contract(exec_ctx, "priv_to_priv_function_call");

    contract.new_private_state("x");
    contract.new_private_state("y");

    // Solely used for assigning vk indices.
    contract.set_functions({
        { .name = "function1", .is_private = true },
        { .name = "function2", .is_private = true },
    });

    return contract;
}

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call