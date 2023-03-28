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
using aztec3::circuits::abis::private_kernel::Globals;
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
        .tx_origin = msg_sender,
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

    TurboVerifier final_kernel_verifier = private_kernel_composer.create_verifier();
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

TEST(private_kernel_tests, test_basic_contract_deployment)
{
    //***************************************************************************
    // Some private circuit proof (`constructor`, in this case)
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
        .tx_origin = msg_sender,
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

    private_kernel_circuit(private_kernel_composer, private_inputs);

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

TEST(private_kernel_tests, test_create_proof_cbind_circuit)
{
    //***************************************************************************
    // Some private CIRCUIT proof (`constructor`, in this case)
    // and the cbind to generate it
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
        .tx_origin = msg_sender,
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

    uint8_t const* pk_buf;
    size_t pk_size = private_kernel__init_proving_key(&pk_buf);
    info("Proving key size: ", pk_size);

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
    size_t proof_data_size;
    uint8_t const* public_inputs;
    info("creating proof");
    size_t public_inputs_size = private_kernel__create_proof(signed_constructor_tx_request_vec.data(),
                                                             pk_buf,
                                                             private_constructor_call_vec.data(),
                                                             pk_buf,
                                                             false, // proverless
                                                             &proof_data,
                                                             &proof_data_size,
                                                             &public_inputs);
    info("Proof size: ", proof_data_size);
    info("PublicInputs size: ", public_inputs_size);

    free((void*)pk_buf);
    free((void*)vk_buf);
    free((void*)proof_data);
    free((void*)public_inputs);
}

TEST(private_kernel_tests, test_create_proof_cbind_native)
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
        .tx_origin = msg_sender,
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

    uint8_t const* pk_buf;
    size_t pk_size = private_kernel__init_proving_key(&pk_buf);
    info("Proving key size: ", pk_size);

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
    size_t proof_data_size;
    uint8_t const* public_inputs;
    info("creating proof");
    size_t public_inputs_size = private_kernel__create_proof(signed_constructor_tx_request_vec.data(),
                                                             pk_buf,
                                                             private_constructor_call_vec.data(),
                                                             pk_buf,
                                                             true, // proverless
                                                             &proof_data,
                                                             &proof_data_size,
                                                             &public_inputs);
    info("Proof size: ", proof_data_size);
    info("PublicInputs size: ", public_inputs_size);

    free((void*)pk_buf);
    free((void*)vk_buf);
    free((void*)proof_data);
    free((void*)public_inputs);
}

} // namespace aztec3::circuits::kernel::private_kernel