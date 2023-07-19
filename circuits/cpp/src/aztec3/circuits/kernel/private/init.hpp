#pragma once
#include "aztec3/circuits/apps/function_execution_context.hpp"
#include "aztec3/circuits/apps/oracle_wrapper.hpp"
#include "aztec3/circuits/recursion/aggregator.hpp"
#include "aztec3/oracle/oracle.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::kernel::private_kernel {

using Builder = UltraCircuitBuilder;

using Aggregator = aztec3::circuits::recursion::Aggregator;

// Generic:
using CT = aztec3::utils::types::CircuitTypes<Builder>;
using NT = aztec3::utils::types::NativeTypes;
using aztec3::utils::types::to_ct;

using DB = oracle::FakeDB;
using oracle::NativeOracle;
using OracleWrapper = aztec3::circuits::apps::OracleWrapperInterface<Builder>;

using FunctionExecutionContext = aztec3::circuits::apps::FunctionExecutionContext<Builder>;

// Used when calling library functions like `psuh_array` which have their own generic error code.
// So we pad this in front of the error message to identify where the error originally came from.
const std::string PRIVATE_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING = "private_kernel_circuit: ";

}  // namespace aztec3::circuits::kernel::private_kernel