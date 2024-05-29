#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

class GoblinRecursionTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite()
    {
        srs::init_crs_factory("../srs_db/ignition");
        srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using KernelInput = GoblinAccumulationOutput;
    using ProverInstance = ProverInstance_<MegaFlavor>;
    using VerifierInstance = VerifierInstance_<MegaFlavor>;
    using ECCVMVerificationKey = bb::ECCVMFlavor::VerificationKey;
    using TranslatorVerificationKey = bb::TranslatorFlavor::VerificationKey;

    static GoblinAccumulationOutput construct_accumulator(MegaCircuitBuilder& builder)
    {
        auto prover_instance = std::make_shared<ProverInstance>(builder);
        auto verification_key = std::make_shared<MegaFlavor::VerificationKey>(prover_instance->proving_key);
        auto verifier_instance = std::make_shared<VerifierInstance>(verification_key);
        MegaProver prover(prover_instance);
        auto ultra_proof = prover.construct_proof();
        return { ultra_proof, verifier_instance->verification_key };
    }
};

/**
 * @brief Test illustrating a Goblin-based IVC scheme
 * @details Goblin is usd to accumulate recursive verifications of the MegaHonk proving system.
 */
TEST_F(GoblinRecursionTests, Vanilla)
{
    GoblinProver goblin;

    GoblinAccumulationOutput kernel_accum;

    size_t NUM_CIRCUITS = 2;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {

        // Construct and accumulate a mock function circuit containing both arbitrary arithmetic gates and goblin
        // ecc op gates to make it a meaningful test
        MegaCircuitBuilder function_circuit{ goblin.op_queue };
        MockCircuits::construct_arithmetic_circuit(function_circuit, /*target_log2_dyadic_size=*/8);
        MockCircuits::construct_goblin_ecc_op_circuit(function_circuit);
        goblin.merge(function_circuit);
        auto function_accum = construct_accumulator(function_circuit);

        // Construct and accumulate the mock kernel circuit (no kernel accum in first round)
        MegaCircuitBuilder kernel_circuit{ goblin.op_queue };
        GoblinMockCircuits::construct_mock_kernel_small(kernel_circuit,
                                                        { function_accum.proof, function_accum.verification_key },
                                                        { kernel_accum.proof, kernel_accum.verification_key });
        goblin.merge(kernel_circuit);
        kernel_accum = construct_accumulator(kernel_circuit);
    }

    GoblinProof proof = goblin.prove();
    // Verify the final ultra proof
    MegaVerifier ultra_verifier{ kernel_accum.verification_key };
    bool ultra_verified = ultra_verifier.verify_proof(kernel_accum.proof);
    // Verify the goblin proof (eccvm, translator, merge)
    auto eccvm_vkey = std::make_shared<ECCVMVerificationKey>(goblin.get_eccvm_proving_key());
    auto translator_vkey = std::make_shared<TranslatorVerificationKey>(goblin.get_translator_proving_key());
    GoblinVerifier goblin_verifier{ eccvm_vkey, translator_vkey };
    bool verified = goblin_verifier.verify(proof);
    EXPECT_TRUE(ultra_verified && verified);
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/787) Expand these tests.
