#include "function1.hpp"
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>
#include "contract.hpp"

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> function1(FunctionExecutionContext<Composer>& exec_ctx,
                                                 NT::fr const& _a,
                                                 NT::fr const& _b,
                                                 NT::fr const& _c)
{
    auto& composer = exec_ctx.composer;
    auto& oracle = exec_ctx.oracle;
    Contract<Composer> contract = init_contract(exec_ctx);

    CT::fr a = to_ct(composer, _a);
    CT::fr b = to_ct(composer, _b);
    CT::fr c = to_ct(composer, _c);

    CT::address msg_sender = oracle.get_msg_sender();

    auto& x = contract.get_private_state_var("x");

    x.add({
        .value = a,
        .owner = msg_sender,
        .creator_address = msg_sender,
        .memo = 0,
    });

    // auto function2 = contract.get_function("function2");
    const NT::address fn2_contract_address = 23456;

    Composer fn2_composer;

    // Note: it's ok that we swap back into Native Types here - we don't need constraints. Creation of fn2_oracle is
    // necessary for circuit construction only; it's not part of the circuit itself. We check that the call_contexts
    // (msg_sender, contract_address, tx_origin) of functions 1 & 2 relate to one-another in the private kernel circuit,
    // by comparing the functions' public inputs.
    NativeOracle fn2_oracle = NativeOracle(oracle.oracle.db,
                                           fn2_contract_address,
                                           oracle.get_this_contract_address()
                                               .to_field()
                                               .get_value(), // TODO: add get_value() method to address type directly.
                                           oracle.get_tx_origin().to_field().get_value());
    OracleWrapper fn2_oracle_wrapper = OracleWrapper(fn2_composer, fn2_oracle);

    FunctionExecutionContext<Composer> fn1_exec_ctx(fn2_composer, fn2_oracle_wrapper);

    // auto result = function2.call(a, b, c);

    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.args[0] = a;
    public_inputs.args[1] = b;
    public_inputs.args[2] = c;

    // public_inputs.private_call_stack[0] = ...

    exec_ctx.finalise();

    info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<Composer>();
    // TODO: also return note preimages and nullifier preimages.
};

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call