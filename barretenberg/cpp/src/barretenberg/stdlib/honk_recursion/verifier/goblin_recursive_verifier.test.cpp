#include "barretenberg/stdlib/honk_recursion/verifier/goblin_recursive_verifier.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

namespace bb::stdlib::recursion::honk {
class GoblinRecursiveVerifierTests : public testing::Test {
  public:
    using Builder = GoblinRecursiveVerifier::Builder;
    using ECCVMVK = GoblinVerifier::ECCVMVerificationKey;
    using TranslatorVK = GoblinVerifier::TranslatorVerificationKey;

    using OuterFlavor = UltraFlavor;
    using OuterProver = UltraProver_<OuterFlavor>;
    using OuterVerifier = UltraVerifier_<OuterFlavor>;
    using OuterProverInstance = ProverInstance_<OuterFlavor>;

    static void SetUpTestSuite()
    {
        bb::srs::init_crs_factory("../srs_db/ignition");
        bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    static MegaCircuitBuilder construct_mock_circuit(std::shared_ptr<ECCOpQueue> op_queue)
    {
        MegaCircuitBuilder circuit{ op_queue };
        MockCircuits::construct_arithmetic_circuit(circuit, /*target_log2_dyadic_size=*/8);
        MockCircuits::construct_goblin_ecc_op_circuit(circuit);
        return circuit;
    }

    struct ProverOutput {
        GoblinProof proof;
        GoblinVerifier::VerifierInput verfier_input;
    };

    /**
     * @brief Create a goblin proof and the VM verification keys needed by the goblin recursive verifier
     *
     * @return ProverOutput
     */
    ProverOutput create_goblin_prover_output()
    {
        GoblinProver goblin;

        // Construct and accumulate multiple circuits
        size_t NUM_CIRCUITS = 3;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = construct_mock_circuit(goblin.op_queue);
            goblin.merge(circuit); // appends a recurisve merge verifier if a merge proof exists
        }

        // Output is a goblin proof plus ECCVM/Translator verification keys
        return { goblin.prove(),
                 { std::make_shared<ECCVMVK>(goblin.get_eccvm_proving_key()),
                   std::make_shared<TranslatorVK>(goblin.get_translator_proving_key()) } };
    }
};

/**
 * @brief Ensure the Goblin proof produced by the test method can be natively verified
 *
 */
TEST_F(GoblinRecursiveVerifierTests, NativeVerification)
{
    auto [proof, verifier_input] = create_goblin_prover_output();

    GoblinVerifier verifier{ verifier_input };

    EXPECT_TRUE(verifier.verify(proof));
}

/**
 * @brief Construct and check a goblin recursive verification circuit
 *
 */
TEST_F(GoblinRecursiveVerifierTests, Basic)
{
    auto [proof, verifier_input] = create_goblin_prover_output();

    Builder builder;
    GoblinRecursiveVerifier verifier{ &builder, verifier_input };
    verifier.verify(proof);

    info("Recursive Verifier: num gates = ", builder.num_gates);

    EXPECT_EQ(builder.failed(), false) << builder.err();

    EXPECT_TRUE(CircuitChecker::check(builder));

    // Construct and verify a proof for the Goblin Recursive Verifier circuit
    {
        auto instance = std::make_shared<OuterProverInstance>(builder);
        OuterProver prover(instance);
        auto verification_key = std::make_shared<typename OuterFlavor::VerificationKey>(instance->proving_key);
        OuterVerifier verifier(verification_key);
        auto proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);

        ASSERT(verified);
    }
}

/**
 * @brief Ensure failure of the goblin recursive verification circuit for a bad ECCVM proof
 *
 */
TEST_F(GoblinRecursiveVerifierTests, ECCVMFailure)
{
    auto [proof, verifier_input] = create_goblin_prover_output();

    // Tamper with the ECCVM proof
    for (auto& val : proof.eccvm_proof) {
        if (val > 0) { // tamper by finding the first non-zero value and incrementing it by 1
            val += 1;
            break;
        }
    }

    Builder builder;
    GoblinRecursiveVerifier verifier{ &builder, verifier_input };
    verifier.verify(proof);

    EXPECT_FALSE(CircuitChecker::check(builder));
}

/**
 * @brief Ensure failure of the goblin recursive verification circuit for a bad Translator proof
 *
 */
TEST_F(GoblinRecursiveVerifierTests, TranslatorFailure)
{
    auto [proof, verifier_input] = create_goblin_prover_output();

    // Tamper with the Translator proof
    for (auto& val : proof.translator_proof) {
        if (val > 0) { // tamper by finding the first non-zero value and incrementing it by 1
            val += 1;
            break;
        }
    }

    Builder builder;
    GoblinRecursiveVerifier verifier{ &builder, verifier_input };
    verifier.verify(proof);

    EXPECT_FALSE(CircuitChecker::check(builder));
}

/**
 * @brief Ensure failure of the goblin recursive verification circuit for bad translation evaluations
 *
 */
TEST_F(GoblinRecursiveVerifierTests, TranslationEvaluationsFailure)
{
    auto [proof, verifier_input] = create_goblin_prover_output();

    // Tamper with one of the translation evaluations
    proof.translation_evaluations.Px += 1;

    Builder builder;
    GoblinRecursiveVerifier verifier{ &builder, verifier_input };
    verifier.verify(proof);

    EXPECT_FALSE(CircuitChecker::check(builder));
}

} // namespace bb::stdlib::recursion::honk