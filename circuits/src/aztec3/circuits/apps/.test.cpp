
// #include <common/serialize.hpp>
// #include <stdlib/types/turbo.hpp>
// #include <numeric/random/engine.hpp>

#include <gtest/gtest.h>
#include <common/test.hpp>

// #include "utxo_state_var.hpp"

#include "contract.hpp"
#include "function_execution_context.hpp"
#include "oracle_wrapper.hpp"
#include "utxo_datum.hpp"

#include "notes/note_interface.hpp"
#include "notes/default_private_note/note.hpp"
#include "notes/default_private_note/note_preimage.hpp"

#include "state_vars/field_state_var.hpp"
#include "state_vars/mapping_state_var.hpp"
#include "state_vars/utxo_state_var.hpp"
#include "state_vars/utxo_set_state_var.hpp"

#include <aztec3/oracle/oracle.hpp>

#include <stdlib/types/convert.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/turbo.hpp>

namespace {
// Composer
using C = plonk::stdlib::types::turbo::Composer;

// Types
using CT = plonk::stdlib::types::CircuitTypes<C>;
using NT = plonk::stdlib::types::NativeTypes;
using plonk::stdlib::types::to_ct;

// exec_ctx
// using aztec3::circuits::apps::FunctionExecutionContext;

// Oracle
using DB = aztec3::oracle::FakeDB;
using aztec3::oracle::NativeOracle;
using OracleWrapper = aztec3::circuits::apps::OracleWrapperInterface<C>;

// Contract
using Contract = aztec3::circuits::apps::Contract<C>;

// StateVars
using aztec3::circuits::apps::state_vars::FieldStateVar;
using aztec3::circuits::apps::state_vars::MappingStateVar;
using aztec3::circuits::apps::state_vars::UTXOSetStateVar;
using aztec3::circuits::apps::state_vars::UTXOStateVar;

using aztec3::circuits::apps::notes::DefaultPrivateNote;
using aztec3::circuits::apps::notes::NoteInterface;
// using aztec3::circuits::apps::notes::DefaultPrivateNotePreimage;

//********
// Get rid of ugle `Composer` template arg from our state var types:
template <typename T> struct SpecialisedTypes {
    typedef MappingStateVar<C, T> mapping;
    typedef UTXOStateVar<C, T> utxo;
    typedef UTXOSetStateVar<C, T> utxo_set;
};

template <typename V> using Mapping = typename SpecialisedTypes<V>::mapping;

template <typename Note> using UTXO = typename SpecialisedTypes<Note>::utxo;
template <typename Note> using UTXOSet = typename SpecialisedTypes<Note>::utxo_set;

using Field = FieldStateVar<C>;

//********

} // namespace

