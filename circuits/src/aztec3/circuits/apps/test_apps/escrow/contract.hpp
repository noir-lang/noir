#pragma once

#include "init.hpp"

#include <aztec3/circuits/apps/contract.hpp>
#include <aztec3/circuits/apps/function_declaration.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

inline Contract init_contract(FunctionExecutionContext& exec_ctx)
{
    Contract contract(exec_ctx, "Escrow");

    contract.declare_state_var("balances");

    // Solely used for assigning vk indices.
    contract.set_functions({
        { .name = "deposit", .is_private = true },
        { .name = "transfer", .is_private = true },
        { .name = "withdraw", .is_private = true },
    });

    // TODO: this L1 declaration interface is just to get something working.
    contract.import_l1_function({
        .function_name = "withdraw",
        .function_selector = 12345,
        .num_params = 3,
    });

    return contract;
}

} // namespace aztec3::circuits::apps::test_apps::escrow