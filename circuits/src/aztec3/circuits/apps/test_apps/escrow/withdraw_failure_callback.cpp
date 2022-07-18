#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>
#include "deposit.hpp"
#include "contract.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

void withdraw_failure_callback(FunctionExecutionContext<Composer>& exec_ctx,
                               NT::fr const& _asset_id,
                               NT::fr const& _amount,
                               NT::address const& _owner_address,
                               NT::fr const& _memo)
{
    auto& composer = exec_ctx.composer;
    auto& oracle = exec_ctx.oracle;
    Contract<Composer> contract = init_contract(exec_ctx);

    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr amount = to_ct(composer, _amount);
    CT::address owner_address = to_ct(composer, _owner_address);
    CT::fr memo = to_ct(composer, _memo);

    CT::address msg_sender = oracle.get_msg_sender();

    auto& balances = contract.get_private_state("balances");

    balances.at({ owner_address.to_field(), asset_id })
        .add({
            .value = amount,
            .owner_address = owner_address,
            .creator_address = msg_sender,
            .memo = memo,
        });

    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.custom_inputs[0] = asset_id;
    public_inputs.custom_inputs[1] = amount;
    public_inputs.custom_inputs[2] = owner_address.to_field();
    public_inputs.custom_inputs[3] = memo;

    exec_ctx.finalise();

    info("public inputs: ", public_inputs);
};

} // namespace aztec3::circuits::apps::test_apps::escrow