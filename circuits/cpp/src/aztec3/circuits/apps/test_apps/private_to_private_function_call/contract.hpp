#pragma once

#include "init.hpp"

#include <aztec3/circuits/apps/contract.hpp>
#include <aztec3/circuits/apps/function_declaration.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

Contract init_contract_2();

inline Contract init_contract_1()
{
    Contract contract_1("contract_1");

    contract_1.declare_state_var("x");

    // Solely used for assigning vk indices.
    contract_1.set_functions({
        { .name = "function_1_1", .is_private = true },
    });

    contract_1.import_contracts({ std::make_pair("contract_2", init_contract_2()) });

    return contract_1;
}

inline Contract init_contract_2()
{
    Contract contract_2("contract_2");

    contract_2.declare_state_var("y");

    // Solely used for assigning vk indices.
    contract_2.set_functions({
        { .name = "function_2_1", .is_private = true },
    });

    return contract_2;
}

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call