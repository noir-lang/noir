#pragma once
#include <stdlib/types/turbo.hpp>
#include <stdlib/types/convert.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/native_types.hpp>
#include <aztec3/oracle/oracle.hpp>
#include <aztec3/circuits/apps/function_executor.hpp>
#include <aztec3/circuits/apps/oracle_wrapper.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using Composer = plonk::stdlib::types::turbo::Composer;
using CT = plonk::stdlib::types::CircuitTypes<Composer>;
using NT = plonk::stdlib::types::NativeTypes;

using DB = oracle::FakeDB;
using oracle::NativeOracle;
using OracleWrapper = OracleWrapperInterface<Composer>;

using plonk::stdlib::types::to_ct;

} // namespace aztec3::circuits::apps::test_apps::escrow