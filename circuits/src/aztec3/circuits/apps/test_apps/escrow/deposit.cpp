#include "deposit.hpp"
#include "contract.hpp"
#include <aztec3/circuits/apps/private_state_note.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
// #include <aztec3/circuits/abis/call_context.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::PrivateCircuitPublicInputs;

void deposit(
    Composer& composer, OracleWrapper& oracle, NT::fr const& _amount, NT::fr const& _asset_id, NT::fr const& _memo)
{
    CT::fr amount = to_ct(composer, _amount);
    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr memo = to_ct(composer, _memo);

    CT::address msg_sender = oracle.get_msg_sender();

    auto contract = init(composer, oracle);

    auto& balances = contract.get_private_state("balances");

    balances.at({ msg_sender.to_field(), asset_id })
        .add({
            .value = amount,
            .owner_address = msg_sender,
            .creator_address = msg_sender,
            .memo = memo,
        });

    contract.finalise();

    // TODO: maybe pass `oracle` to this `create()` function as well?
    auto public_inputs = PrivateCircuitPublicInputs<CT>::create();

    public_inputs.call_context = oracle.get_call_context(); /// TODO: can this be abstracted away out of this body?

    public_inputs.custom_public_inputs[0] = amount;
    public_inputs.custom_public_inputs[1] = asset_id;
    public_inputs.custom_public_inputs[2] = memo;

    public_inputs.set_commitments(contract.private_state_factory.commitments);
    public_inputs.set_nullifiers(contract.private_state_factory.nullifiers);

    public_inputs.pay_fee_from_l1 = to_ct(composer, true);
    CT::fr(*public_inputs.pay_fee_from_l1)
        .assert_equal(1); /// TODO: Ugly way of hard-coding a witness. Is there a nicer way?

    public_inputs.set_public(composer);

    info("public inputs: ", public_inputs);
};

} // namespace aztec3::circuits::apps::test_apps::escrow