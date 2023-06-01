#pragma once

#include "contract.hpp"
#include "init.hpp"

#include "aztec3/circuits/apps/contract.hpp"
#include "aztec3/circuits/apps/function_declaration.hpp"

namespace aztec3::circuits::apps::test_apps::basic_contract_deployment {

inline Contract init_contract()
{
    Contract contract("BasicContractDeployment");

    // Solely used for assigning vk indices.
    contract.set_functions({
        { .name = "constructor", .is_private = true, .is_constructor = true },
    });

    return contract;
}

}  // namespace aztec3::circuits::apps::test_apps::basic_contract_deployment