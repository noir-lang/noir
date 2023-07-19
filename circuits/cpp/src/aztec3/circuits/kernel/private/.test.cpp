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

}  // namespace

namespace aztec3::circuits::kernel::private_kernel {

class private_kernel_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }
};

/**
 * @brief Check private kernel circuit for arbitrary valid app proof and previous kernel proof
 * @details The purpose of this test is to check the private kernel circuit given a valid app proof and a valid previous
 * private kernel proof. To avoid doing actual proof construction, we simply read in an arbitrary but valid proof and a
 * corresponding valid verification key from file. The same proof and vkey data is used for both the app and the
 * previous kernel.
 * @note The choice of app circuit (currently 'deposit') is entirely arbitrary and can be replaced with any other valid
 * app circuit.
 */
TEST_F(private_kernel_tests, circuit_basic)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& empty_logs_hash = { NT::fr(16), NT::fr(69) };
    NT::fr const& empty_log_preimages_length = NT::fr(100);

    // Generate private inputs including proofs and vkeys for app circuit and previous kernel
    auto const& private_inputs = do_private_call_get_kernel_inputs_inner(false,
                                                                         deposit,
                                                                         { amount, asset_id, memo },
                                                                         empty_logs_hash,
                                                                         empty_logs_hash,
                                                                         empty_log_preimages_length,
                                                                         empty_log_preimages_length,
                                                                         empty_logs_hash,
                                                                         empty_logs_hash,
                                                                         empty_log_preimages_length,
                                                                         empty_log_preimages_length,
                                                                         true);

    // Execute and prove the first kernel iteration
    Builder private_kernel_builder;
    private_kernel_circuit(private_kernel_builder, private_inputs, true);

    // Check the private kernel circuit
    EXPECT_TRUE(private_kernel_builder.check_circuit());
}

/**
 * @brief Some private circuit simulation checked against its results via cbinds
 */
TEST_F(private_kernel_tests, circuit_cbinds)
{
    NT::fr const& arg0 = 5;
    NT::fr const& arg1 = 1;
    NT::fr const& arg2 = 999;
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash = { NT::fr(16), NT::fr(69) };
    NT::fr const& encrypted_log_preimages_length = NT::fr(100);
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash = { NT::fr(26), NT::fr(47) };
    NT::fr const& unencrypted_log_preimages_length = NT::fr(50);

    // first run actual simulation to get public inputs
    auto const& private_inputs = do_private_call_get_kernel_inputs_init(true,
                                                                        constructor,
                                                                        { arg0, arg1, arg2 },
                                                                        encrypted_logs_hash,
                                                                        unencrypted_logs_hash,
                                                                        encrypted_log_preimages_length,
                                                                        unencrypted_log_preimages_length,
                                                                        true);
    DummyBuilder builder = DummyBuilder("private_kernel_tests__circuit_create_proof_cbinds");
    auto const& public_inputs = native_private_kernel_circuit_initial(builder, private_inputs);

    // serialize expected public inputs for later comparison
    std::vector<uint8_t> expected_public_inputs_vec;
    serialize::write(expected_public_inputs_vec, public_inputs);

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
    write(signed_constructor_tx_request_vec, private_inputs.tx_request);

    std::vector<uint8_t> private_constructor_call_vec;
    write(private_constructor_call_vec, private_inputs.private_call);

    // uint8_t const* proof_data_buf = nullptr;
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
}

}  // namespace aztec3::circuits::kernel::private_kernel
