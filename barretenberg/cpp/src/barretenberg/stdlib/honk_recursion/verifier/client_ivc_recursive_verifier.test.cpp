#include "barretenberg/stdlib/honk_recursion/verifier/client_ivc_recursive_verifier.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/common/test.hpp"

namespace bb::stdlib::recursion::honk {
class ClientIVCRecursionTests : public testing::Test {
  public:
    using Builder = UltraCircuitBuilder;
    using ClientIVCVerifier = ClientIVCRecursiveVerifier;
    using VerifierInput = ClientIVCVerifier::VerifierInput;
    using FoldVerifierInput = ClientIVCVerifier::FoldVerifierInput;
    using GoblinVerifierInput = ClientIVCVerifier::GoblinVerifierInput;
    using VerifierInstance = FoldVerifierInput::Instance;
    using ECCVMVK = GoblinVerifier::ECCVMVerificationKey;
    using TranslatorVK = GoblinVerifier::TranslatorVerificationKey;
    using Proof = ClientIVC::Proof;

    static void SetUpTestSuite()
    {
        bb::srs::init_crs_factory("../srs_db/ignition");
        srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    struct ClientIVCProverOutput {
        Proof proof;
        VerifierInput verifier_input;
    };

    /**
     * @brief Construct a genuine ClientIVC prover output based on accumulation of an arbitrary set of mock circuits
     *
     */
    static ClientIVCProverOutput construct_client_ivc_prover_output(ClientIVC& ivc)
    {
        using Builder = ClientIVC::ClientCircuit;

        size_t NUM_CIRCUITS = 3;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            Builder circuit{ ivc.goblin.op_queue };
            GoblinMockCircuits::construct_mock_function_circuit(circuit);
            ivc.accumulate(circuit);
        }

        Proof proof = ivc.prove();
        FoldVerifierInput fold_verifier_input{ ivc.verifier_accumulator, { ivc.instance_vk } };
        GoblinVerifierInput goblin_verifier_input{ std::make_shared<ECCVMVK>(ivc.goblin.get_eccvm_proving_key()),
                                                   std::make_shared<TranslatorVK>(
                                                       ivc.goblin.get_translator_proving_key()) };

        return { proof, { fold_verifier_input, goblin_verifier_input } };
    }
};

/**
 * @brief Ensure the ClientIVC proof used herein can be natively verified
 *
 */
TEST_F(ClientIVCRecursionTests, NativeVerification)
{
    ClientIVC ivc;
    auto [proof, verifier_input] = construct_client_ivc_prover_output(ivc);

    // Construct the set of native verifier instances to be processed by the folding verifier
    std::vector<std::shared_ptr<VerifierInstance>> instances{ verifier_input.fold_input.accumulator };
    for (auto vk : verifier_input.fold_input.instance_vks) {
        instances.emplace_back(std::make_shared<VerifierInstance>(vk));
    }

    // Confirm that the IVC proof can be natively verified
    EXPECT_TRUE(ivc.verify(proof, instances));
}

/**
 * @brief Construct and Check a recursive ClientIVC verification circuit
 *
 */
TEST_F(ClientIVCRecursionTests, Basic)
{
    // Generate a genuine ClientIVC prover output
    ClientIVC ivc;
    auto [proof, verifier_input] = construct_client_ivc_prover_output(ivc);

    // Construct the ClientIVC recursive verifier
    auto builder = std::make_shared<Builder>();
    ClientIVCVerifier verifier{ builder, verifier_input };

    // Generate the recursive verification circuit
    verifier.verify(proof);

    EXPECT_TRUE(CircuitChecker::check(*builder));
}

} // namespace bb::stdlib::recursion::honk