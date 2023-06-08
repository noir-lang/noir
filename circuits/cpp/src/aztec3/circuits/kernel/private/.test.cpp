#include "c_bind.h"
#include "index.hpp"
#include "init.hpp"
#include "testing_harness.hpp"

#include "aztec3/circuits/apps/test_apps/basic_contract_deployment/basic_contract_deployment.hpp"
#include "aztec3/circuits/apps/test_apps/escrow/deposit.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

namespace {

using aztec3::circuits::apps::test_apps::basic_contract_deployment::constructor;
using aztec3::circuits::apps::test_apps::escrow::deposit;

using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_init;
using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_inner;
using aztec3::circuits::kernel::private_kernel::testing_harness::validate_deployed_contract_address;

}  // namespace

namespace aztec3::circuits::kernel::private_kernel {

/**
 * @brief Some private circuit proof (`deposit`, in this case)
 */
TEST(private_kernel_tests, circuit_deposit)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;
    std::array<NT::fr, 2> const& encrypted_logs_hash = { NT::fr(16), NT::fr(69) };
    NT::fr const& encrypted_log_preimages_length = NT::fr(100);

    auto const& private_inputs = do_private_call_get_kernel_inputs_inner(
        false, deposit, { amount, asset_id, memo }, encrypted_logs_hash, encrypted_log_preimages_length, true);

    // Execute and prove the first kernel iteration
    Composer private_kernel_composer("../barretenberg/cpp/srs_db/ignition");
    auto const& public_inputs = private_kernel_circuit(private_kernel_composer, private_inputs, true);

    // TODO(jeanmon): this is a temporary hack until we have private_kernel_circuit init and inner
    // variant. Once this is supported, we will be able to generate public_inputs with
    // a call to private_kernel_circuit_init(private_inputs_init, ...)
    auto const& private_inputs_init = do_private_call_get_kernel_inputs_init(
        false, deposit, { amount, asset_id, memo }, encrypted_logs_hash, encrypted_log_preimages_length, true);

    // TODO(jeanmon): Once we have an inner/init private kernel circuit,
    // there should not be any new deployed contract address in public_inputs
    // and the following assertion can be uncommented:
    // validate_no_new_deployed_contract(public_inputs);

    // TODO(jeanmon): Remove once we have an inner/innit private kernel circuit
    // Check contract address was correctly computed by the circuit
    EXPECT_TRUE(validate_deployed_contract_address(private_inputs_init, public_inputs));
    EXPECT_FALSE(private_kernel_composer.failed());

    // Create the final kernel proof and verify it natively.
    auto final_kernel_prover = private_kernel_composer.create_prover();
    auto const& final_kernel_proof = final_kernel_prover.construct_proof();

    auto final_kernel_verifier = private_kernel_composer.create_verifier();
    auto const& final_result = final_kernel_verifier.verify_proof(final_kernel_proof);
    EXPECT_EQ(final_result, true);
}

/**
 * @brief Some private circuit proof (`constructor`, in this case)
 */
TEST(private_kernel_tests, circuit_basic_contract_deployment)
{
    NT::fr const& arg0 = 5;
    NT::fr const& arg1 = 1;
    NT::fr const& arg2 = 999;
    std::array<NT::fr, 2> const& encrypted_logs_hash = { NT::fr(16), NT::fr(69) };
    NT::fr const& encrypted_log_preimages_length = NT::fr(100);

    auto const& private_inputs = do_private_call_get_kernel_inputs_inner(
        true, constructor, { arg0, arg1, arg2 }, encrypted_logs_hash, encrypted_log_preimages_length, true);

    // Execute and prove the first kernel iteration
    Composer private_kernel_composer("../barretenberg/cpp/srs_db/ignition");
    auto const& public_inputs = private_kernel_circuit(private_kernel_composer, private_inputs, true);

    // TODO(jeanmon): this is a temporary hack until we have private_kernel_circuit init and inner
    // variant. Once this is supported, we will be able to generate public_inputs with
    // a call to private_kernel_circuit_init(private_inputs_init, ...)
    auto const& private_inputs_init = do_private_call_get_kernel_inputs_init(
        true, constructor, { arg0, arg1, arg2 }, encrypted_logs_hash, encrypted_log_preimages_length, true);

    // Check contract address was correctly computed by the circuit
    EXPECT_TRUE(validate_deployed_contract_address(private_inputs_init, public_inputs));
    EXPECT_FALSE(private_kernel_composer.failed());

    // Create the final kernel proof and verify it natively.
    auto final_kernel_prover = private_kernel_composer.create_prover();
    auto const& final_kernel_proof = final_kernel_prover.construct_proof();

    auto final_kernel_verifier = private_kernel_composer.create_verifier();
    auto const& final_result = final_kernel_verifier.verify_proof(final_kernel_proof);
    EXPECT_EQ(final_result, true);
}

