#pragma once

#include "aztec3/circuits/apps/contract.hpp"
#include "aztec3/circuits/apps/function_execution_context.hpp"
#include "aztec3/circuits/apps/notes/default_private_note/note.hpp"
#include "aztec3/circuits/apps/oracle_wrapper.hpp"
#include "aztec3/circuits/apps/state_vars/mapping_state_var.hpp"
#include "aztec3/circuits/apps/state_vars/utxo_set_state_var.hpp"
#include "aztec3/oracle/oracle.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::apps::test_apps::basic_contract_deployment {

// Builder
using C = UltraCircuitBuilder;

// Native and circuit types
using CT = aztec3::utils::types::CircuitTypes<C>;
using NT = aztec3::utils::types::NativeTypes;

// Database types
using DB = oracle::FakeDB;
using oracle::NativeOracle;
using OracleWrapper = apps::OracleWrapperInterface<C>;

using Contract = apps::Contract<NT>;
using FunctionExecutionContext = apps::FunctionExecutionContext<C>;

using aztec3::utils::types::to_ct;

// StateVars
using apps::state_vars::MappingStateVar;
using apps::state_vars::UTXOSetStateVar;

// Get rid of ugly `Builder` template arg from our state var types:
template <typename V> using Mapping = MappingStateVar<C, V>;
template <typename Note> using UTXOSet = UTXOSetStateVar<C, Note>;

using DefaultNote = apps::notes::DefaultPrivateNote<C, CT::fr>;

}  // namespace aztec3::circuits::apps::test_apps::basic_contract_deployment