// #include <common/serialize.hpp>
// #include <stdlib/types/turbo.hpp>
// #include <aztec3/oracle/oracle.hpp>
// #include <aztec3/circuits/apps/oracle_wrapper.hpp>
// #include <numeric/random/engine.hpp>
#include "index.hpp"

#include <aztec3/circuits/apps/test_apps/escrow/deposit.hpp>

#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/call_stack_item.hpp>
#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/abis/tx_context.hpp>
#include <aztec3/circuits/abis/signed_tx_object.hpp>
#include <aztec3/circuits/abis/tx_object.hpp>
#include <aztec3/circuits/abis/function_signature.hpp>
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/accumulated_data.hpp>
#include <aztec3/circuits/abis/private_kernel/constant_data.hpp>
#include <aztec3/circuits/abis/private_kernel/old_tree_roots.hpp>
#include <aztec3/circuits/abis/private_kernel/globals.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>

// #include <aztec3/circuits/mock/mock_circuit.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

#include <common/map.hpp>
#include <common/test.hpp>
#include <gtest/gtest.h>

// #include <aztec3/constants.hpp>
// #include <crypto/pedersen/pedersen.hpp>
// #include <stdlib/hash/pedersen/pedersen.hpp>

namespace {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::CallType;
using aztec3::circuits::abis::FunctionSignature;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
using aztec3::circuits::abis::SignedTxObject;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxObject;

using aztec3::circuits::abis::private_kernel::AccumulatedData;
using aztec3::circuits::abis::private_kernel::ConstantData;
using aztec3::circuits::abis::private_kernel::Globals;
using aztec3::circuits::abis::private_kernel::OldTreeRoots;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateInputs;
using aztec3::circuits::abis::private_kernel::PublicInputs;

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
    const NT::fr escrow_contract_leaf_index = 1;
    const NT::fr escrow_portal_contract_address = 23456;

    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::address tx_origin = msg_sender;

    Composer deposit_composer;
    DB db;

    NativeOracle deposit_oracle =
        NativeOracle(db, escrow_contract_address, msg_sender, tx_origin, msg_sender_private_key);
    OracleWrapper deposit_oracle_wrapper = OracleWrapper(deposit_composer, deposit_oracle);

    FunctionExecutionContext deposit_ctx(deposit_composer, deposit_oracle_wrapper);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);

    OptionalPrivateCircuitPublicInputs<NT> opt_deposit_public_inputs = deposit(deposit_ctx, amount, asset_id, memo);
    PrivateCircuitPublicInputs<NT> deposit_public_inputs = opt_deposit_public_inputs.remove_optionality();

    UnrolledProver deposit_prover = deposit_composer.create_unrolled_prover();
    NT::Proof deposit_proof = deposit_prover.construct_proof();
    // info("\ndeposit_proof: ", deposit_proof.proof_data);

    std::shared_ptr<NT::VK> deposit_vk = deposit_composer.compute_verification_key();

    //***************************************************************************
    // We can create a TxObject from some of the above data. Users must sign a TxObject in order to give permission for
    // a tx to take place - creating a SignedTxObject.
    //***************************************************************************

    TxObject<NT> deposit_tx_object = TxObject<NT>{
        .from = tx_origin,
        .to = escrow_contract_address,
        .function_signature =
            FunctionSignature<NT>{
                .vk_index = 0, // TODO: deduce this from the contract, somehow.
                .is_private = true,
                .is_constructor = false,
            },
        .custom_inputs = deposit_public_inputs.custom_inputs,
        .nonce = 0,
        .tx_context =
            TxContext<NT>{
                .called_from_l1 = false,
                .called_from_public_l2 = false,
                .is_fee_payment_tx = false,
                .reference_block_num = 0,
            },
        .chain_id = 1,
    };

    SignedTxObject<NT> signed_deposit_tx_object = SignedTxObject<NT>{
        .tx_object = deposit_tx_object,

        //     .signature = TODO: need a method for signing a TxObject.
    };

    //***************************************************************************
    // We mock a kernel circuit proof for the base case of kernel recursion (because even the first iteration of the
    // kernel circuit expects to verify some previous kernel circuit).
    //***************************************************************************

    Composer mock_kernel_composer;

    // TODO: we have a choice to make:
    // Either the `end` state of the mock kernel's public inputs can be set equal to the public call we _want_ to
    // verify in the first round of recursion, OR, we have some fiddly conditional logic in the circuit to ignore
    // certain checks if we're handling the 'base case' of the recursion.
    // I've chosen the former, for now.
    const CallStackItem<NT, CallType::Private> deposit_call_stack_item{
        .contract_address = deposit_tx_object.to,

        .function_signature = deposit_tx_object.function_signature,

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
                        .private_data_tree_root = deposit_public_inputs.old_private_data_tree_root,
                        // .nullifier_tree_root =
                        // .contract_tree_root =
                        // .private_kernel_vk_tree_root =
                    },
                .tx_context = deposit_tx_object.tx_context,
            },

        .is_private = true,
        // .is_public = false,
        // .is_contract_deployment = false,
    };

    mock_kernel_circuit(mock_kernel_composer, mock_kernel_public_inputs);

    UnrolledProver mock_kernel_prover = mock_kernel_composer.create_unrolled_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();
    // info("\nmock_kernel_proof: ", mock_kernel_proof.proof_data);

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    //***************************************************************************
    // Now we can execute and prove the first kernel iteration, with all the data generated above:
    // - app proof, public inputs, etc.
    // - mock kernel proof, public inputs, etc.
    //***************************************************************************

    Composer private_kernel_composer;

    // TODO: I think we need a different kind of oracle for the kernel circuits...
    NativeOracle private_kernel_oracle = NativeOracle(db, escrow_contract_address, msg_sender, msg_sender_private_key);
    OracleWrapper private_kernel_oracle_wrapper = OracleWrapper(private_kernel_composer, private_kernel_oracle);

    PrivateInputs<NT> private_inputs = PrivateInputs<NT>{
        .signed_tx_object = signed_deposit_tx_object,

        .previous_kernel =
            PreviousKernelData<NT>{
                .public_inputs = mock_kernel_public_inputs,
                .proof = mock_kernel_proof,
                .vk = mock_kernel_vk,
            },

        .private_call =
            PrivateCallData<NT>{
                .call_stack_item = deposit_call_stack_item,
                // .call_context_reconciliation_data = TODO

                .proof = deposit_proof,
                .vk = deposit_vk,
                // .vk_path TODO

                // .contract_tree_root TODO
                .contract_leaf_index = escrow_contract_leaf_index,
                // .contract_path TODO

                .portal_contract_address = escrow_portal_contract_address,
            },
    };

    private_kernel_circuit(private_kernel_composer, private_kernel_oracle_wrapper, private_inputs);

    info("computed witness: ", private_kernel_composer.computed_witness);
    info("witness: ", private_kernel_composer.witness);
    // info("constant variables: ", private_kernel_composer.constant_variables);
    // info("variables: ", private_kernel_composer.variables);

    // TODO: this fails intermittently, with:
    // bigfield multiply range check failed
    info("failed?: ", private_kernel_composer.failed);
    info("err: ", private_kernel_composer.err);
    info("n: ", private_kernel_composer.n);
}

} // namespace aztec3::circuits::kernel::private_kernel