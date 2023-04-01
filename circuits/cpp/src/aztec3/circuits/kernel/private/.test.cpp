// #include <barretenberg/common/serialize.hpp>
// #include <barretenberg/stdlib/types/types.hpp>
// #include <aztec3/oracle/oracle.hpp>
// #include <aztec3/circuits/apps/oracle_wrapper.hpp>
// #include <barretenberg/numeric/random/engine.hpp>
#include "index.hpp"
#include "init.hpp"
#include "c_bind.h"

#include <aztec3/circuits/apps/test_apps/escrow/deposit.hpp>
#include <aztec3/circuits/apps/test_apps/basic_contract_deployment/basic_contract_deployment.hpp>

#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/abis/call_stack_item.hpp>
#include <aztec3/circuits/abis/contract_deployment_data.hpp>
#include <aztec3/circuits/abis/function_data.hpp>
#include <aztec3/circuits/abis/signed_tx_request.hpp>
#include <aztec3/circuits/abis/tx_context.hpp>
#include <aztec3/circuits/abis/tx_request.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/accumulated_data.hpp>
#include <aztec3/circuits/abis/private_kernel/constant_data.hpp>
#include <aztec3/circuits/abis/private_kernel/old_tree_roots.hpp>
#include <aztec3/circuits/abis/private_kernel/globals.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include "aztec3/circuits/kernel/private/utils.hpp"

#include <aztec3/circuits/apps/function_execution_context.hpp>

// #include <aztec3/circuits/mock/mock_circuit.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

#include <barretenberg/common/map.hpp>
#include <barretenberg/common/test.hpp>
#include <gtest/gtest.h>

// #include <aztec3/constants.hpp>
// #include <barretenberg/crypto/pedersen/pedersen.hpp>
// #include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>

namespace {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::CallType;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
using aztec3::circuits::abis::SignedTxRequest;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;

using aztec3::circuits::abis::private_kernel::AccumulatedData;
using aztec3::circuits::abis::private_kernel::ConstantData;
using aztec3::circuits::abis::private_kernel::OldTreeRoots;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateInputs;
using aztec3::circuits::abis::private_kernel::PublicInputs;

using aztec3::circuits::apps::test_apps::basic_contract_deployment::constructor;
using aztec3::circuits::apps::test_apps::escrow::deposit;

// using aztec3::circuits::mock::mock_circuit;
using aztec3::circuits::mock::mock_kernel_circuit;

} // namespace

namespace aztec3::circuits::kernel::private_kernel {

class private_kernel_tests : public ::testing::Test {};

TEST(private_kernel_tests, test_deposit)
{
    //***************************************************************************
    // Some private circuit proof (`deposit`, in this case)
    //***************************************************************************

    const NT::address escrow_contract_address = 12345;
    // const NT::fr escrow_contract_leaf_index = 1;
    const NT::fr escrow_portal_contract_address = 23456;

    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::address tx_origin = msg_sender;

    Composer deposit_composer = Composer("../barretenberg/cpp/srs_db/ignition");
    DB db;

    FunctionData<NT> function_data{
        .function_selector = 1, // TODO: deduce this from the contract, somehow.
        .is_private = true,
        .is_constructor = false,
    };

    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = escrow_contract_address,
        .portal_contract_address = 0,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = false,
    };

    NativeOracle deposit_oracle =
        NativeOracle(db, escrow_contract_address, function_data, call_context, msg_sender_private_key);
    OracleWrapper deposit_oracle_wrapper = OracleWrapper(deposit_composer, deposit_oracle);

