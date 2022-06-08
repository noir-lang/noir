#include "withdraw.hpp"
#include "contract.hpp"
#include <aztec3/circuits/apps/private_state_note.hpp>
#include <aztec3/circuits/apps/l1_promise.hpp>
#include <aztec3/circuits/apps/l1_result.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
// #include <aztec3/circuits/abis/call_context.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

void withdraw(Composer& composer,
              OracleWrapper& oracle,
              NT::fr const& _amount,
              NT::fr const& _asset_id,
              NT::fr const& _memo,
              NT::fr const& _l1_withdrawal_address,
              NT::fr const& _fee,
              NT::boolean const& _is_fee_payment)
{
    info("\n\nin withdraw...");

    // Initialisation ***************************************************************

    CT::fr amount = to_ct(composer, _amount);
    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr memo = to_ct(composer, _memo);
    CT::fr l1_withdrawal_address = to_ct(composer, _l1_withdrawal_address);
    CT::fr fee = to_ct(composer, _fee);
    CT::boolean is_fee_payment = to_ct(composer, _is_fee_payment);

    // Get states and globals *******************************************************

    CT::address msg_sender = oracle.get_msg_sender();

    auto contract = init(composer, oracle);

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

    // Finalise state_factory *******************************************************

    contract.finalise();

    // Assign circuit-specific public inputs ****************************************

    auto public_inputs = OptionalPrivateCircuitPublicInputs<CT>::create();

    public_inputs.call_context = oracle.get_call_context(); /// TODO: can this be abstracted away out of this body?

    public_inputs.custom_public_inputs[0] = amount;
    public_inputs.custom_public_inputs[1] = asset_id;
    public_inputs.custom_public_inputs[2] = memo;
    public_inputs.custom_public_inputs[3] = l1_withdrawal_address;
    public_inputs.custom_public_inputs[4] = fee;
    public_inputs.custom_public_inputs[5] = is_fee_payment;

    public_inputs.emitted_public_inputs[0] = CT::fr::copy_as_new_witness(composer, l1_withdrawal_address);
    public_inputs.emitted_public_inputs[1] = CT::fr::copy_as_new_witness(composer, asset_id);
    public_inputs.emitted_public_inputs[2] = CT::fr::copy_as_new_witness(composer, fee);

    public_inputs.set_commitments(contract.private_state_factory.commitments);
    public_inputs.set_nullifiers(contract.private_state_factory.nullifiers);

    /// TODO: merkle membership check
    // public_inputs.old_private_data_tree_root

    public_inputs.is_fee_payment = CT::boolean(CT::fr::copy_as_new_witness(composer, is_fee_payment));

    public_inputs.set_public(composer);

    info("public inputs: ", public_inputs);
};

} // namespace aztec3::circuits::apps::test_apps::escrow