/**
 * @brief Some private circuit simulation checked against its results via cbinds
 */
TEST(private_kernel_tests, circuit_create_proof_cbinds)
{
    NT::fr const& arg0 = 5;
    NT::fr const& arg1 = 1;
    NT::fr const& arg2 = 999;
    std::array<NT::fr, 2> const& encrypted_logs_hash = { NT::fr(16), NT::fr(69) };
    NT::fr const& encrypted_log_preimages_length = NT::fr(100);

    // first run actual simulation to get public inputs
    auto const& private_inputs = do_private_call_get_kernel_inputs_init(
        true, constructor, { arg0, arg1, arg2 }, encrypted_logs_hash, encrypted_log_preimages_length, true);
    DummyComposer composer = DummyComposer("private_kernel_tests__circuit_create_proof_cbinds");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    // serialize expected public inputs for later comparison
    std::vector<uint8_t> expected_public_inputs_vec;
    write(expected_public_inputs_vec, public_inputs);

    //***************************************************************************
    // Now run the simulate/prove cbinds to make sure their outputs match
    //***************************************************************************
    // TODO(david): might be able to get rid of proving key buffer
    uint8_t const* pk_buf = nullptr;
    private_kernel__init_proving_key(&pk_buf);
    // info("Proving key size: ", pk_size);

    // TODO(david): might be able to get rid of verification key buffer
    // uint8_t const* vk_buf;
    // size_t vk_size = private_kernel__init_verification_key(pk_buf, &vk_buf);
    // info("Verification key size: ", vk_size);

    std::vector<uint8_t> signed_constructor_tx_request_vec;
    write(signed_constructor_tx_request_vec, private_inputs.signed_tx_request);

    std::vector<uint8_t> private_constructor_call_vec;
    write(private_constructor_call_vec, private_inputs.private_call);

    uint8_t const* proof_data_buf = nullptr;
    uint8_t const* public_inputs_buf = nullptr;
    size_t public_inputs_size = 0;
    // info("Simulating to generate public inputs...");
    uint8_t* const circuit_failure_ptr = private_kernel__sim_init(signed_constructor_tx_request_vec.data(),
                                                                  private_constructor_call_vec.data(),
                                                                  &public_inputs_size,
                                                                  &public_inputs_buf);
    ASSERT_TRUE(circuit_failure_ptr == nullptr);

    // TODO(david): better equality check
    // for (size_t i = 0; i < public_inputs_size; i++)
    for (size_t i = 0; i < 10; i++) {
        ASSERT_EQ(public_inputs_buf[i], expected_public_inputs_vec[i]);
    }
    (void)public_inputs_size;
    // info("Proving");
    size_t const proof_data_size = private_kernel__prove(signed_constructor_tx_request_vec.data(),
                                                         nullptr,  // no previous kernel on first iteration
                                                         private_constructor_call_vec.data(),
                                                         pk_buf,
                                                         true,  // first iteration
                                                         &proof_data_buf);
    (void)proof_data_size;
    // info("Proof size: ", proof_data_size);
    // info("PublicInputs size: ", public_inputs_size);

    free((void*)pk_buf);
    // free((void*)vk_buf);
    free((void*)proof_data_buf);
    free((void*)public_inputs_buf);
}

}  // namespace aztec3::circuits::kernel::private_kernel