namespace aztec3::circuits::apps {

class state_var_tests : public ::testing::Test {
  protected:
    NativeOracle get_test_native_oracle()
    {
        DB db;
        // No cheating: you have to grab this stuff from the oracle in your tests - hence the 'private' scope.
        NT::fr msg_sender_private_key = 123456789;
        NT::address contract_address = 12345;
        NT::address msg_sender = NT::fr(
            uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
        NT::address tx_origin = msg_sender;

        return NativeOracle(db, contract_address, msg_sender, tx_origin, msg_sender_private_key);
    };
};

TEST_F(state_var_tests, mapping)
{
    C composer;
    NativeOracle native_oracle = get_test_native_oracle();
    OracleWrapper oracle = OracleWrapper(composer, native_oracle);
    FunctionExecutionContext<C> exec_ctx(composer, oracle);

    // TODO:
    // Interestingly, if I scope the below, the debugger works, but running the test via the command line fails. I
    // reckon the contract (and crucially, all pointers to the contract which are stored in other classes) is being
    // deleted... so the declaration of this contract and any pointers probably all need to be shared_ptr<Contract>.
    // {
    Contract contract(exec_ctx, "TestContract");
    contract.declare_state_var("my_mapping");
    // }

    Mapping<Field> my_mapping(&exec_ctx, "my_mapping");

    my_mapping[5] = to_ct(composer, NT::fr(5));

    info("my_mapping[5]: ", my_mapping[5]);
    info("my_mapping[5].start_slot: ", my_mapping[5].start_slot);
    info("my_mapping[5].storage_slot_point: ", my_mapping[5].storage_slot_point);
}

TEST_F(state_var_tests, mapping_within_mapping)
{
    C composer;
    NativeOracle native_oracle = get_test_native_oracle();
    OracleWrapper oracle = OracleWrapper(composer, native_oracle);
    FunctionExecutionContext<C> exec_ctx(composer, oracle);

    // {
    Contract contract(exec_ctx, "TestContract");
    contract.declare_state_var("my_mapping");
    // }

    Mapping<Mapping<Field>> my_mapping(&exec_ctx, "my_mapping");

    my_mapping[5][6] = 7;

    info(my_mapping[5][6].state_var_name, ": ", my_mapping[5][6]);
}

TEST_F(state_var_tests, partial_mapping)
{
    C composer;
    NativeOracle native_oracle = get_test_native_oracle();
    OracleWrapper oracle = OracleWrapper(composer, native_oracle);
    FunctionExecutionContext<C> exec_ctx(composer, oracle);

    // {
    Contract contract(exec_ctx, "TestContract");
    contract.declare_state_var("my_mapping");
    // }

    Mapping<Mapping<Field>> my_mapping(&exec_ctx, "my_mapping");

    my_mapping["?"][6] = 7;

    info(my_mapping["?"][6].state_var_name, ": ", my_mapping["?"][6]);
}

TEST_F(state_var_tests, utxo_of_default_private_note_fr)
{
    C composer;
    NativeOracle native_oracle = get_test_native_oracle();
    OracleWrapper oracle = OracleWrapper(composer, native_oracle);
    FunctionExecutionContext<C> exec_ctx(composer, oracle);

    Contract contract(exec_ctx, "TestContract");
    contract.declare_state_var("my_utxo");

    // FUNCTION:

    using Note = DefaultPrivateNote<C, CT::fr>;

    UTXO<Note> my_utxo(&exec_ctx, "my_utxo");

    const auto& msg_sender = oracle.get_msg_sender();

    Note old_note = my_utxo.get({ .owner = msg_sender });

    old_note.remove();

    CT::fr old_value = *(old_note.get_preimage().value);

    CT::fr new_value = old_value + 5;

    my_utxo.insert({ .value = new_value, //
                     .owner = msg_sender,
                     .creator_address = msg_sender,
                     .memo = 1234 });

    exec_ctx.finalise();

    // Here, we test that the shared_ptr of a note, stored within the exec_ctx, works. TODO: put this in its own little
    // test, instead of this ever-growing beast test.
    auto new_note_pointers = exec_ctx.get_new_notes();
    std::shared_ptr<Note> debug_note = std::dynamic_pointer_cast<Note>(new_note_pointers[0]);
    info("new_note_pointers: ", new_note_pointers);
    info("*(new_note_pointers[0]): ", debug_note->get_preimage());

    auto new_nullifiers = exec_ctx.get_new_nullifiers();
    info("new_nullifiers: ", new_nullifiers);
}

TEST_F(state_var_tests, utxo_set_of_default_private_notes_fr)
{
    C composer;
    NativeOracle native_oracle = get_test_native_oracle();
    OracleWrapper oracle = OracleWrapper(composer, native_oracle);
    FunctionExecutionContext<C> exec_ctx(composer, oracle);

    // bool sort(NT::uint256 i, NT::uint256 j)
    // {
    //     return (i < j);
    // };

    Contract contract(exec_ctx, "TestContract");
    contract.declare_state_var("balances");

    // FUNCTION:

    using Note = DefaultPrivateNote<C, CT::fr>;

    UTXOSet<Note> balances(&exec_ctx, "balances");

    // Imagine these were passed into the function as args:
    CT::fr amount = 5;
    CT::address to_address = 765976;

    const auto& msg_sender = oracle.get_msg_sender();

    std::vector<Note> old_balance_notes = balances.get(2, { .owner = msg_sender });

    CT::fr old_value_1 = *(old_balance_notes[0].get_preimage().value);
    CT::fr old_value_2 = *(old_balance_notes[1].get_preimage().value);

    old_balance_notes[0].remove();
    old_balance_notes[1].remove();

    // MISSING: overflow & underflow checks, but I can't be bothered with safe_uint or range checks yet.

    CT::fr new_value = (old_value_1 + old_value_2) - amount;

    balances.insert({
        .value = new_value,
        .owner = to_address,
        .creator_address = msg_sender,
        .memo = 1234,
    });

    exec_ctx.finalise();

    // Here, we test that the shared_ptr of a note, stored within the exec_ctx, works. TODO: put this in its own little
    // test, instead of this ever-growing beast test.
    auto new_note_pointers = exec_ctx.get_new_notes();
    std::shared_ptr<Note> debug_note = std::dynamic_pointer_cast<Note>(new_note_pointers[0]);
    info("new_note_pointers: ", new_note_pointers);
    info("*(new_note_pointers[0]): ", debug_note->get_preimage());

    auto new_nullifiers = exec_ctx.get_new_nullifiers();
    info("new_nullifiers: ", new_nullifiers);
}

} // namespace aztec3::circuits::apps