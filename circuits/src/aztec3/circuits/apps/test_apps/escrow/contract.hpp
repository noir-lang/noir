#pragma once
#include "init.hpp"
#include <aztec3/circuits/apps/contract_factory.hpp>
#include <aztec3/circuits/apps/function.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

inline ContractFactory<Composer> init(Composer& composer, OracleWrapper& oracle)
{
    ContractFactory<Composer> contract(composer, oracle, "Escrow");

    contract.new_private_state("balances", { "owner", "asset_id" });

    // Solely used for assigning vk indices.
    contract.set_functions({
        { .name = "deposit", .is_private = true },
        { .name = "transfer", .is_private = true },
        { .name = "withdraw", .is_private = true },
        // not needed, but it's helping me figure out success cases:
        { .name = "withdraw_success_callback", .is_private = true },
        { .name = "withdraw_failure_callback", .is_private = true },
    });

    contract.import_l1_function({
        .function_name = "withdraw",
        .function_selector = 12345,
        .num_params = 3,
    });

    return contract;
}

} // namespace aztec3::circuits::apps::test_apps::escrow