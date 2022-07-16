#include <aztec3/circuits/apps/private_state_note.hpp>
#include <aztec3/circuits/apps/function_executor.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
// #include <aztec3/circuits/abis/call_context.hpp>
#include "transfer.hpp"
#include "contract.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> transfer(FunctionExecutionContext<Composer>& exec_ctx,
                                                NT::fr const& _amount,
                                                NT::address const& _to,
                                                NT::fr const& _asset_id,
                                                NT::fr const& _memo,
                                                NT::boolean const& _reveal_msg_sender_to_recipient,
                                                NT::fr const& _fee)
{
    info("\n\nin transfer...");

    // Initialisation ***************************************************************

    auto& composer = exec_ctx.composer;
    auto& oracle = exec_ctx.oracle;
    Contract<Composer> contract = init_contract(exec_ctx);

    CT::fr amount = to_ct(composer, _amount);
    CT::address to = to_ct(composer, _to);
    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr memo = to_ct(composer, _memo);
    CT::boolean reveal_msg_sender_to_recipient = to_ct(composer, _reveal_msg_sender_to_recipient);
    CT::fr fee = to_ct(composer, _fee);

    // Get states and globals *******************************************************

    CT::address msg_sender = oracle.get_msg_sender();

    auto& balances = contract.get_private_state("balances");

    // Circuit-specific logic *******************************************************

    CT::address creator_address =
        CT::address::conditional_assign(reveal_msg_sender_to_recipient, msg_sender, CT::address(0));

    balances.at({ msg_sender.to_field(), asset_id })
        .subtract({
            .value = amount + fee,
            .owner_address = msg_sender,
            .creator_address = msg_sender,
            .memo = memo,
        });

    balances.at({ to.to_field(), asset_id })
        .add({
            .value = amount,
            .owner_address = to,
            .creator_address = creator_address,
            .memo = memo,
        });

    // Assign circuit-specific public inputs ****************************************

    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.custom_inputs[0] = amount;
    public_inputs.custom_inputs[1] = to.to_field();
    public_inputs.custom_inputs[2] = asset_id;
    public_inputs.custom_inputs[3] = memo;
    public_inputs.custom_inputs[4] = CT::fr(reveal_msg_sender_to_recipient);
    public_inputs.custom_inputs[5] = fee;

    public_inputs.emitted_public_inputs[0] = CT::fr::copy_as_new_witness(composer, fee);
    public_inputs.emitted_public_inputs[1] = CT::fr::copy_as_new_witness(composer, asset_id);

    /// TODO: merkle membership check
    // public_inputs.old_private_data_tree_root

    // Finalise *********************************************************************

    exec_ctx.finalise();

    info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<Composer>();
};

} // namespace aztec3::circuits::apps::test_apps::escrow