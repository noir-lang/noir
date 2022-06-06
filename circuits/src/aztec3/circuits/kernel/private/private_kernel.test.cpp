#include <gtest/gtest.h>
#include <common/test.hpp>
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
#include <aztec3/circuits/abis/function_signature.hpp>
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/accumulated_data.hpp>
#include <aztec3/circuits/abis/private_kernel/constant_data.hpp>
#include <aztec3/circuits/abis/private_kernel/old_tree_roots.hpp>
#include <aztec3/circuits/abis/private_kernel/globals.hpp>
#include <aztec3/circuits/abis/executed_callback.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>

// #include <aztec3/circuits/mock/mock_circuit.hpp>
#include <aztec3/circuits/mock/mock_circuit_2.hpp>

// #include <aztec3/constants.hpp>
// #include <crypto/pedersen/pedersen.hpp>
// #include <stdlib/hash/pedersen/pedersen.hpp>

namespace {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::CallType;
using aztec3::circuits::abis::ExecutedCallback;
using aztec3::circuits::abis::FunctionSignature;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
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
using aztec3::circuits::mock::mock_circuit_2;

} // namespace

namespace aztec3::circuits::kernel::private_kernel {

class private_kernel_tests : public ::testing::Test {};

TEST(private_kernel_tests, test_deposit)
{
    const NT::address escrow_contract_address = 12345;
    const NT::fr escrow_contract_leaf_index = 1;
    const NT::fr escrow_portal_contract_address = 23456;

    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::fr msg_sender_private_key = 123456789;

    Composer deposit_composer;
    DB db;

    NativeOracle deposit_oracle = NativeOracle(db, escrow_contract_address, msg_sender, msg_sender_private_key);
    OracleWrapper deposit_oracle_wrapper = OracleWrapper(deposit_composer, deposit_oracle);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);

    PrivateCircuitPublicInputs<NT> deposit_public_inputs =
        deposit(deposit_composer, deposit_oracle_wrapper, amount, asset_id, memo);

    UnrolledProver deposit_prover = deposit_composer.create_unrolled_prover();
    NT::Proof deposit_proof = deposit_prover.construct_proof();
    // info("\ndeposit_proof: ", deposit_proof.proof_data);

    std::shared_ptr<NT::VK> deposit_vk = deposit_composer.compute_verification_key();

    //**************************************************

    // MIKE! Need to create a dummy kernel circuit and generate a proof and vk to feed into the below!!!

    Composer mock_composer;
    auto mock_public_inputs = PublicInputs<NT>{
        .end = AccumulatedData<NT>{},
        .constants =
            ConstantData<NT>{
                .old_tree_roots =
                    OldTreeRoots<NT>{
                        // TODO: this needs to be populated from the start, if the roots
                        // are used in any of the functions being recursed-through.
                        // .private_data_tree_root =
                        // .contract_tree_root =
                        // .l1_results_tree_root =
                        // .private_kernel_vk_tree_root =
                    },
                // .is_constructor_recursion = false,
                // .is_callback_recursion = false,
                .executed_callback = ExecutedCallback<NT>{},
                .globals = Globals<NT>{},
            },
        .is_private = true,
        // .is_public = false,
        // .is_contract_deployment = false,
    };
    mock_circuit_2(mock_composer, mock_public_inputs);

    UnrolledProver mock_prover = mock_composer.create_unrolled_prover();
    NT::Proof mock_proof = mock_prover.construct_proof();
    // info("\nmock_proof: ", mock_proof.proof_data);

    std::shared_ptr<NT::VK> mock_vk = mock_composer.compute_verification_key();

    //**************************************************

    Composer private_kernel_composer;

    NativeOracle private_kernel_oracle = NativeOracle(db, escrow_contract_address, msg_sender, msg_sender_private_key);
    OracleWrapper private_kernel_oracle_wrapper = OracleWrapper(private_kernel_composer, private_kernel_oracle);

    const CallStackItem<NT, CallType::Private> deposit_call_stack_item{
        .function_signature =
            FunctionSignature<NT>{
                .contract_address = escrow_contract_address,
                .vk_index = 0, // TODO: deduce this from a NT state_factory.
                .is_private = true,
                // .is_constructor = false,
                // .is_callback = false,
            },
        .public_inputs = deposit_public_inputs,
        .call_context = *deposit_public_inputs.call_context,
        //   .is_delegate_call = false,
        //   .is_static_call = false,
    };

    // PrivateInputs<NT> private_inputs;
    PrivateInputs<NT> private_inputs = {
        .start =
            AccumulatedData<NT>{
                .private_call_stack =
                    std::array<fr, KERNEL_PRIVATE_CALL_STACK_LENGTH>{
                        deposit_call_stack_item.hash(), 0, 0, 0, 0, 0, 0, 0 } }, // AccumulatedData starts out mostly
                                                                                 // empty, since nothing has been
                                                                                 // accumulated through kernel recursion
                                                                                 // yet.
        .previous_kernel =
            PreviousKernelData<NT>{
                .public_inputs =
                    PublicInputs<NT>{
                        .end = AccumulatedData<NT>{},
                        .constants =
                            ConstantData<NT>{
                                .old_tree_roots =
                                    OldTreeRoots<NT>{
                                        // .private_data_tree_root =
                                        // .contract_tree_root =
                                        // .l1_results_tree_root =
                                        // .private_kernel_vk_tree_root =
                                    },
                                // .is_constructor_recursion = false,
                                // .is_callback_recursion = false,
                                .executed_callback = ExecutedCallback<NT>{},
                                .globals = Globals<NT>{},
                            },
                        // .is_private = true,
                        // .is_public = false,
                        // .is_contract_deployment = false,
                    },
                .proof = mock_proof,
                .vk = mock_vk,
            },
        .private_call =
            PrivateCallData<NT>{
                .call_stack_item = deposit_call_stack_item,
                .proof = deposit_proof,
                .vk = deposit_vk,
                // .vk_path TODO
                .portal_contract_address = escrow_portal_contract_address,
                .contract_leaf_index = escrow_contract_leaf_index,
                // .contract_path TODO
            },
    };

    private_kernel_circuit(private_kernel_composer, private_kernel_oracle_wrapper, private_inputs);

    info("computed witness: ", private_kernel_composer.computed_witness);
    info("witness: ", private_kernel_composer.witness);
    // info("constant variables: ", private_kernel_composer.constant_variables);
    // info("variables: ", private_kernel_composer.variables);
    info("failed?: ", private_kernel_composer.failed);
    info("err: ", private_kernel_composer.err);
    info("n: ", private_kernel_composer.n);
}

} // namespace aztec3::circuits::kernel::private_kernel