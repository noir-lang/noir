#include "basic_contract_deployment.hpp"

#include "contract.hpp"

#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/hash.hpp"

namespace aztec3::circuits::apps::test_apps::basic_contract_deployment {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> constructor(FunctionExecutionContext& exec_ctx, std::vector<NT::fr> const& args)
{
    /****************************************************************
     * PREAMBLE
     ****************************************************************/

    // Make the exec_ctx aware of the contract's layout.
    Contract contract = init_contract();
    exec_ctx.register_contract(&contract);

    // Convert params into circuit types:
    auto& builder = exec_ctx.builder;

    CT::fr const arg0 = to_ct(builder, args[0]);
    CT::fr const arg1 = to_ct(builder, args[1]);
    CT::fr const arg2 = to_ct(builder, args[2]);

    auto& oracle = exec_ctx.oracle;
    const CT::address msg_sender = oracle.get_msg_sender();

    /****************************************************************
     * BODY
     ****************************************************************/
    // SKIPPED

    /****************************************************************
     * CLEANUP
     ****************************************************************/

    // Push args to the public inputs.
    // TODO: don't give function direct access to the exec_ctx?
    auto& public_inputs = exec_ctx.private_circuit_public_inputs;
    public_inputs.args_hash = compute_var_args_hash<CT>({ arg0, arg1, arg2 });

    exec_ctx.finalize();

    // info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<C>();
}

}  // namespace aztec3::circuits::apps::test_apps::basic_contract_deployment
