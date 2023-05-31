#pragma once
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/circuits/apps/oracle_wrapper.hpp>
#include <aztec3/circuits/recursion/aggregator.hpp>
#include <aztec3/oracle/oracle.hpp>
#include <aztec3/utils/dummy_composer.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

namespace aztec3::circuits::kernel::public_kernel {

using Composer = plonk::UltraPlonkComposer;

using Aggregator = aztec3::circuits::recursion::Aggregator;

// Generic:
using CT = aztec3::utils::types::CircuitTypes<Composer>;
using NT = aztec3::utils::types::NativeTypes;
using aztec3::utils::types::to_ct;
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;

using DB = oracle::FakeDB;
using oracle::NativeOracle;
using OracleWrapper = aztec3::circuits::apps::OracleWrapperInterface<Composer>;

}  // namespace aztec3::circuits::kernel::public_kernel