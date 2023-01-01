#pragma once

#include "init.hpp"

#include <aztec3/circuits/apps/contract.hpp>
#include <aztec3/circuits/apps/function_declaration.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

inline Contract<Composer> init_contract(FunctionExecutionContext<Composer>& exec_ctx)
{
    Contract<Composer> contract(exec_ctx, "Escrow");

    // mapping(owner => mapping(asset_id => balance)) balances;
    contract.declare_private_state_var("balances", { "owner", "asset_id" });

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

    // TODO: create a new FunctionExecutor(contract) and return it.
    return contract;
}

} // namespace aztec3::circuits::apps::test_apps::escrow