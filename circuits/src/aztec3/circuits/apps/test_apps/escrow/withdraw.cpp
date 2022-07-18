#include "withdraw.hpp"
#include <aztec3/circuits/apps/private_state_note.hpp>
#include <aztec3/circuits/apps/l1_promise.hpp>
#include <aztec3/circuits/apps/l1_result.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include "contract.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> withdraw(FunctionExecutionContext<Composer>& exec_ctx,
                                                NT::fr const& _amount,
                                                NT::fr const& _asset_id,
                                                NT::fr const& _memo,
                                                NT::fr const& _l1_withdrawal_address,
                                                NT::fr const& _fee)
{
    info("\n\nin withdraw...");

    // Initialisation ***************************************************************

    auto& composer = exec_ctx.composer;
    auto& oracle = exec_ctx.oracle;
    Contract<Composer> contract = init_contract(exec_ctx);

    CT::fr amount = to_ct(composer, _amount);
    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr memo = to_ct(composer, _memo);
    CT::fr l1_withdrawal_address = to_ct(composer, _l1_withdrawal_address);
    CT::fr fee = to_ct(composer, _fee);

    // Get states and globals *******************************************************

    CT::address msg_sender = oracle.get_msg_sender();

    auto& balances = contract.get_private_state("balances");

    // Circuit-specific logic *******************************************************

    balances.at({ msg_sender.to_field(), asset_id })
        .subtract({
            .value = amount + fee,
            .owner_address = msg_sender,
            .creator_address = msg_sender,
            .memo = memo,
        });

    auto& l1_withdraw_function = contract.get_l1_function("withdraw");

    auto [l1_promise, l1_result] = l1_withdraw_function.call({ asset_id, amount, msg_sender.to_field() });
    l1_promise.on_success("withdraw_success_callback",
                          {
                              l1_result[0],
                              amount,
                              msg_sender.to_field(),
                          });
    l1_promise.on_failure("withdraw_failure_callback",
                          {
                              asset_id,
                              amount,
                              msg_sender.to_field(),
                              memo,
                          });

    // Assign circuit-specific public inputs ****************************************

    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.custom_inputs[0] = amount;
    public_inputs.custom_inputs[1] = asset_id;
    public_inputs.custom_inputs[2] = memo;
    public_inputs.custom_inputs[3] = l1_withdrawal_address;
    public_inputs.custom_inputs[4] = fee;

    public_inputs.emitted_public_inputs[0] = CT::fr::copy_as_new_witness(composer, l1_withdrawal_address);
    public_inputs.emitted_public_inputs[1] = CT::fr::copy_as_new_witness(composer, asset_id);
    public_inputs.emitted_public_inputs[2] = CT::fr::copy_as_new_witness(composer, fee);

    exec_ctx.finalise();

    /// TODO: merkle membership check
    // public_inputs.old_private_data_tree_root

    info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<Composer>();
};

} // namespace aztec3::circuits::apps::test_apps::escrow