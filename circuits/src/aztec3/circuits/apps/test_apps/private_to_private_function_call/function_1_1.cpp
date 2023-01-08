#include "function_1_1.hpp"

#include "contract.hpp"

#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> function_1_1(FunctionExecutionContext& exec_ctx,
                                                    NT::fr const& _a,
                                                    NT::fr const& _b,
                                                    NT::fr const& _c)
{
    /****************************************************************
     * Initialisation
     ****************************************************************/

    // Make the exec_ctx aware of the contract's layout.
    Contract contract_1 = init_contract_1();
    exec_ctx.register_contract(&contract_1);

    // Convert arguments into circuit types:
    auto& composer = exec_ctx.composer;

    CT::fr a = to_ct(composer, _a);
    CT::fr b = to_ct(composer, _b);
    CT::fr c = to_ct(composer, _c);

    /****************************************************************
     * Get States & Globals used by the function
     ****************************************************************/

    auto& oracle = exec_ctx.oracle;

    CT::address msg_sender = oracle.get_msg_sender();

    // Syntactic sugar for declaring a state variable:
    UTXO<Note> x(&exec_ctx, "x");

    /****************************************************************
     * BODY
     ****************************************************************/

    // Hard-coded to match tests.
    const CT::address unique_person_who_may_initialise =
        NT::uint256(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL);

    unique_person_who_may_initialise.assert_equal(msg_sender);

    x.initialise({
        .value = a,
        .owner = msg_sender,
    });

    // auto function_2_1 = contract_1.get_function("function_2_1");
    // const NT::address fn2_contract_address = 23456;

    // C fn2_composer;

    // // Note: it's ok that we swap back into Native Types here - we don't need constraints. Creation of fn2_oracle is
    // // necessary for circuit construction only; it's not part of the circuit itself. We check that the call_contexts
    // // (msg_sender, contract_address, tx_origin) of functions 1 & 2 relate to one-another in the private kernel
    // circuit,
    // // by comparing the functions' public inputs.
    // NativeOracle fn2_oracle = NativeOracle( //
    //     oracle.oracle.db,
    //     fn2_contract_address,
    //     oracle.get_this_contract_address().get_value(),
    //     oracle.get_tx_origin().get_value());
    // OracleWrapper fn2_oracle_wrapper = OracleWrapper(fn2_composer, fn2_oracle);

    // FunctionExecutionContext fn1_exec_ctx(fn2_composer, fn2_oracle_wrapper);

    // auto result = function_2_1.call(a, b, c);

    /****************************************************************
     * CLEANUP
     ****************************************************************/

    // Push args to the public inputs.
    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.args[0] = a;
    public_inputs.args[1] = b;
    public_inputs.args[2] = c;

    // public_inputs.private_call_stack[0] = ...

    exec_ctx.finalise();

    info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<C>();
    // TODO: also return note preimages and nullifier preimages.
};

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call