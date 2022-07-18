#pragma once
#include "init.hpp"
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/apps/function_executor.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> function1(FunctionExecutionContext<Composer>& exec_ctx,
                                                 NT::fr const& _a,
                                                 NT::fr const& _b,
                                                 NT::fr const& _c);

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call