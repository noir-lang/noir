#pragma once

#include "init.hpp"

#include <aztec3/circuits/apps/contract.hpp>
#include <aztec3/circuits/apps/function_declaration.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

inline Contract init_contract()
{
    Contract contract("Escrow");

    contract.declare_state_var("balances");

    // Solely used for assigning vk indices.
    contract.set_functions({
        { .name = "deposit", .is_private = true },
        { .name = "transfer", .is_private = true },
        { .name = "withdraw", .is_private = true },
    });

    return contract;
}

}  // namespace aztec3::circuits::apps::test_apps::escrow