    FunctionExecutionContext deposit_ctx(deposit_composer, deposit_oracle_wrapper);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);

    OptionalPrivateCircuitPublicInputs<NT> opt_deposit_public_inputs = deposit(deposit_ctx, amount, asset_id, memo);
    PrivateCircuitPublicInputs<NT> deposit_public_inputs = opt_deposit_public_inputs.remove_optionality();

    Prover deposit_prover = deposit_composer.create_prover();
    NT::Proof deposit_proof = deposit_prover.construct_proof();
    // info("\ndeposit_proof: ", deposit_proof.proof_data);

    std::shared_ptr<NT::VK> deposit_vk = deposit_composer.compute_verification_key();

    //***************************************************************************
    // We can create a TxRequest from some of the above data. Users must sign a TxRequest in order to give permission
    // for a tx to take place - creating a SignedTxRequest.
    //***************************************************************************

    TxRequest<NT> deposit_tx_request = TxRequest<NT>{
        .from = tx_origin,
        .to = escrow_contract_address,
        .function_data = function_data,
        .args = deposit_public_inputs.args,
        .nonce = 0,
        .tx_context =
            TxContext<NT>{
                .is_fee_payment_tx = false,
                .is_rebate_payment_tx = false,
                .is_contract_deployment_tx = false,
                .contract_deployment_data = ContractDeploymentData<NT>(),
            },
        .chain_id = 1,
    };

    SignedTxRequest<NT> signed_deposit_tx_request = SignedTxRequest<NT>{
        .tx_request = deposit_tx_request,

        //     .signature = TODO: need a method for signing a TxRequest.
    };

    //***************************************************************************
    // We mock a kernel circuit proof for the base case of kernel recursion (because even the first iteration of the
    // kernel circuit expects to verify some previous kernel circuit).
    //***************************************************************************

    Composer mock_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    // TODO: we have a choice to make:
    // Either the `end` state of the mock kernel's public inputs can be set equal to the public call we _want_ to
    // verify in the first round of recursion, OR, we have some fiddly conditional logic in the circuit to ignore
    // certain checks if we're handling the 'base case' of the recursion.
    // I've chosen the former, for now.
    const CallStackItem<NT, CallType::Private> deposit_call_stack_item{
        .contract_address = deposit_tx_request.to,

        .function_data = deposit_tx_request.function_data,

        .public_inputs = deposit_public_inputs,
    };

    std::array<NT::fr, KERNEL_PRIVATE_CALL_STACK_LENGTH> initial_kernel_private_call_stack{};
    initial_kernel_private_call_stack[0] = deposit_call_stack_item.hash();

    // Some test data:
    auto mock_kernel_public_inputs = PublicInputs<NT>{
        .end =
            AccumulatedData<NT>{
                .private_call_stack = initial_kernel_private_call_stack,
            },

        // These will be constant throughout all recursions, so can be set to those of the first function call - the
        // deposit tx.
        .constants =
            ConstantData<NT>{
                .old_tree_roots =
                    OldTreeRoots<NT>{
                        .private_data_tree_root = deposit_public_inputs.historic_private_data_tree_root,
                        // .nullifier_tree_root =
                        // .contract_tree_root =
                        // .private_kernel_vk_tree_root =
                    },
                .tx_context = deposit_tx_request.tx_context,
            },

        .is_private = true,
        // .is_public = false,
        // .is_contract_deployment = false,
    };

    mock_kernel_circuit(mock_kernel_composer, mock_kernel_public_inputs);

    Prover mock_kernel_prover = mock_kernel_composer.create_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();
    // info("\nmock_kernel_proof: ", mock_kernel_proof.proof_data);

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    //***************************************************************************
    // Now we can execute and prove the first kernel iteration, with all the data generated above:
    // - app proof, public inputs, etc.
    // - mock kernel proof, public inputs, etc.
    //***************************************************************************

    Composer private_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    PrivateInputs<NT> private_inputs = PrivateInputs<NT>{
        .signed_tx_request = signed_deposit_tx_request,

        .previous_kernel =
            PreviousKernelData<NT>{
                .public_inputs = mock_kernel_public_inputs,
                .proof = mock_kernel_proof,
                .vk = mock_kernel_vk,
            },

        .private_call =
            PrivateCallData<NT>{
                .call_stack_item = deposit_call_stack_item,
                .private_call_stack_preimages = deposit_ctx.get_private_call_stack_items(),

                .proof = deposit_proof,
                .vk = deposit_vk,

                // .function_leaf_membership_witness TODO
                // .contract_leaf_membership_witness TODO

                .portal_contract_address = escrow_portal_contract_address,

                // TODO: MembershipWitness<NCT, NULLIFIER_TREE_HEIGHT> function_leaf_membership_witness;
                // TODO: MembershipWitness<NCT, CONTRACT_TREE_HEIGHT> contract_leaf_membership_witness;
            },
    };

    private_kernel_circuit(private_kernel_composer, private_inputs);

    Prover final_kernel_prover = private_kernel_composer.create_prover();
    NT::Proof final_kernel_proof = final_kernel_prover.construct_proof();

    stdlib::types::Verifier final_kernel_verifier = private_kernel_composer.create_verifier();
    auto final_result = final_kernel_verifier.verify_proof(final_kernel_proof);
    EXPECT_EQ(final_result, true);

    info("computed witness: ", private_kernel_composer.computed_witness);
    // info("witness: ", private_kernel_composer.witness);
    // info("constant variables: ", private_kernel_composer.constant_variables);
    // info("variables: ", private_kernel_composer.variables);

    // TODO: this fails intermittently, with:
    // bigfield multiply range check failed
    info("failed?: ", private_kernel_composer.failed());
    info("err: ", private_kernel_composer.err());
    info("n: ", private_kernel_composer.get_num_gates());
}

