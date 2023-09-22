#include "function_1_1.hpp"

#include "contract.hpp"
#include "function_2_1.hpp"

#include "aztec3/circuits/apps/function_execution_context.hpp"
#include "aztec3/circuits/hash.hpp"

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

void function_1_1(FunctionExecutionContext& exec_ctx, std::vector<NT::fr> const& _args)
{
    /****************************************************************
     * Initialisation
     ****************************************************************/

    // Make the exec_ctx aware of the contract's layout.
    Contract contract_1 = init_contract_1();
    exec_ctx.register_contract(&contract_1);

    // Convert arguments into circuit types:
    auto& builder = exec_ctx.builder;
    const auto a = to_ct(builder, _args[0]);
    const auto b = to_ct(builder, _args[1]);
    const auto c = to_ct(builder, _args[2]);

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

    /**
     * Now we want to call an external function of another smart contract.
     * What I've written below is a bit of a hack.
     * In reality what we'll need from Noir++ is syntax which hides all of the boilerplate I write below.
     * Also, I _know_ where all the code for `function_2_1` is, so I've taken a big shortcut and #included
     * `function_2_1.hpp`. This won't be the way we'll fetch bytecode in practice. In practice, we might only learn the
     * contract address at runtime, and hence we'll have to fetch some acir bytecode at runtime from a DB and execute
     * that in a simulator (e.g. the ACVM). This is where all this noddy C++ example code that I'm writing falls short.
     * But hopefully this code still serves as a useful example of how the public inputs of a private function should be
     * computed.
     */
    // auto function_2_1 = contract_1.get_function("function_2_1");
    const CT::address fn_2_1_contract_address = 23456;

    // TODO: this can probably be tidied up.
    auto return_values =
        exec_ctx.call(fn_2_1_contract_address,
                      "function_2_1",
                      std::function<void(FunctionExecutionContext&, std::vector<NT::fr>)>(function_2_1),
                      { a, b, c, 0, 0, 0, 0, 0 });

    // Use the return value in some way, just for fun:
    x.initialise({
        .value = return_values[0],
        .owner = msg_sender,
    });

    /****************************************************************
     * CLEANUP
     ****************************************************************/

    // Push args to the public inputs.
    auto& public_inputs = exec_ctx.private_circuit_public_inputs;
    public_inputs.args_hash = compute_var_args_hash<CT>({ a, b, c });

    exec_ctx.finalize();
};

}  // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call