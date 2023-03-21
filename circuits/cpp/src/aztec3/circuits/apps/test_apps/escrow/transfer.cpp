#include "transfer.hpp"

#include "contract.hpp"

#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> transfer(FunctionExecutionContext& exec_ctx,
                                                NT::fr const& _amount,
                                                NT::address const& _to,
                                                NT::fr const& _asset_id,
                                                NT::fr const& _memo,
                                                NT::boolean const& _reveal_msg_sender_to_recipient,
                                                NT::fr const& _fee)
{
    /****************************************************************
     * Initialisation
     ****************************************************************/

    // Make the exec_ctx aware of the contract's layout.
    Contract contract = init_contract();
    exec_ctx.register_contract(&contract);

    // Convert arguments into circuit types:
    auto& composer = exec_ctx.composer;

    CT::fr amount = to_ct(composer, _amount);
    CT::address to = to_ct(composer, _to);
    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr memo = to_ct(composer, _memo);
    CT::boolean reveal_msg_sender_to_recipient = to_ct(composer, _reveal_msg_sender_to_recipient);
    CT::fr fee = to_ct(composer, _fee);

    /****************************************************************
     * Get States & Globals used by the function
     ****************************************************************/

    auto& oracle = exec_ctx.oracle;
    CT::address msg_sender = oracle.get_msg_sender();

    // Syntactic sugar for a state variable:
    // Note: these Mappings always map-from a field type (because it was complicated enough!!!)
    // mapping(asset_id => mapping(owner => UTXOSet< >)) balances;
    Mapping<Mapping<UTXOSet<DefaultNote>>> balances(&exec_ctx, "balances");

    /****************************************************************
     * BODY
     ****************************************************************/

    CT::address creator_address =
        CT::address::conditional_assign(reveal_msg_sender_to_recipient, msg_sender, CT::address(0));

    // TODO: sort & filter functions!
    std::vector<DefaultNote> old_balance_notes =
        balances[asset_id][msg_sender.to_field()].get(2, { .owner = msg_sender });

    CT::fr old_value_1 = *(old_balance_notes[0].get_preimage().value);
    CT::fr old_value_2 = *(old_balance_notes[1].get_preimage().value);

    // MISSING: overflow & underflow checks, but I can't be bothered with safe_uint or range checks yet.
    CT::fr change = (old_value_1 + old_value_2) - (amount + fee);

    old_balance_notes[0].remove();
    old_balance_notes[1].remove();

    // Send amount to `to` address.
    balances[asset_id][to.to_field()].insert({
        .value = amount,
        .owner = to,
        .creator_address = creator_address,
        .memo = memo,
    });

    // Return change to sender:
    balances[asset_id][msg_sender.to_field()].insert({
        .value = change,
        .owner = msg_sender,
        .creator_address = msg_sender,
        .memo = memo,
    });

    /****************************************************************
     * CLEANUP
     ****************************************************************/

    // Push args to the public inputs.
    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.args[0] = amount;
    public_inputs.args[1] = to.to_field();
    public_inputs.args[2] = asset_id;
    public_inputs.args[3] = memo;
    public_inputs.args[4] = CT::fr(reveal_msg_sender_to_recipient);
    public_inputs.args[5] = fee;

    // Emit events
    public_inputs.emitted_events[0] = CT::fr::copy_as_new_witness(composer, fee);
    public_inputs.emitted_events[1] = CT::fr::copy_as_new_witness(composer, asset_id);

    /// TODO: merkle membership check
    // public_inputs.historic_private_data_tree_root

    exec_ctx.finalise();

    info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<C>();
};

} // namespace aztec3::circuits::apps::test_apps::escrow