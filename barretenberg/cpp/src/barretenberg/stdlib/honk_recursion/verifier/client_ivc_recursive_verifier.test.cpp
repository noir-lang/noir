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
            circuit.add_recursive_proof(stdlib::recursion::init_default_agg_obj_indices<Builder>(circuit));
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

    info("Recursive Verifier: num gates = ", builder->num_gates);

    EXPECT_EQ(builder->failed(), false) << builder->err();

    EXPECT_TRUE(CircuitChecker::check(*builder));
}

TEST_F(ClientIVCRecursionTests, ClientTubeBase)
{
    // Generate a genuine ClientIVC prover output
    ClientIVC ivc;
    auto [proof, verifier_input] = construct_client_ivc_prover_output(ivc);

    // Construct the ClientIVC recursive verifier
    auto tube_builder = std::make_shared<Builder>();
    ClientIVCVerifier verifier{ tube_builder, verifier_input };

    // Generate the recursive verification circuit
    verifier.verify(proof);

    tube_builder->add_recursive_proof(stdlib::recursion::init_default_agg_obj_indices<Builder>(*tube_builder));

    info("ClientIVC Recursive Verifier: num prefinalized gates = ", tube_builder->num_gates);

    EXPECT_EQ(tube_builder->failed(), false) << tube_builder->err();

    // EXPECT_TRUE(CircuitChecker::check(*tube_builder));

    // Construct and verify a proof for the ClientIVC Recursive Verifier circuit
    auto inner_instance = std::make_shared<ProverInstance_<UltraFlavor>>(*tube_builder);
    auto vk = std::make_shared<typename UltraFlavor::VerificationKey>(inner_instance->proving_key);
    UltraProver_<UltraFlavor> tube_prover{ inner_instance };
    auto tube_proof = tube_prover.construct_proof();

    Builder base_builder;
    UltraRecursiveVerifier_<UltraRecursiveFlavor_<Builder>> base_verifier{ &base_builder, vk };
    base_verifier.verify_proof(
        tube_proof,
        stdlib::recursion::init_default_aggregation_state<Builder, UltraRecursiveFlavor_<Builder>::Curve>(
            base_builder));
    info("UH Recursive Verifier: num prefinalized gates = ", base_builder.num_gates);

    EXPECT_EQ(base_builder.failed(), false) << base_builder.err();
    EXPECT_TRUE(CircuitChecker::check(base_builder));
}

} // namespace bb::stdlib::recursion::honk