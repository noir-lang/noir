#include "withdraw.hpp"

#include "contract.hpp"

#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/hash.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> withdraw(FunctionExecutionContext& exec_ctx,
                                                NT::fr const& _amount,
                                                NT::fr const& _asset_id,
                                                NT::fr const& _memo,
                                                NT::fr const& _l1_withdrawal_address,
                                                NT::fr const& _fee)
{
    /****************************************************************
     * Initialisation
     ****************************************************************/

    // Make the exec_ctx aware of the contract's layout.
    Contract contract = init_contract();
    exec_ctx.register_contract(&contract);

    // Convert arguments into circuit types:
    auto& builder = exec_ctx.builder;

    CT::fr const amount = to_ct(builder, _amount);
    CT::fr asset_id = to_ct(builder, _asset_id);
    CT::fr memo = to_ct(builder, _memo);
    CT::fr const l1_withdrawal_address = to_ct(builder, _l1_withdrawal_address);
    CT::fr const fee = to_ct(builder, _fee);

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

    // TODO: sort & filter functions!
    std::vector<DefaultNote> old_balance_notes =
        balances[asset_id][msg_sender.to_field()].get(2, { .owner = msg_sender });

    CT::fr const old_value_1 = *(old_balance_notes[0].get_preimage().value);
    CT::fr const old_value_2 = *(old_balance_notes[1].get_preimage().value);

    // MISSING: overflow & underflow checks, but I can't be bothered with safe_uint or range checks yet.
    CT::fr change = (old_value_1 + old_value_2) - (amount + fee);

    old_balance_notes[0].remove();
    old_balance_notes[1].remove();

    // Return change to self:
    balances[asset_id][msg_sender.to_field()].insert({
        .value = change,
        .owner = msg_sender,
        .creator_address = msg_sender,
        .memo = memo,
    });

    // auto& l1_withdraw_function = contract.get_l1_function("withdraw");

    // TODO: this doesn't do anything at the moment:
    // l1_withdraw_function.call({ asset_id, amount, msg_sender.to_field() });

    /****************************************************************
     * CLEANUP
     ****************************************************************/

    // Push args to the public inputs.
    auto& public_inputs = exec_ctx.private_circuit_public_inputs;
    public_inputs.args_hash = compute_var_args_hash<CT>({ amount, asset_id, memo, l1_withdrawal_address, fee });

    exec_ctx.finalize();

    /// TODO: merkle membership check
    // public_inputs.historic_private_data_tree_root

    // info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<C>();
};

}  // namespace aztec3::circuits::apps::test_apps::escrow