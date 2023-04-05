#include "function_2_1.hpp"

#include "contract.hpp"

#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

void function_2_1(FunctionExecutionContext& exec_ctx, std::array<NT::fr, ARGS_LENGTH> const& _args)
{
    /****************************************************************
     * Initialisation
     ****************************************************************/

    // Make the exec_ctx aware of the contract's layout.
    Contract contract_2 = init_contract_2();
    exec_ctx.register_contract(&contract_2);

    // Convert arguments into circuit types:
    auto& composer = exec_ctx.composer;
    const auto a = to_ct(composer, _args[0]);
    const auto b = to_ct(composer, _args[1]);
    const auto c = to_ct(composer, _args[2]);

    /****************************************************************
     * Get States & Globals used by the function
     ****************************************************************/

    auto& oracle = exec_ctx.oracle;

    CT::address msg_sender = oracle.get_msg_sender();

    // Syntactic sugar for declaring a state variable:
    UTXO<Note> y(&exec_ctx, "y");

    /****************************************************************
     * BODY
     ****************************************************************/

    auto product = a * b * c;

    CT::address unique_person_who_may_initialise = 999999;

    unique_person_who_may_initialise.assert_equal(msg_sender);

    y.initialise({
        .value = product,
        .owner = msg_sender,
    });
    // TODO: how to initialise a UTXO if it's part of a nested function call, because the msg_sender will be a contract
    // address (currently the unique_initialiser_address is asserted to be the msg_sender).

    /****************************************************************
     * CLEANUP
     ****************************************************************/

    // Push args to the public inputs.

    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.args[0] = a;
    public_inputs.args[1] = b;
    public_inputs.args[2] = c;

    public_inputs.return_values[0] = product;

    exec_ctx.finalise();

    // info("public inputs: ", public_inputs);

    // TODO: also return note preimages and nullifier preimages.
};

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call