TEST(private_kernel_tests, test_native_deposit)
{
    //***************************************************************************
    // Some private circuit proof (`deposit`, in this case)
    //***************************************************************************

    const NT::address escrow_contract_address = 12345;
    // const NT::fr escrow_contract_leaf_index = 1;
    const NT::fr escrow_portal_contract_address = 23456;

    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::address tx_origin = msg_sender;

    /**
     * NOTE: this is a bit cheeky. We want to test the _native_ kernel circuit implementation. But I don't want to write
     * a corresponding _native_ version of every 'app'. So let's just compute the circuit version of the app, and then
     * convert it to native types, so that it can be fed into the kernel circuit.
     *
     */
    Composer deposit_composer = Composer("../barretenberg/cpp/srs_db/ignition");
    DB db;

    FunctionData<NT> function_data{
        .function_selector = 1, // TODO: deduce this from the contract, somehow.
        .is_private = true,
        .is_constructor = false,
    };

    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = escrow_contract_address,
        .portal_contract_address = 0,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = false,
    };

    NativeOracle deposit_oracle =
        NativeOracle(db, escrow_contract_address, function_data, call_context, msg_sender_private_key);
    OracleWrapper deposit_oracle_wrapper = OracleWrapper(deposit_composer, deposit_oracle);

    FunctionExecutionContext deposit_ctx(deposit_composer, deposit_oracle_wrapper);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);

    OptionalPrivateCircuitPublicInputs<NT> opt_deposit_public_inputs = deposit(deposit_ctx, amount, asset_id, memo);
    PrivateCircuitPublicInputs<NT> deposit_public_inputs = opt_deposit_public_inputs.remove_optionality();

    Prover deposit_prover = deposit_composer.create_prover();
    NT::Proof deposit_proof = deposit_prover.construct_proof();

    std::shared_ptr<NT::VK> deposit_vk = deposit_composer.compute_verification_key();

    //***************************************************************************
    // We can create a TxRequest from some of the above data. Users must sign a TxRequest in order to give permission
    // for a tx to take place - creating a SignedTxRequest.
    //***************************************************************************

    TxRequest<NT> deposit_tx_request = TxRequest<NT>{
        .from = tx_origin,
        .to = escrow_contract_address,
        .function_data = function_data,
        .args = deposit_public_inputs.args,
        .nonce = 0,
        .tx_context =
            TxContext<NT>{
                .is_fee_payment_tx = false,
                .is_rebate_payment_tx = false,
                .is_contract_deployment_tx = false,
                .contract_deployment_data = ContractDeploymentData<NT>(),
            },
        .chain_id = 1,
    };

    SignedTxRequest<NT> signed_deposit_tx_request = SignedTxRequest<NT>{
        .tx_request = deposit_tx_request,

        //     .signature = TODO: need a method for signing a TxRequest.
    };

    //***************************************************************************
    // We mock a kernel circuit proof for the base case of kernel recursion (because even the first iteration of the
    // kernel circuit expects to verify some previous kernel circuit).
    //***************************************************************************

    Composer mock_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    // TODO: we have a choice to make:
    // Either the `end` state of the mock kernel's public inputs can be set equal to the public call we _want_ to
    // verify in the first round of recursion, OR, we have some fiddly conditional logic in the circuit to ignore
    // certain checks if we're handling the 'base case' of the recursion.
    // I've chosen the former, for now.
    const CallStackItem<NT, CallType::Private> deposit_call_stack_item{
        .contract_address = deposit_tx_request.to,

        .function_data = deposit_tx_request.function_data,

        .public_inputs = deposit_public_inputs,
    };

    std::array<NT::fr, KERNEL_PRIVATE_CALL_STACK_LENGTH> initial_kernel_private_call_stack{};
    initial_kernel_private_call_stack[0] = deposit_call_stack_item.hash();

    // Some test data:
    PublicInputs<NT> mock_kernel_public_inputs = PublicInputs<NT>{
        .end =
            AccumulatedData<NT>{
                .private_call_stack = initial_kernel_private_call_stack,
            },

        // These will be constant throughout all recursions, so can be set to those of the first function call - the
        // deposit tx.
        .constants =
            ConstantData<NT>{
                .old_tree_roots =
                    OldTreeRoots<NT>{
                        .private_data_tree_root = deposit_public_inputs.historic_private_data_tree_root,
                        // .nullifier_tree_root =
                        // .contract_tree_root =
                        // .private_kernel_vk_tree_root =
                    },
                .tx_context = deposit_tx_request.tx_context,
            },

        .is_private = true,
        // .is_public = false,
        // .is_contract_deployment = false,
    };

    mock_kernel_circuit(mock_kernel_composer, mock_kernel_public_inputs);

    Prover mock_kernel_prover = mock_kernel_composer.create_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    //***************************************************************************
    // Now we can execute and prove the first kernel iteration, with all the data generated above:
    // - app proof, public inputs, etc.
    // - mock kernel proof, public inputs, etc.
    //***************************************************************************

    // NOTE: WE DON'T USE A COMPOSER HERE, SINCE WE WANT TO TEST THE `native_private_kernel_circuit`
    // Composer private_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    PrivateInputs<NT> private_inputs = PrivateInputs<NT>{
        .signed_tx_request = signed_deposit_tx_request,

        .previous_kernel =
            PreviousKernelData<NT>{
                .public_inputs = mock_kernel_public_inputs,
                .proof = mock_kernel_proof,
                .vk = mock_kernel_vk,
            },

        .private_call =
            PrivateCallData<NT>{
                .call_stack_item = deposit_call_stack_item,
                .private_call_stack_preimages = deposit_ctx.get_private_call_stack_items(),

                .proof = deposit_proof,
                .vk = deposit_vk,
                // .vk_path TODO

                // TODO: MembershipWitness<NCT, NULLIFIER_TREE_HEIGHT> function_leaf_membership_witness;
                // TODO: MembershipWitness<NCT, CONTRACT_TREE_HEIGHT> contract_leaf_membership_witness;

                .portal_contract_address = escrow_portal_contract_address,
            },
    };

    PublicInputs<NT> public_inputs = native_private_kernel_circuit(private_inputs);

    // Prover final_kernel_prover = private_kernel_composer.create_prover();
    // NT::Proof final_kernel_proof = final_kernel_prover.construct_proof();

    // stdlib::types::Verifier final_kernel_verifier = private_kernel_composer.create_verifier();
    // auto final_result = final_kernel_verifier.verify_proof(final_kernel_proof);
    // EXPECT_EQ(final_result, true);

    // info("computed witness: ", private_kernel_composer.computed_witness);
    // info("witness: ", private_kernel_composer.witness);
    // info("constant variables: ", private_kernel_composer.constant_variables);
    // info("variables: ", private_kernel_composer.variables);

    // TODO: this fails intermittently, with:
    // bigfield multiply range check failed
    // info("failed?: ", private_kernel_composer.failed());
    // info("err: ", private_kernel_composer.err());
    // info("n: ", private_kernel_composer.get_num_gates());
}

