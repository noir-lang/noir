
#include <gtest/gtest.h>

// #include "utxo_state_var.hpp"

#include "contract.hpp"
#include "function_execution_context.hpp"
#include "oracle_wrapper.hpp"
#include "notes/default_private_note/note.hpp"
#include "notes/default_private_note/note_preimage.hpp"
#include "notes/default_singleton_private_note/note.hpp"
#include "notes/default_singleton_private_note/note_preimage.hpp"
#include "notes/note_interface.hpp"
#include "state_vars/field_state_var.hpp"
#include "state_vars/mapping_state_var.hpp"
#include "state_vars/utxo_set_state_var.hpp"
#include "state_vars/utxo_state_var.hpp"

#include "aztec3/oracle/oracle.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
// Builder
using C = UltraCircuitBuilder;

// Types
using CT = aztec3::utils::types::CircuitTypes<C>;
using NT = aztec3::utils::types::NativeTypes;
using aztec3::utils::types::to_ct;

// exec_ctx
// using aztec3::circuits::apps::FunctionExecutionContext;

// Contract
using Contract = aztec3::circuits::apps::Contract<NT>;

// Oracle
using DB = aztec3::oracle::FakeDB;
using aztec3::oracle::NativeOracle;
using OracleWrapper = aztec3::circuits::apps::OracleWrapperInterface<C>;

// StateVars
using aztec3::circuits::apps::state_vars::FieldStateVar;
using aztec3::circuits::apps::state_vars::MappingStateVar;
using aztec3::circuits::apps::state_vars::UTXOSetStateVar;
using aztec3::circuits::apps::state_vars::UTXOStateVar;


using aztec3::circuits::apps::notes::DefaultPrivateNote;

using aztec3::circuits::apps::notes::DefaultSingletonPrivateNote;

// State variables
// Get rid of ugly `Builder` template arg from our state var types:
template <typename V> using Mapping = MappingStateVar<C, V>;
template <typename Note> using UTXO = UTXOStateVar<C, Note>;
template <typename Note> using UTXOSet = UTXOSetStateVar<C, Note>;

using Field = FieldStateVar<C>;
}  // namespace

namespace aztec3::circuits::apps {

class state_var_tests : public ::testing::Test {
  protected:
    static NativeOracle get_test_native_oracle(DB& db)
    {
        const NT::address contract_address = 12345;
        const NT::fr msg_sender_private_key = 123456789;
        const NT::address msg_sender = NT::fr(
            uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));

        FunctionData<NT> const function_data{
            .selector =
                {
                    .value = 1,  // TODO: deduce this from the contract, somehow.
                },
            .is_private = true,
            .is_constructor = false,
        };

        CallContext<NT> const call_context{ .msg_sender = msg_sender,
                                            .storage_contract_address = contract_address,
                                            .portal_contract_address = 0,
                                            .is_delegate_call = false,
                                            .is_static_call = false,
                                            .is_contract_deployment = false };

        return NativeOracle(db, contract_address, function_data, call_context, msg_sender_private_key);
    };
};

TEST_F(state_var_tests, circuit_mapping)
{
    // TODO: currently, we can't hide all of this boilerplate in a test fixture function, because each of these classes
    // contains a reference to earlier-declared classes... so we'd end up with classes containing dangling references,
    // if all this stuff were to be declared in a setup function's scope.
    // We could instead store shared_ptrs in every class...?
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext<C> exec_ctx(builder, oracle_wrapper);

    // TODO:
    // Interestingly, if I scope the below, the debugger works, but running the test via the command line fails. This is
    // because all pointers to the contract which are stored in other classes become dangling pointers, because contract
    // would go out of scope immediately... so the declaration of this contract and any pointers probably all need to be
    // shared_ptr<Contract> eventually.
    // {

    // I'm not entirely sure why we need to prepend `::` to `Contract`, to get to the unnamed namespace's declaration of
    // `Contract` above...
    ::Contract contract("TestContract");
    exec_ctx.register_contract(&contract);

    contract.declare_state_var("my_mapping");
    // }

    Mapping<Field> my_mapping(&exec_ctx, "my_mapping");

    my_mapping[5] = to_ct(builder, NT::fr(5));

    // info("my_mapping[5]: ", my_mapping[5]);
    // info("my_mapping[5].start_slot: ", my_mapping[5].start_slot);
    // info("my_mapping[5].storage_slot_point: ", my_mapping[5].storage_slot_point);
}

