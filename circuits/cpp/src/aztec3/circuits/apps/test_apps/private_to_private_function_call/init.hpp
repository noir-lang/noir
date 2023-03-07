#pragma once

#include <aztec3/circuits/apps/contract.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>
#include <aztec3/circuits/apps/notes/default_singleton_private_note/note.hpp>
#include <aztec3/circuits/apps/oracle_wrapper.hpp>
#include <aztec3/circuits/apps/state_vars/utxo_state_var.hpp>

#include <aztec3/oracle/oracle.hpp>

#include <stdlib/types/types.hpp>
#include <stdlib/types/convert.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/native_types.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

using C = plonk::stdlib::types::Composer;
using CT = plonk::stdlib::types::CircuitTypes<C>;
using NT = plonk::stdlib::types::NativeTypes;

using DB = oracle::FakeDB;
using oracle::NativeOracle;
using OracleWrapper = OracleWrapperInterface<C>;

using Contract = apps::Contract<NT>;
using FunctionExecutionContext = apps::FunctionExecutionContext<C>;

using plonk::stdlib::types::to_ct;

// StateVars
using apps::state_vars::UTXOStateVar;

// Get rid of ugle `Composer` template arg from our state var types:
template <typename T> struct SpecialisedTypes {
    typedef UTXOStateVar<C, T> utxo;
};

template <typename Note> using UTXO = typename SpecialisedTypes<Note>::utxo;

using Note = apps::notes::DefaultSingletonPrivateNote<C, CT::fr>;

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call