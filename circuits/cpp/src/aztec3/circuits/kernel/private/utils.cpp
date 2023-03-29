#include "index.hpp"
#include "init.hpp"

#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>
#include "aztec3/circuits/abis/private_kernel/new_contract_data.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::OptionallyRevealedData;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::private_kernel::AccumulatedData;
using aztec3::circuits::abis::private_kernel::NewContractData;
using aztec3::circuits::abis::private_kernel::OldTreeRoots;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::PublicInputs;
using aztec3::circuits::mock::mock_kernel_circuit;

using plonk::TurboComposer;
using namespace plonk::stdlib::types;

} // namespace

namespace aztec3::circuits::kernel::private_kernel::utils {

PreviousKernelData<NT> very_empty_previous_kernel()
{

    std::array<NewContractData<NT>, KERNEL_NEW_CONTRACTS_LENGTH> new_contracts;
    new_contracts.fill(NewContractData<NT>{
        .contract_address = fr::zero(), .portal_contract_address = fr::zero(), .function_tree_root = fr::zero() });

    std::array<OptionallyRevealedData<NT>, KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH> optionally_revealed_data;

    optionally_revealed_data.fill(OptionallyRevealedData<NT>{ .call_stack_item_hash = fr::zero(),
                                                              .function_data = FunctionData<NT>::empty(),
                                                              .emitted_events = { 0 },
                                                              .vk_hash = fr::zero(),
                                                              .portal_contract_address = { 0 },
                                                              .pay_fee_from_l1 = false,
                                                              .pay_fee_from_public_l2 = false,
                                                              .called_from_l1 = false,
                                                              .called_from_public_l2 = false });

    AccumulatedData<NT> accumulated_data = {
        .aggregation_object = AggregationObject{}, // TODO initialize members to 0
        .private_call_count = fr::zero(),
        .new_commitments = { 0 },
        .new_nullifiers = { 0 },
        .private_call_stack = { 0 },
        .public_call_stack = { 0 },
        .l1_msg_stack = { 0 },
        .new_contracts = new_contracts,
        .optionally_revealed_data = optionally_revealed_data,
    };

    OldTreeRoots<NT> old_tree_roots = {
        .private_data_tree_root = fr::zero(),
        .nullifier_tree_root = fr::zero(),
        .contract_tree_root = fr::zero(),
        .private_kernel_vk_tree_root = fr::zero(),
    };

    TxContext<NT> tx_context = {
        .is_fee_payment_tx = false,
        .is_rebate_payment_tx = false,
        .is_contract_deployment_tx = false,
        .contract_deployment_data = {
            .constructor_vk_hash = fr::zero(),
            .function_tree_root = fr::zero(),
            .contract_address_salt = fr::zero(),
            .portal_contract_address = fr::zero(),
        },
    };

    PublicInputs<NT> kernel_public_inputs = {
        .end = accumulated_data,
        .constants = { .old_tree_roots = old_tree_roots, .tx_context = tx_context },
        .is_private = true,
    };

    PreviousKernelData<NT> kernel_data = {
        .public_inputs = kernel_public_inputs,
    };

    return kernel_data;
}

PreviousKernelData<NT> dummy_previous_kernel_with_vk_proof()
{
    // TODO confirm this is the right way to initialize struct of 0s
    auto init_previous_kernel = very_empty_previous_kernel();

    auto crs_factory = std::make_shared<EnvReferenceStringFactory>();
    Composer mock_kernel_composer = Composer(crs_factory);
    auto mock_kernel_public_inputs = mock_kernel_circuit(mock_kernel_composer, init_previous_kernel.public_inputs);

    plonk::stdlib::types::Prover mock_kernel_prover = mock_kernel_composer.create_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    PreviousKernelData<NT> previous_kernel = {
        .public_inputs = mock_kernel_public_inputs,
        .proof = mock_kernel_proof,
        .vk = mock_kernel_vk,
    };
    return previous_kernel;
}

} // namespace aztec3::circuits::kernel::private_kernel::utils