TEST(private_kernel_tests, test_basic_contract_deployment)
{
    //***************************************************************************
    // Some private circuit proof (`constructor`, in this case)
    //***************************************************************************

    // Set this to 0 and then fill it in with correct contract address
    const NT::address new_contract_address = 0;
    // const NT::fr new_contract_leaf_index = 1;
    const NT::fr new_portal_contract_address = 23456;
    const NT::fr contract_address_salt = 34567;

    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::address tx_origin = msg_sender;

    FunctionData<NT> function_data{
        .function_selector = 1, // TODO: deduce this from the contract, somehow.
        .is_private = true,
        .is_constructor = true,
    };

    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = new_contract_address,
        .portal_contract_address = 0,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = true,
    };

    NT::fr arg0 = 5;
    NT::fr arg1 = 1;
    NT::fr arg2 = 999;

    Composer dummy_constructor_composer = Composer("../barretenberg/cpp/srs_db/ignition");
    {
        // Dummmy invokation, in order to derive the vk of this circuit

        // We need to use _dummy_ contract_deployment_data first, because the _proper_ version of the
        // contract_deployment_data will need to contain the constructor_vk_hash... but the constructor's vk can only be
        // computed after the composer has composed the circuit!
        ContractDeploymentData<NT> dummy_contract_deployment_data{
            .constructor_vk_hash = 0, // dummy
            .function_tree_root = 0,  // TODO actually get this?
            .contract_address_salt = contract_address_salt,
            .portal_contract_address = new_portal_contract_address,
        };

        DB dummy_db;
        NativeOracle dummy_constructor_oracle = NativeOracle(dummy_db,
                                                             new_contract_address,
                                                             function_data,
                                                             call_context,
                                                             dummy_contract_deployment_data,
                                                             msg_sender_private_key);
        OracleWrapper dummy_constructor_oracle_wrapper =
            OracleWrapper(dummy_constructor_composer, dummy_constructor_oracle);

        FunctionExecutionContext dummy_constructor_ctx(dummy_constructor_composer, dummy_constructor_oracle_wrapper);

        constructor(dummy_constructor_ctx, arg0, arg1, arg2);
    }

    // Now we can derive the vk:
    std::shared_ptr<NT::VK> constructor_vk = dummy_constructor_composer.compute_verification_key();
    auto constructor_vk_hash = stdlib::recursion::verification_key<CT::bn254>::compress_native(constructor_vk);

    // Now, we can proceed with the proper (non-dummy) invokation of our constructor circuit:

    Composer constructor_composer = Composer("../barretenberg/cpp/srs_db/ignition");
    DB db;

    ContractDeploymentData<NT> contract_deployment_data{
        .constructor_vk_hash = constructor_vk_hash, // TODO actually get this?
        .function_tree_root = 0,                    // TODO actually get this?
        .contract_address_salt = contract_address_salt,
        .portal_contract_address = new_portal_contract_address,
    };

    NativeOracle constructor_oracle = NativeOracle(
        db, new_contract_address, function_data, call_context, contract_deployment_data, msg_sender_private_key);
    OracleWrapper constructor_oracle_wrapper = OracleWrapper(constructor_composer, constructor_oracle);

    FunctionExecutionContext constructor_ctx(constructor_composer, constructor_oracle_wrapper);

    OptionalPrivateCircuitPublicInputs<NT> opt_constructor_public_inputs =
        constructor(constructor_ctx, arg0, arg1, arg2);

    PrivateCircuitPublicInputs<NT> constructor_public_inputs = opt_constructor_public_inputs.remove_optionality();

    Prover constructor_prover = constructor_composer.create_prover();
    NT::Proof constructor_proof = constructor_prover.construct_proof();
    // info("\nconstructor_proof: ", constructor_proof.proof_data);

    auto expected_constructor_hash =
        NT::compress({ function_data.hash(),
                       NT::compress<ARGS_LENGTH>(constructor_public_inputs.args, CONSTRUCTOR_ARGS),
                       constructor_vk_hash },
                     CONSTRUCTOR);
    NT::fr expected_contract_address = NT::compress({ msg_sender,
                                                      contract_deployment_data.contract_address_salt,
                                                      contract_deployment_data.function_tree_root,
                                                      expected_constructor_hash },
                                                    CONTRACT_ADDRESS);

    //***************************************************************************
    // We can create a TxRequest from some of the above data. Users must sign a TxRequest in order to give permission
    // for a tx to take place - creating a SignedTxRequest.
    //***************************************************************************

    TxRequest<NT> constructor_tx_request = TxRequest<NT>{
        .from = tx_origin,
        .to = new_contract_address,
        .function_data = function_data,
        .args = constructor_public_inputs.args,
        .nonce = 0,
        .tx_context =
            TxContext<NT>{
                .is_fee_payment_tx = false,
                .is_rebate_payment_tx = false,
                .is_contract_deployment_tx = true,
                .contract_deployment_data = contract_deployment_data,
            },
        .chain_id = 1,
    };

    SignedTxRequest<NT> signed_constructor_tx_request = SignedTxRequest<NT>{
        .tx_request = constructor_tx_request,

        //     .signature = TODO: need a method for signing a TxRequest.
    };

    //***************************************************************************
    // We mock a kernel circuit proof for the base case of kernel recursion (because even the first iteration of the
    // kernel circuit expects to verify some previous kernel circuit).
    //***************************************************************************

    Composer mock_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    // TODO: we have a choice to make:
    // Either the `end` state of the mock kernel's public inputs can be set equal to the public call we _want_ to
    // verify in the first round of recursion, OR, we have some fiddly conditional logic in the circuit to ignore
    // certain checks if we're handling the 'base case' of the recursion.
    // I've chosen the former, for now.
    const CallStackItem<NT, CallType::Private> constructor_call_stack_item{
        .contract_address = constructor_tx_request.to,

        .function_data = constructor_tx_request.function_data,

        .public_inputs = constructor_public_inputs,
    };

    std::array<NT::fr, KERNEL_PRIVATE_CALL_STACK_LENGTH> initial_kernel_private_call_stack{};
    initial_kernel_private_call_stack[0] = constructor_call_stack_item.hash();

    // Some test data:
    auto mock_kernel_public_inputs = PublicInputs<NT>();
    mock_kernel_public_inputs.end.private_call_stack = initial_kernel_private_call_stack,
    mock_kernel_public_inputs.constants.old_tree_roots.private_data_tree_root =
        constructor_public_inputs.historic_private_data_tree_root;
    mock_kernel_public_inputs.constants.tx_context = constructor_tx_request.tx_context;
    mock_kernel_public_inputs.is_private = true;

    mock_kernel_circuit(mock_kernel_composer, mock_kernel_public_inputs);

    Prover mock_kernel_prover = mock_kernel_composer.create_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();
    // info("\nmock_kernel_proof: ", mock_kernel_proof.proof_data);

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    //***************************************************************************
    // Now we can execute and prove the first kernel iteration, with all the data generated above:
    // - app proof, public inputs, etc.
    // - mock kernel proof, public inputs, etc.
    //***************************************************************************

    Composer private_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    PrivateInputs<NT> private_inputs = PrivateInputs<NT>{
        .signed_tx_request = signed_constructor_tx_request,

        .previous_kernel =
            PreviousKernelData<NT>{
                .public_inputs = mock_kernel_public_inputs,
                .proof = mock_kernel_proof,
                .vk = mock_kernel_vk,
            },

        .private_call =
            PrivateCallData<NT>{
                .call_stack_item = constructor_call_stack_item,
                .private_call_stack_preimages = constructor_ctx.get_private_call_stack_items(),

                .proof = constructor_proof,
                .vk = constructor_vk,

                // .function_leaf_membership_witness TODO
                // .contract_leaf_membership_witness TODO

                .portal_contract_address = new_portal_contract_address,
            },
    };

    auto private_kernel_circuit_public_inputs = private_kernel_circuit(private_kernel_composer, private_inputs);

    // Check contract address was correctly computed by the circuit
    EXPECT_EQ(private_kernel_circuit_public_inputs.end.new_contracts[0].contract_address.to_field(),
              expected_contract_address);

    // Create the final kernel proof and verify it natively.
    stdlib::types::Prover final_kernel_prover = private_kernel_composer.create_prover();
    NT::Proof final_kernel_proof = final_kernel_prover.construct_proof();

    stdlib::types::Verifier final_kernel_verifier = private_kernel_composer.create_verifier();
    auto final_result = final_kernel_verifier.verify_proof(final_kernel_proof);
    EXPECT_EQ(final_result, true);

    info("computed witness: ", private_kernel_composer.computed_witness);
    // info("witness: ", private_kernel_composer.witness);
    // info("constant variables: ", private_kernel_composer.constant_variables);
    // info("variables: ", private_kernel_composer.variables);

    // TODO: this fails intermittently, with:
    // bigfield multiply range check failed
    info("failed?: ", private_kernel_composer.failed());
    info("err: ", private_kernel_composer.err());
    info("n: ", private_kernel_composer.num_gates);
}

