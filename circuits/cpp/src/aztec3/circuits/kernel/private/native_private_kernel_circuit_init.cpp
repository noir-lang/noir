#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/combined_constant_data.hpp"
#include "aztec3/circuits/abis/historic_block_data.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_init.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"


namespace {
using NT = aztec3::utils::types::NativeTypes;

using aztec3::circuits::abis::CombinedConstantData;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using aztec3::utils::array_push;
using aztec3::utils::CircuitErrorCode;
using aztec3::utils::DummyCircuitBuilder;
using aztec3::utils::is_array_empty;


void initialise_end_values(PrivateKernelInputsInit<NT> const& private_inputs,
                           KernelCircuitPublicInputs<NT>& public_inputs)
{
    // Define the constants data.
    auto const& private_call_public_inputs = private_inputs.private_call.call_stack_item.public_inputs;
    auto const constants = CombinedConstantData<NT>{
        .block_data = private_call_public_inputs.historic_block_data,
        // TODO(dbanks12): remove historic root from app circuit public inputs and
        // add it to PrivateCallData: https://github.com/AztecProtocol/aztec-packages/issues/778
        // Then use this:
        // .private_data_tree_root = private_inputs.private_call.historic_private_data_tree_root,
        .tx_context = private_inputs.tx_request.tx_context,
    };

    // Set the constants in public_inputs.
    public_inputs.constants = constants;
}
}  // namespace

