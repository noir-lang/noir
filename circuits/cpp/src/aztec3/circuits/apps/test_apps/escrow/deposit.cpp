#include "deposit.hpp"

#include "contract.hpp"

#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/apps/function_execution_context.hpp"
#include "aztec3/circuits/hash.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> deposit(FunctionExecutionContext& exec_ctx, std::vector<NT::fr> const& args)
{
    /****************************************************************
     * PREAMBLE
     ****************************************************************/

    // Make the exec_ctx aware of the contract's layout.
    Contract contract = init_contract();
    exec_ctx.register_contract(&contract);

    // Convert params into circuit types:
    auto& builder = exec_ctx.builder;

    CT::fr amount = to_ct(builder, args[0]);
    CT::fr asset_id = to_ct(builder, args[1]);
    CT::fr memo = to_ct(builder, args[2]);

    auto& oracle = exec_ctx.oracle;
    const CT::address msg_sender = oracle.get_msg_sender();

    /****************************************************************
     * BODY
     ****************************************************************/

    // Syntactic sugar for a state variable:
    // Note: these Mappings always map-from a field type (because it was complicated enough!!!)
    // mapping(asset_id => mapping(owner => UTXOSet< >)) balances;
    Mapping<Mapping<UTXOSet<DefaultNote>>> balances(&exec_ctx, "balances");

    balances[asset_id][msg_sender.to_field()].insert({
        .value = amount,
        .owner = msg_sender,
        .creator_address = msg_sender,
        .memo = memo,
    });

    /****************************************************************
     * CLEANUP
     ****************************************************************/

    // Push args to the public inputs.
    // TODO: don't give function direct access to the exec_ctx?
    auto& public_inputs = exec_ctx.private_circuit_public_inputs;
    public_inputs.args_hash = compute_var_args_hash<CT>({ amount, asset_id, memo });

    exec_ctx.finalize();

    // info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<C>();
    // TODO: also return note preimages and nullifier preimages.
    // TODO: or, we'll be collecting this data in the exec_ctx.
};

}  // namespace aztec3::circuits::apps::test_apps::escrow