TEST(private_kernel_tests, test_native_basic_contract_deployment)
{
    //***************************************************************************
    // Some private circuit proof (`constructor`, in this case)
    //***************************************************************************

    // Set this to 0 and then fill it in with correct contract address
    const NT::address new_contract_address = 0;
    // const NT::fr new_contract_leaf_index = 1;
    const NT::fr new_portal_contract_address = 23456;
    const NT::fr contract_address_salt = 34567;

    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::address tx_origin = msg_sender;

    /**
     * NOTE: this is a bit cheeky. We want to test the _native_ kernel circuit implementation. But I don't want to write
     * a corresponding _native_ version of every 'app'. So let's just compute the circuit version of the app, and then
     * convert it to native types, so that it can be fed into the kernel circuit.
     *
     */
    FunctionData<NT> function_data{
        .function_selector = 1, // TODO: deduce this from the contract, somehow.
        .is_private = true,
        .is_constructor = true,
    };

    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = new_contract_address,
        .portal_contract_address = 0,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = true,
    };

    NT::fr arg0 = 5;
    NT::fr arg1 = 1;
    NT::fr arg2 = 999;

    Composer dummy_constructor_composer = Composer("../barretenberg/cpp/srs_db/ignition");
    {
        // Dummmy invokation, in order to derive the vk of this circuit

        // We need to use _dummy_ contract_deployment_data first, because the _proper_ version of the
        // contract_deployment_data will need to contain the constructor_vk_hash... but the constructor's vk can only be
        // computed after the composer has composed the circuit!
        ContractDeploymentData<NT> dummy_contract_deployment_data{
            .constructor_vk_hash = 0, // dummy
            .function_tree_root = 0,  // TODO actually get this?
            .contract_address_salt = contract_address_salt,
            .portal_contract_address = new_portal_contract_address,
        };

        DB dummy_db;
        NativeOracle dummy_constructor_oracle = NativeOracle(dummy_db,
                                                             new_contract_address,
                                                             function_data,
                                                             call_context,
                                                             dummy_contract_deployment_data,
                                                             msg_sender_private_key);
        OracleWrapper dummy_constructor_oracle_wrapper =
            OracleWrapper(dummy_constructor_composer, dummy_constructor_oracle);

        FunctionExecutionContext dummy_constructor_ctx(dummy_constructor_composer, dummy_constructor_oracle_wrapper);

        constructor(dummy_constructor_ctx, arg0, arg1, arg2);
    }

    // Now we can derive the vk:
    std::shared_ptr<NT::VK> constructor_vk = dummy_constructor_composer.compute_verification_key();
    auto constructor_vk_hash = stdlib::recursion::verification_key<CT::bn254>::compress_native(constructor_vk);

    // Now, we can proceed with the proper (non-dummy) invokation of our constructor circuit:

    Composer constructor_composer = Composer("../barretenberg/cpp/srs_db/ignition");
    DB db;

    ContractDeploymentData<NT> contract_deployment_data{
        .constructor_vk_hash = constructor_vk_hash, // TODO actually get this?
        .function_tree_root = 0,                    // TODO actually get this?
        .contract_address_salt = contract_address_salt,
        .portal_contract_address = new_portal_contract_address,
    };

    NativeOracle constructor_oracle = NativeOracle(
        db, new_contract_address, function_data, call_context, contract_deployment_data, msg_sender_private_key);
    OracleWrapper constructor_oracle_wrapper = OracleWrapper(constructor_composer, constructor_oracle);

    FunctionExecutionContext constructor_ctx(constructor_composer, constructor_oracle_wrapper);

    OptionalPrivateCircuitPublicInputs<NT> opt_constructor_public_inputs =
        constructor(constructor_ctx, arg0, arg1, arg2);

    PrivateCircuitPublicInputs<NT> constructor_public_inputs = opt_constructor_public_inputs.remove_optionality();

    Prover constructor_prover = constructor_composer.create_prover();
    NT::Proof constructor_proof = constructor_prover.construct_proof();

    // auto constructor_hash_real =
    //     NT::compress({ function_data.hash(),
    //                    NT::compress<ARGS_LENGTH>(constructor_public_inputs.args, CONSTRUCTOR_ARGS),
    //                    constructor_vk_hash },
    //                  CONSTRUCTOR);
    // auto contract_address_real = NT::compress({ msg_sender,
    //                                             contract_deployment_data.contract_address_salt,
    //                                             contract_deployment_data.function_tree_root,
    //                                             constructor_hash_real },
    //                                           CONTRACT_ADDRESS);

    //***************************************************************************
    // We can create a TxRequest from some of the above data. Users must sign a TxRequest in order to give permission
    // for a tx to take place - creating a SignedTxRequest.
    //***************************************************************************

    TxRequest<NT> constructor_tx_request = TxRequest<NT>{
        .from = tx_origin,
        .to = new_contract_address,
        .function_data = function_data,
        .args = constructor_public_inputs.args,
        .nonce = 0,
        .tx_context =
            TxContext<NT>{
                .is_fee_payment_tx = false,
                .is_rebate_payment_tx = false,
                .is_contract_deployment_tx = true,
                .contract_deployment_data = contract_deployment_data,
            },
        .chain_id = 1,
    };

    SignedTxRequest<NT> signed_constructor_tx_request = SignedTxRequest<NT>{
        .tx_request = constructor_tx_request,

        //     .signature = TODO: need a method for signing a TxRequest.
    };

    //***************************************************************************
    // We mock a kernel circuit proof for the base case of kernel recursion (because even the first iteration of the
    // kernel circuit expects to verify some previous kernel circuit).
    //***************************************************************************

    Composer mock_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    // TODO: we have a choice to make:
    // Either the `end` state of the mock kernel's public inputs can be set equal to the public call we _want_ to
    // verify in the first round of recursion, OR, we have some fiddly conditional logic in the circuit to ignore
    // certain checks if we're handling the 'base case' of the recursion.
    // I've chosen the former, for now.
    const CallStackItem<NT, CallType::Private> constructor_call_stack_item{
        .contract_address = constructor_tx_request.to,

        .function_data = constructor_tx_request.function_data,

        .public_inputs = constructor_public_inputs,
    };

    std::array<NT::fr, KERNEL_PRIVATE_CALL_STACK_LENGTH> initial_kernel_private_call_stack{};
    initial_kernel_private_call_stack[0] = constructor_call_stack_item.hash();

    // Some test data:
    auto mock_kernel_public_inputs = PublicInputs<NT>();
    mock_kernel_public_inputs.end.private_call_stack = initial_kernel_private_call_stack,
    mock_kernel_public_inputs.constants.old_tree_roots.private_data_tree_root =
        constructor_public_inputs.historic_private_data_tree_root;
    mock_kernel_public_inputs.constants.tx_context = constructor_tx_request.tx_context;
    mock_kernel_public_inputs.is_private = true;

    mock_kernel_circuit(mock_kernel_composer, mock_kernel_public_inputs);

    Prover mock_kernel_prover = mock_kernel_composer.create_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();
    // info("\nmock_kernel_proof: ", mock_kernel_proof.proof_data);

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    //***************************************************************************
    // Now we can execute and prove the first kernel iteration, with all the data generated above:
    // - app proof, public inputs, etc.
    // - mock kernel proof, public inputs, etc.
    //***************************************************************************

    // NOTE: WE DON'T USE A COMPOSER HERE, SINCE WE WANT TO TEST THE `native_private_kernel_circuit`
    // Composer private_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    PrivateInputs<NT> private_inputs = PrivateInputs<NT>{
        .signed_tx_request = signed_constructor_tx_request,

        .previous_kernel =
            PreviousKernelData<NT>{
                .public_inputs = mock_kernel_public_inputs,
                .proof = mock_kernel_proof,
                .vk = mock_kernel_vk,
            },

        .private_call =
            PrivateCallData<NT>{
                .call_stack_item = constructor_call_stack_item,
                .private_call_stack_preimages = constructor_ctx.get_private_call_stack_items(),

                .proof = constructor_proof,
                .vk = constructor_vk,

                // .function_leaf_membership_witness TODO
                // .contract_leaf_membership_witness TODO

                .portal_contract_address = new_portal_contract_address,
            },
    };

    PublicInputs<NT> private_kernel_circuit_public_inputs = native_private_kernel_circuit(private_inputs);
}

