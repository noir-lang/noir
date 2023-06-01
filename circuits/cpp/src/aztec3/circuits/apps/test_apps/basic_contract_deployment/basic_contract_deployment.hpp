#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/apps/function_execution_context.hpp"

namespace aztec3::circuits::apps::test_apps::basic_contract_deployment {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> constructor(FunctionExecutionContext& exec_ctx,
                                                   std::array<NT::fr, ARGS_LENGTH> const& args);

}  // namespace aztec3::circuits::apps::test_apps::basic_contract_deployment