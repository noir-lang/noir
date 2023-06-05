#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/apps/function_execution_context.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> deposit(FunctionExecutionContext& exec_ctx, std::vector<NT::fr> const& args);

}  // namespace aztec3::circuits::apps::test_apps::escrow