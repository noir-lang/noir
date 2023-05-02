#pragma once

#include "init.hpp"

#include <aztec3/circuits/apps/function_execution_context.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

void function_1_1(FunctionExecutionContext& exec_ctx, std::array<NT::fr, ARGS_LENGTH> const& _args);

}  // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call