use crate::{
    abis::private_circuit_public_inputs::PrivateCircuitPublicInputs,
    tests::testing_harness::{PrivateCircuitPublicInputsBuilder, PrivateAppInputs},
    utils::uint128::U128,
    hash::NUM_FIELDS_PER_SHA256,
};

global BALANCES_SLOT = 1;

struct DepositParams {
    amount: Field,
    asset_id: Field,
    memo: Field,
    encrypted_logs_hash : [U128; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash : [U128; NUM_FIELDS_PER_SHA256],
    encrypted_log_preimages_length : Field,
    unencrypted_log_preimages_length : Field,
}

pub fn deposit_app(inputs: PrivateAppInputs, params: DepositParams) -> PrivateCircuitPublicInputs {
    let msg_sender = inputs.call_context.msg_sender;

    let mut context = PrivateCircuitPublicInputsBuilder::new(inputs);


    let user_balance_slot = dep::std::hash::pedersen_hash([BALANCES_SLOT, params.asset_id, msg_sender.to_field()]);

    let commitment = dep::std::hash::pedersen_hash([user_balance_slot, params.amount, msg_sender.to_field(), params.memo]);

    context.new_commitments.push(commitment);

    context.encrypted_logs_hash = params.encrypted_logs_hash.map(|uint: U128| uint.to_field());
    context.unencrypted_logs_hash = params.unencrypted_logs_hash.map(|uint: U128| uint.to_field());

    context.encrypted_log_preimages_length = params.encrypted_log_preimages_length;
    context.unencrypted_log_preimages_length = params.unencrypted_log_preimages_length;

    context.finish()
}