TEST_F(state_var_tests, circuit_mapping_within_mapping)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext<C> exec_ctx(builder, oracle_wrapper);

    // {
    ::Contract contract("TestContract");
    exec_ctx.register_contract(&contract);

    contract.declare_state_var("my_mapping");
    // }

    Mapping<Mapping<Field>> my_mapping(&exec_ctx, "my_mapping");

    my_mapping[5][6] = 7;

    info(my_mapping[5][6].state_var_name, ": ", my_mapping[5][6]);
}

TEST_F(state_var_tests, circuit_partial_mapping)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext<C> exec_ctx(builder, oracle_wrapper);

    // {
    ::Contract contract("TestContract");
    exec_ctx.register_contract(&contract);

    contract.declare_state_var("my_mapping");
    // }

    Mapping<Mapping<Field>> my_mapping(&exec_ctx, "my_mapping");

    my_mapping["?"][6] = 7;

    info(my_mapping["?"][6].state_var_name, ": ", my_mapping["?"][6]);
}

TEST_F(state_var_tests, circuit_utxo_of_default_private_note_fr)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext<C> exec_ctx(builder, oracle_wrapper);

    ::Contract contract("TestContract");
    exec_ctx.register_contract(&contract);

    contract.declare_state_var("my_utxo");

    // FUNCTION:

    using Note = DefaultPrivateNote<C, CT::fr>;

    UTXO<Note> my_utxo(&exec_ctx, "my_utxo");

    const auto& msg_sender = oracle_wrapper.get_msg_sender();

    Note old_note = my_utxo.get({ .owner = msg_sender });

    old_note.remove();

    CT::fr const old_value = *(old_note.get_preimage().value);

    CT::fr new_value = old_value + 5;

    my_utxo.insert({ .value = new_value,  //
                     .owner = msg_sender,
                     .creator_address = msg_sender,
                     .memo = 1234 });

    exec_ctx.finalise();

    // Here, we test that the shared_ptr of a note, stored within the exec_ctx, works. TODO: put this in its own little
    // test, instead of this ever-growing beast test.
    auto new_note_pointers = exec_ctx.get_new_notes();
    std::shared_ptr<Note> const debug_note = std::dynamic_pointer_cast<Note>(new_note_pointers[0]);
    // info("new_note_pointers: ", new_note_pointers);
    // info("*(new_note_pointers[0]): ", debug_note->get_preimage());

    auto new_nullifiers = exec_ctx.get_new_nullifiers();
    // info("new_nullifiers: ", new_nullifiers);
}

TEST_F(state_var_tests, circuit_utxo_set_of_default_private_notes_fr)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext<C> exec_ctx(builder, oracle_wrapper);

    // bool sort(NT::uint256 i, NT::uint256 j)
    // {
    //     return (i < j);
    // };

    ::Contract contract("TestContract");
    exec_ctx.register_contract(&contract);

    contract.declare_state_var("balances");

    // FUNCTION:

    using Note = DefaultPrivateNote<C, CT::fr>;

    UTXOSet<Note> balances(&exec_ctx, "balances");

    // Imagine these were passed into the function as args:
    CT::fr const amount = 5;
    CT::address to_address = 765976;

    const auto& msg_sender = oracle_wrapper.get_msg_sender();

    std::vector<Note> old_balance_notes = balances.get(2, { .owner = msg_sender });

    CT::fr const old_value_1 = *(old_balance_notes[0].get_preimage().value);
    CT::fr const old_value_2 = *(old_balance_notes[1].get_preimage().value);

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
    std::shared_ptr<Note> const debug_note = std::dynamic_pointer_cast<Note>(new_note_pointers[0]);
    // info("new_note_pointers: ", new_note_pointers);
    // info("*(new_note_pointers[0]): ", debug_note->get_preimage());

    auto new_nullifiers = exec_ctx.get_new_nullifiers();
    // info("new_nullifiers: ", new_nullifiers);
}

