#pragma once

#include "init.hpp"

#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

void function_2_1(FunctionExecutionContext& exec_ctx, std::array<NT::fr, ARGS_LENGTH> const& _args);

}  // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call