TEST(private_kernel_tests, test_create_proof_cbinds)
{
    //***************************************************************************
    // Some private NATIVE mocked proof (`constructor`, in this case)
    // and the cbind to generate valid outputs
    //***************************************************************************

    const NT::address new_contract_address = 12345;
    // const NT::fr new_contract_leaf_index = 1;
    const NT::fr new_portal_contract_address = 23456;

    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::address tx_origin = msg_sender;

    Composer constructor_composer = Composer("../barretenberg/cpp/srs_db/ignition");
    DB db;

    FunctionData<NT> function_data{
        .function_selector = 1, // TODO: deduce this from the contract, somehow.
        .is_private = true,
        .is_constructor = true,
    };

    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = new_contract_address,
        .portal_contract_address = 0,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = true,
    };

    NativeOracle constructor_oracle =
        NativeOracle(db, new_contract_address, function_data, call_context, msg_sender_private_key);
    OracleWrapper constructor_oracle_wrapper = OracleWrapper(constructor_composer, constructor_oracle);

    FunctionExecutionContext constructor_ctx(constructor_composer, constructor_oracle_wrapper);

    auto arg0 = NT::fr(5);
    auto arg1 = NT::fr(1);
    auto arg2 = NT::fr(999);

    OptionalPrivateCircuitPublicInputs<NT> opt_constructor_public_inputs =
        constructor(constructor_ctx, arg0, arg1, arg2);

    ContractDeploymentData<NT> contract_deployment_data{
        .constructor_vk_hash = 0, // TODO actually get this?
        .function_tree_root = 0,  // TODO actually get this?
        .contract_address_salt = 42,
        .portal_contract_address = new_portal_contract_address,
    };
    opt_constructor_public_inputs.contract_deployment_data = contract_deployment_data;

    PrivateCircuitPublicInputs<NT> constructor_public_inputs = opt_constructor_public_inputs.remove_optionality();

    Prover constructor_prover = constructor_composer.create_prover();
    NT::Proof constructor_proof = constructor_prover.construct_proof();
    // info("\nconstructor_proof: ", constructor_proof.proof_data);

    std::shared_ptr<NT::VK> constructor_vk = constructor_composer.compute_verification_key();

    //***************************************************************************
    // We can create a TxRequest from some of the above data. Users must sign a TxRequest in order to give permission
    // for a tx to take place - creating a SignedTxRequest.
    //***************************************************************************

    TxRequest<NT> constructor_tx_request = TxRequest<NT>{
        .from = tx_origin,
        .to = new_contract_address,
        .function_data = function_data,
        .args = constructor_public_inputs.args,
        .nonce = 0,
        .tx_context =
            TxContext<NT>{
                .is_fee_payment_tx = false,
                .is_rebate_payment_tx = false,
                .is_contract_deployment_tx = false,
                .contract_deployment_data = contract_deployment_data,
            },
        .chain_id = 1,
    };

    SignedTxRequest<NT> signed_constructor_tx_request = SignedTxRequest<NT>{
        .tx_request = constructor_tx_request,

        //     .signature = TODO: need a method for signing a TxRequest.
    };

    //***************************************************************************
    // We mock a kernel circuit proof for the base case of kernel recursion (because even the first iteration of the
    // kernel circuit expects to verify some previous kernel circuit).
    //***************************************************************************

    Composer mock_kernel_composer = Composer("../barretenberg/cpp/srs_db/ignition");

    // TODO: we have a choice to make:
    // Either the `end` state of the mock kernel's public inputs can be set equal to the public call we _want_ to
    // verify in the first round of recursion, OR, we have some fiddly conditional logic in the circuit to ignore
    // certain checks if we're handling the 'base case' of the recursion.
    // I've chosen the former, for now.
    const CallStackItem<NT, CallType::Private> constructor_call_stack_item{
        .contract_address = constructor_tx_request.to,

        .function_data = constructor_tx_request.function_data,

        .public_inputs = constructor_public_inputs,
    };

    std::array<NT::fr, KERNEL_PRIVATE_CALL_STACK_LENGTH> initial_kernel_private_call_stack{};
    initial_kernel_private_call_stack[0] = constructor_call_stack_item.hash();

    // TODO might be able to get rid of proving key buffer
    uint8_t const* pk_buf;
    size_t pk_size = private_kernel__init_proving_key(&pk_buf);
    info("Proving key size: ", pk_size);

    // TODO might be able to get rid of verification key buffer
    uint8_t const* vk_buf;
    size_t vk_size = private_kernel__init_verification_key(pk_buf, &vk_buf);
    info("Verification key size: ", vk_size);

    std::vector<uint8_t> signed_constructor_tx_request_vec;
    write(signed_constructor_tx_request_vec, signed_constructor_tx_request);

    PrivateCallData<NT> private_constructor_call = PrivateCallData<NT>{
        .call_stack_item = constructor_call_stack_item,
        .private_call_stack_preimages = constructor_ctx.get_private_call_stack_items(),

        .proof = constructor_proof,
        .vk = constructor_vk,

        // .function_leaf_membership_witness TODO
        // .contract_leaf_membership_witness TODO

        .portal_contract_address = new_portal_contract_address,
    };
    std::vector<uint8_t> private_constructor_call_vec;
    write(private_constructor_call_vec, private_constructor_call);

    uint8_t const* proof_data;
    uint8_t const* public_inputs;
    info("Simulating to generate public inputs...");
    size_t public_inputs_size = private_kernel__sim(signed_constructor_tx_request_vec.data(),
                                                    nullptr, // no previous kernel on first iteration
                                                    private_constructor_call_vec.data(),
                                                    true, // first iteration
                                                    &public_inputs);

    info("Proving");
    size_t proof_data_size = private_kernel__prove(signed_constructor_tx_request_vec.data(),
                                                   nullptr,
                                                   private_constructor_call_vec.data(),
                                                   pk_buf,
                                                   true, // first iteration
                                                   &proof_data);
    info("Proof size: ", proof_data_size);
    info("PublicInputs size: ", public_inputs_size);

    free((void*)pk_buf);
    free((void*)vk_buf);
    free((void*)proof_data);
    free((void*)public_inputs);
}

TEST(private_kernel_tests, test_dummy_previous_kernel_cbind)
{
    uint8_t const* cbind_previous_kernel_buf;
    size_t cbind_buf_size = private_kernel__dummy_previous_kernel(&cbind_previous_kernel_buf);

    PreviousKernelData<NT> previous_kernel = utils::dummy_previous_kernel_with_vk_proof();
    std::vector<uint8_t> expected_vec;
    write(expected_vec, previous_kernel);

    // Just compare the first 10 bytes of the serialized public outputs
    // TODO this is not a good test as it only checks a few bytes
    // would be best if we could just check struct equality or check
    // equality of an entire memory region (same as other similar TODOs
    // in other test files)
    if (cbind_buf_size > 10) {
        // for (size_t 0; i < public_inputs_size; i++) {
        for (size_t i = 0; i < 10; i++) {
            ASSERT_EQ(cbind_previous_kernel_buf[i], expected_vec[i]);
        }
    }
}

} // namespace aztec3::circuits::kernel::private_kernel