namespace aztec3::circuits::kernel::private_kernel {

void validate_this_private_call_against_tx_request(DummyCircuitBuilder& builder,
                                                   PrivateKernelInputsInit<NT> const& private_inputs)
{
    // TODO(mike): this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the 'fee'
    // tx, and the 'gas rebate' tx).

    // Confirm that the TxRequest (user's intent) matches the private call being executed
    const auto& tx_request = private_inputs.tx_request;
    const auto& call_stack_item = private_inputs.private_call.call_stack_item;

    builder.do_assert(tx_request.origin == call_stack_item.contract_address,
                      "user's intent does not match initial private call (origin address of tx_request must match "
                      "call_stack_item's contract_address)",
                      CircuitErrorCode::PRIVATE_KERNEL__USER_INTENT_MISMATCH_BETWEEN_TX_REQUEST_AND_CALL_STACK_ITEM);

    builder.do_assert(tx_request.function_data.hash() == call_stack_item.function_data.hash(),
                      "user's intent does not match initial private call (tx_request.function_data must match "
                      "call_stack_item.function_data)",
                      CircuitErrorCode::PRIVATE_KERNEL__USER_INTENT_MISMATCH_BETWEEN_TX_REQUEST_AND_CALL_STACK_ITEM);

    builder.do_assert(
        tx_request.args_hash == call_stack_item.public_inputs.args_hash,
        "user's intent does not match initial private call (noir function args passed to tx_request must match "
        "args in the call_stack_item)",
        CircuitErrorCode::PRIVATE_KERNEL__USER_INTENT_MISMATCH_BETWEEN_TX_REQUEST_AND_CALL_STACK_ITEM);
};

void validate_inputs(DummyCircuitBuilder& builder, PrivateKernelInputsInit<NT> const& private_inputs)
{
    const auto& this_call_stack_item = private_inputs.private_call.call_stack_item;

    builder.do_assert(this_call_stack_item.function_data.is_private == true,
                      "Cannot execute a non-private function with the private kernel circuit",
                      CircuitErrorCode::PRIVATE_KERNEL__NON_PRIVATE_FUNCTION_EXECUTED_WITH_PRIVATE_KERNEL);

    // TODO(mike): change to allow 3 initial calls on the private call stack, so a fee can be paid and a gas
    // rebate can be paid.

    /* If we are going to have 3 initial calls on the private call stack,
     * then do we still need the `private_call_stack`
     * despite no longer needing a full `previous_kernel`
     */

    builder.do_assert(this_call_stack_item.public_inputs.call_context.is_delegate_call == false,
                      "Users cannot make a delegatecall",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(this_call_stack_item.public_inputs.call_context.is_static_call == false,
                      "Users cannot make a static call",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);

    // The below also prevents delegatecall/staticcall in the base case
    builder.do_assert(this_call_stack_item.public_inputs.call_context.storage_contract_address ==
                          this_call_stack_item.contract_address,
                      "Storage contract address must be that of the called contract",
                      CircuitErrorCode::PRIVATE_KERNEL__CONTRACT_ADDRESS_MISMATCH);
}

void update_end_values(DummyCircuitBuilder& builder,
                       PrivateKernelInputsInit<NT> const& private_inputs,
                       KernelCircuitPublicInputs<NT>& public_inputs)
{
    // We only initialized constants member of public_inputs so far. Therefore, there must not be any
    // new nullifiers or logs as part of public_inputs.
    builder.do_assert(is_array_empty(public_inputs.end.new_commitments),
                      "public_inputs.end.new_commitments must start as empty in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(is_array_empty(public_inputs.end.new_nullifiers),
                      "public_inputs.end.new_nullifiers must start as empty in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(is_array_empty(public_inputs.end.nullified_commitments),
                      "public_inputs.end.nullified_commitments must start as empty in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(is_array_empty(public_inputs.end.encrypted_logs_hash),
                      "public_inputs.end.encrypted_logs_hash must start as empty in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(is_array_empty(public_inputs.end.unencrypted_logs_hash),
                      "public_inputs.end.unencrypted_logs_hash must start as empty in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(is_array_empty(public_inputs.end.read_requests),
                      "public_inputs.end.read_requests must start as empty in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(public_inputs.end.encrypted_log_preimages_length == NT::fr(0),
                      "public_inputs.end.encrypted_log_preimages_length must start as 0 in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    builder.do_assert(public_inputs.end.unencrypted_log_preimages_length == NT::fr(0),
                      "public_inputs.end.unencrypted_log_preimages_length must start as 0 in initial kernel iteration",
                      CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);

    // Since it's the first iteration, we need to push the the tx hash nullifier into the `new_nullifiers` array
    array_push(builder,
               public_inputs.end.new_nullifiers,
               private_inputs.tx_request.hash(),
               format(PRIVATE_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING,
                      "could not push tx hash nullifier into new_nullifiers array. Too many new nullifiers in one tx"));
    // Push an empty nullified commitment too since each nullifier must
    // be paired with a nonzero (real or "empty") nullified commitment
    array_push(builder,
               public_inputs.end.nullified_commitments,
               NT::fr(EMPTY_NULLIFIED_COMMITMENT),
               format(PRIVATE_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING,
                      "could not push tx hash nullifier into new_nullifiers array. Too many new nullifiers in one tx"));

    // Note that we do not need to nullify the transaction request nonce anymore.
    // Should an account want to additionally use nonces for replay protection or handling cancellations,
    // they will be able to do so in the account contract logic:
    // https://github.com/AztecProtocol/aztec-packages/issues/660
}

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_initial(DummyCircuitBuilder& builder,
                                                                    PrivateKernelInputsInit<NT> const& private_inputs)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // Do this before any functions can modify the inputs.
    initialise_end_values(private_inputs, public_inputs);

    validate_inputs(builder, private_inputs);

    common_validate_arrays(builder, private_inputs.private_call.call_stack_item.public_inputs);

    validate_this_private_call_against_tx_request(builder, private_inputs);

    common_validate_call_stack(builder, private_inputs.private_call);

    common_validate_read_requests(
        builder,
        public_inputs.constants.block_data.private_data_tree_root,
        private_inputs.private_call.call_stack_item.public_inputs.read_requests,  // read requests from private call
        private_inputs.private_call.read_request_membership_witnesses);

    // TODO(dbanks12): feels like update_end_values should happen after contract logic
    update_end_values(builder, private_inputs, public_inputs);
    common_update_end_values(builder, private_inputs.private_call, public_inputs);

    common_contract_logic(builder,
                          private_inputs.private_call,
                          public_inputs,
                          private_inputs.tx_request.tx_context.contract_deployment_data,
                          private_inputs.tx_request.function_data);

    // This is where a real circuit would perform recursive verification of the previous kernel proof and private call
    // proof.

    // In the native version, as there is no verify_proofs call, we can initialize aggregation object with the default
    // constructor.
    NT::AggregationObject const empty_aggregation_object{};
    public_inputs.end.aggregation_object = empty_aggregation_object;

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::private_kernel