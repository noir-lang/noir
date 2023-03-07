#pragma once

#include <aztec3/circuits/apps/contract.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>
#include <aztec3/circuits/apps/notes/default_private_note/note.hpp>
#include <aztec3/circuits/apps/state_vars/mapping_state_var.hpp>
#include <aztec3/circuits/apps/state_vars/utxo_set_state_var.hpp>

#include <aztec3/circuits/apps/oracle_wrapper.hpp>
#include <aztec3/oracle/oracle.hpp>

#include <stdlib/types/types.hpp>
#include <stdlib/types/convert.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/native_types.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using C = plonk::stdlib::types::Composer;

using CT = plonk::stdlib::types::CircuitTypes<C>;
using NT = plonk::stdlib::types::NativeTypes;

using DB = oracle::FakeDB;
using oracle::NativeOracle;
using OracleWrapper = apps::OracleWrapperInterface<C>;

using Contract = apps::Contract<NT>;
using FunctionExecutionContext = apps::FunctionExecutionContext<C>;

using plonk::stdlib::types::to_ct;

// StateVars
using apps::state_vars::MappingStateVar;
using apps::state_vars::UTXOSetStateVar;

// Get rid of ugle `Composer` template arg from our state var types:
template <typename T> struct SpecialisedTypes {
    typedef MappingStateVar<C, T> mapping;
    typedef UTXOSetStateVar<C, T> utxo_set;
};

template <typename V> using Mapping = typename SpecialisedTypes<V>::mapping;
template <typename Note> using UTXOSet = typename SpecialisedTypes<Note>::utxo_set;

using DefaultNote = apps::notes::DefaultPrivateNote<C, CT::fr>;

} // namespace aztec3::circuits::apps::test_apps::escrow