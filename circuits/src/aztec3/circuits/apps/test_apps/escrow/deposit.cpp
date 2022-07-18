#include "deposit.hpp"
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>
#include "contract.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> deposit(FunctionExecutionContext<Composer>& exec_ctx,
                                               NT::fr const& _amount,
                                               NT::fr const& _asset_id,
                                               NT::fr const& _memo)
{
    auto& composer = exec_ctx.composer;
    auto& oracle = exec_ctx.oracle;
    Contract<Composer> contract = init_contract(exec_ctx);

    CT::fr amount = to_ct(composer, _amount);
    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr memo = to_ct(composer, _memo);

    CT::address msg_sender = oracle.get_msg_sender();

    auto& balances = contract.get_private_state("balances");

    balances.at({ msg_sender.to_field(), asset_id })
        .add({
            .value = amount,
            .owner_address = msg_sender,
            .creator_address = msg_sender,
            .memo = memo,
        });

    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.custom_inputs[0] = amount;
    public_inputs.custom_inputs[1] = asset_id;
    public_inputs.custom_inputs[2] = memo;

    exec_ctx.finalise();

    info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<Composer>();
    // TODO: also return note preimages and nullifier preimages.
};

} // namespace aztec3::circuits::apps::test_apps::escrow