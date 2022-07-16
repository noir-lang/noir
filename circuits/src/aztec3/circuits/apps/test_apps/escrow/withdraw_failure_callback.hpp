#pragma once
#include <aztec3/circuits/apps/function_executor.hpp>
#include "init.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

void withdraw_failure_callback(FunctionExecutionContext<Composer>& exec_ctx,
                               NT::fr const& _asset_id,
                               NT::fr const& _amount,
                               NT::address const& _owner_address,
                               NT::fr const& _memo);

} // namespace aztec3::circuits::apps::test_apps::escrow