TEST_F(state_var_tests, circuit_initialise_utxo_of_default_singleton_private_note_fr)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext<C> exec_ctx(builder, oracle_wrapper);

    ::Contract contract("TestContract");
    exec_ctx.register_contract(&contract);

    contract.declare_state_var("my_utxo");

    // FUNCTION:

    // This time we use a slightly different Note type, which is tailored towards singleton UTXO use-cases. In
    // particular, it copes with the distinction between initialisation of the UTXO, vs future modification of the UTXO.
    using Note = DefaultSingletonPrivateNote<C, CT::fr>;

    UTXO<Note> my_utxo(&exec_ctx, "my_utxo");

    // We hard-code the address of the person who may initialise the state in the 'contract's bytecode' (i.e. as a
    // selector value). (Number chosen to match msg_sender of tests).
    const CT::address unique_person_who_may_initialise =
        NT::uint256(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL);

    unique_person_who_may_initialise.assert_equal(oracle_wrapper.get_msg_sender());

    // The person who may initialise the note might be different from the person who's actually given the note to own.
    // (E.g. the caller of this function might be the deployer of the contract, who is initialising notes on behalf of
    // other users)
    CT::address owner_of_initialised_note = 888888;

    my_utxo.initialise({ .value = 100, .owner = owner_of_initialised_note });

    exec_ctx.finalise();

    // Here, we test that the shared_ptr of a note, stored within the exec_ctx, works. TODO: put this in its own little
    // test, instead of this ever-growing beast test.
    auto new_note_pointers = exec_ctx.get_new_notes();
    std::shared_ptr<Note> const debug_note = std::dynamic_pointer_cast<Note>(new_note_pointers[0]);
    // info("new_note_pointers: ", new_note_pointers);
    // info("*(new_note_pointers[0]): ", debug_note->get_preimage());

    auto new_nullifiers = exec_ctx.get_new_nullifiers();
    // info("new_nullifiers: ", new_nullifiers);
}

TEST_F(state_var_tests, circuit_modify_utxo_of_default_singleton_private_note_fr)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext<C> exec_ctx(builder, oracle_wrapper);

    ::Contract contract("TestContract");
    exec_ctx.register_contract(&contract);

    contract.declare_state_var("my_utxo");

    // FUNCTION:

    // This time we use a slightly different Note type, which is tailored towards singleton UTXO use-cases. In
    // particular, it copes with the distinction between initialisation of the UTXO, vs future modification of the UTXO.
    using Note = DefaultSingletonPrivateNote<C, CT::fr>;

    UTXO<Note> my_utxo(&exec_ctx, "my_utxo");

    const auto& msg_sender = oracle_wrapper.get_msg_sender();

    Note old_note = my_utxo.get({ .owner = msg_sender });

    old_note.remove();

    CT::fr const old_value = *(old_note.get_preimage().value);

    CT::fr new_value = old_value + 5;

    my_utxo.insert({
        .value = new_value,  //
        .owner = msg_sender,
    });

    exec_ctx.finalise();

    // Here, we test that the shared_ptr of a note, stored within the exec_ctx, works. TODO: put this in its own little
    // test, instead of this ever-growing beast test.
    auto new_note_pointers = exec_ctx.get_new_notes();
    std::shared_ptr<Note> const debug_note = std::dynamic_pointer_cast<Note>(new_note_pointers[0]);
    // info("new_note_pointers: ", new_note_pointers);
    // info("*(new_note_pointers[0]): ", debug_note->get_preimage());

    auto new_nullifiers = exec_ctx.get_new_nullifiers();
    // info("new_nullifiers: ", new_nullifiers);
}

}  // namespace aztec3::circuits::apps