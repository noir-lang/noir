#include "barretenberg/stdlib/honk_recursion/verifier/protogalaxy_recursive_verifier.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/protogalaxy/decider_prover.hpp"
#include "barretenberg/protogalaxy/decider_verifier.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/stdlib/hash/blake3s/blake3s.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/decider_recursive_verifier.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_recursive_flavor.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

namespace bb::stdlib::recursion::honk {
template <typename RecursiveFlavor> class ProtoGalaxyRecursiveTests : public testing::Test {
  public:
    // Define types for the inner circuit, i.e. the circuit whose proof will be recursively verified
    using InnerFlavor = typename RecursiveFlavor::NativeFlavor;
    using InnerProver = UltraProver_<InnerFlavor>;
    using InnerVerifier = UltraVerifier_<InnerFlavor>;
    using InnerBuilder = typename InnerFlavor::CircuitBuilder;
    using InnerProverInstance = ProverInstance_<InnerFlavor>;
    using InnerVerifierInstance = ::bb::VerifierInstance_<InnerFlavor>;
    using InnerVerificationKey = typename InnerFlavor::VerificationKey;
    using InnerCurve = bn254<InnerBuilder>;
    using Commitment = InnerFlavor::Commitment;
    using FF = InnerFlavor::FF;

    // Defines types for the outer circuit, i.e. the circuit of the recursive verifier
    using OuterBuilder = typename RecursiveFlavor::CircuitBuilder;
    using OuterFlavor = std::conditional_t<IsGoblinUltraBuilder<OuterBuilder>, GoblinUltraFlavor, UltraFlavor>;
    using OuterProver = UltraProver_<OuterFlavor>;
    using OuterVerifier = UltraVerifier_<OuterFlavor>;
    using OuterProverInstance = ProverInstance_<OuterFlavor>;

    using RecursiveVerifierInstances = ::bb::stdlib::recursion::honk::RecursiveVerifierInstances_<RecursiveFlavor, 2>;
    using FoldingRecursiveVerifier = ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances>;
    using DeciderRecursiveVerifier = DeciderRecursiveVerifier_<RecursiveFlavor>;
    using InnerDeciderProver = DeciderProver_<InnerFlavor>;
    using InnerDeciderVerifier = DeciderVerifier_<InnerFlavor>;
    using InnerVerifierInstances = VerifierInstances_<InnerFlavor, 2>;
    using InnerProverInstances = ProverInstances_<InnerFlavor>;
    using InnerFoldingVerifier = ProtoGalaxyVerifier_<InnerVerifierInstances>;
    using InnerFoldingProver = ProtoGalaxyProver_<InnerProverInstances>;

    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
    /**
     * @brief Create a non-trivial arbitrary inner circuit, the proof of which will be recursively verified
     *
     * @param builder
     * @param public_inputs
     * @param log_num_gates
     *
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/744): make testing utility with functionality shared
     * amongst test files
     */
    static void create_function_circuit(InnerBuilder& builder, size_t log_num_gates = 10)
    {
        using fr_ct = typename InnerCurve::ScalarField;
        using fq_ct = typename InnerCurve::BaseField;
        using public_witness_ct = typename InnerCurve::public_witness_ct;
        using witness_ct = typename InnerCurve::witness_ct;
        using byte_array_ct = typename InnerCurve::byte_array_ct;
        using fr = typename InnerCurve::ScalarFieldNative;

        // Create 2^log_n many add gates based on input log num gates
        const size_t num_gates = 1 << log_num_gates;
        for (size_t i = 0; i < num_gates; ++i) {
            fr a = fr::random_element();
            uint32_t a_idx = builder.add_variable(a);

            fr b = fr::random_element();
            fr c = fr::random_element();
            fr d = a + b + c;
            uint32_t b_idx = builder.add_variable(b);
            uint32_t c_idx = builder.add_variable(c);
            uint32_t d_idx = builder.add_variable(d);

            builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }

        // Define some additional non-trivial but arbitrary circuit logic
        fr_ct a(public_witness_ct(&builder, fr::random_element()));
        fr_ct b(public_witness_ct(&builder, fr::random_element()));
        fr_ct c(public_witness_ct(&builder, fr::random_element()));

        for (size_t i = 0; i < 32; ++i) {
            a = (a * b) + b + a;
            a = a.madd(b, c);
        }
        pedersen_hash<InnerBuilder>::hash({ a, b });
        byte_array_ct to_hash(&builder, "nonsense test data");
        blake3s(to_hash);

        fr bigfield_data = fr::random_element();
        fr bigfield_data_a{ bigfield_data.data[0], bigfield_data.data[1], 0, 0 };
        fr bigfield_data_b{ bigfield_data.data[2], bigfield_data.data[3], 0, 0 };

        fq_ct big_a(fr_ct(witness_ct(&builder, bigfield_data_a.to_montgomery_form())), fr_ct(witness_ct(&builder, 0)));
        fq_ct big_b(fr_ct(witness_ct(&builder, bigfield_data_b.to_montgomery_form())), fr_ct(witness_ct(&builder, 0)));

        big_a* big_b;
    };

    static std::tuple<std::shared_ptr<InnerProverInstance>, std::shared_ptr<InnerVerifierInstance>>
    fold_and_verify_native()
    {
        InnerBuilder builder1;
        create_function_circuit(builder1);
        InnerBuilder builder2;
        builder2.add_public_variable(FF(1));
        create_function_circuit(builder2);

        auto prover_instance_1 = std::make_shared<InnerProverInstance>(builder1);
        auto verification_key_1 = std::make_shared<InnerVerificationKey>(prover_instance_1->proving_key);
        auto verifier_instance_1 = std::make_shared<InnerVerifierInstance>(verification_key_1);
        auto prover_instance_2 = std::make_shared<InnerProverInstance>(builder2);
        auto verification_key_2 = std::make_shared<InnerVerificationKey>(prover_instance_2->proving_key);
        auto verifier_instance_2 = std::make_shared<InnerVerifierInstance>(verification_key_2);
        InnerFoldingProver folding_prover({ prover_instance_1, prover_instance_2 });
        InnerFoldingVerifier folding_verifier({ verifier_instance_1, verifier_instance_2 });

        auto [prover_accumulator, folding_proof] = folding_prover.fold_instances();
        auto verifier_accumulator = folding_verifier.verify_folding_proof(folding_proof);
        return { prover_accumulator, verifier_accumulator };
    }

    /**
     *@brief Create inner circuit and call check_circuit on it
     */
    static void test_circuit()
    {
        InnerBuilder builder;

        create_function_circuit(builder);

        bool result = CircuitChecker::check(builder);
        EXPECT_EQ(result, true);
    };

    /**
     * @brief Ensure that evaluating the perturbator in the recursive folding verifier returns the same result as
     * evaluating in Polynomial class.
     *
     */
    static void test_new_evaluate()
    {
        OuterBuilder builder;
        using fr_ct = typename bn254<OuterBuilder>::ScalarField;
        using fr = typename bn254<OuterBuilder>::ScalarFieldNative;

        std::vector<fr> coeffs;
        std::vector<fr_ct> coeffs_ct;
        for (size_t idx = 0; idx < 8; idx++) {
            auto el = fr::random_element();
            coeffs.emplace_back(el);
            coeffs_ct.emplace_back(fr_ct(&builder, el));
        }
        Polynomial<fr> poly(coeffs);
        fr point = fr::random_element();
        fr_ct point_ct(fr_ct(&builder, point));
        auto res1 = poly.evaluate(point);

        auto res2 = FoldingRecursiveVerifier::evaluate_perturbator(coeffs_ct, point_ct);
        EXPECT_EQ(res1, res2.get_value());
    };

    /**
     * @brief Tests that a valid recursive fold  works as expected.
     *
     */
    static void test_recursive_folding()
    {
        // Create two arbitrary circuits for the first round of folding
        InnerBuilder builder1;
        create_function_circuit(builder1);
        InnerBuilder builder2;
        builder2.add_public_variable(FF(1));
        create_function_circuit(builder2);

        auto prover_instance_1 = std::make_shared<InnerProverInstance>(builder1);
        auto verification_key_1 = std::make_shared<InnerVerificationKey>(prover_instance_1->proving_key);
        auto verifier_instance_1 = std::make_shared<InnerVerifierInstance>(verification_key_1);
        auto prover_instance_2 = std::make_shared<InnerProverInstance>(builder2);
        auto verification_key_2 = std::make_shared<InnerVerificationKey>(prover_instance_2->proving_key);
        auto verifier_instance_2 = std::make_shared<InnerVerifierInstance>(verification_key_2);
        // Generate a folding proof
        InnerFoldingProver folding_prover({ prover_instance_1, prover_instance_2 });
        auto folding_proof = folding_prover.fold_instances();

        // Create a recursive folding verifier circuit for the folding proof of the two instances
        OuterBuilder folding_circuit;
        auto verifier =
            FoldingRecursiveVerifier(&folding_circuit, verifier_instance_1, { verifier_instance_2->verification_key });
        verifier.verify_folding_proof(folding_proof.folding_data);
        info("Folding Recursive Verifier: num gates = ", folding_circuit.num_gates);
        EXPECT_EQ(folding_circuit.failed(), false) << folding_circuit.err();

        // Perform native folding verification and ensure it returns the same result (either true or false) as
        // calling check_circuit on the recursive folding verifier
        InnerFoldingVerifier native_folding_verifier({ verifier_instance_1, verifier_instance_2 });
        native_folding_verifier.verify_folding_proof(folding_proof.folding_data);

        // Ensure that the underlying native and recursive folding verification algorithms agree by ensuring the
        // manifestsproduced by each agree.
        auto recursive_folding_manifest = verifier.transcript->get_manifest();
        auto native_folding_manifest = native_folding_verifier.transcript->get_manifest();

        for (size_t i = 0; i < recursive_folding_manifest.size(); ++i) {
            EXPECT_EQ(recursive_folding_manifest[i], native_folding_manifest[i])
                << "Recursive Verifier/Verifier manifest discrepency in round " << i;
        }

        // Check for a failure flag in the recursive verifier circuit

        if constexpr (!IsSimulator<OuterBuilder>) {
            auto instance = std::make_shared<OuterProverInstance>(folding_circuit);
            OuterProver prover(instance);
            auto verification_key = std::make_shared<typename OuterFlavor::VerificationKey>(instance->proving_key);
            OuterVerifier verifier(verification_key);
            auto proof = prover.construct_proof();
            bool verified = verifier.verify_proof(proof);

            ASSERT(verified);
        }
    };

    /**
     * @brief Perform two rounds of folding valid circuits and then recursive verify the final decider proof,
     * make sure the verifer circuits pass check_circuit(). Ensure that the algorithm of the recursive and native
     * verifiers are identical by checking the manifests
     */
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/844): Fold the recursive folding verifier in
    // tests once we can fold instances of different sizes.
    static void test_full_protogalaxy_recursive()
    {
        // Create two arbitrary circuits for the first round of folding
        InnerBuilder builder1;
        create_function_circuit(builder1);
        InnerBuilder builder2;
        builder2.add_public_variable(FF(1));

        create_function_circuit(builder2);

        auto prover_instance_1 = std::make_shared<InnerProverInstance>(builder1);
        auto verification_key_1 = std::make_shared<InnerVerificationKey>(prover_instance_1->proving_key);
        auto verifier_instance_1 = std::make_shared<InnerVerifierInstance>(verification_key_1);
        auto prover_instance_2 = std::make_shared<InnerProverInstance>(builder2);
        auto verification_key_2 = std::make_shared<InnerVerificationKey>(prover_instance_2->proving_key);
        auto verifier_instance_2 = std::make_shared<InnerVerifierInstance>(verification_key_2);
        // Generate a folding proof
        InnerFoldingProver folding_prover({ prover_instance_1, prover_instance_2 });
        auto folding_proof = folding_prover.fold_instances();

        // Create a recursive folding verifier circuit for the folding proof of the two instances
        OuterBuilder folding_circuit;
        auto verifier =
            FoldingRecursiveVerifier(&folding_circuit, verifier_instance_1, { verifier_instance_2->verification_key });
        auto recursive_verifier_accumulator = verifier.verify_folding_proof(folding_proof.folding_data);
        auto native_verifier_acc = std::make_shared<InnerVerifierInstance>(recursive_verifier_accumulator->get_value());
        info("Folding Recursive Verifier: num gates = ", folding_circuit.num_gates);

        // Check for a failure flag in the recursive verifier circuit
        EXPECT_EQ(folding_circuit.failed(), false) << folding_circuit.err();

        // Perform native folding verification and ensure it returns the same result (either true or false) as
        // calling check_circuit on the recursive folding verifier
        InnerFoldingVerifier native_folding_verifier({ verifier_instance_1, verifier_instance_2 });
        auto verifier_accumulator = native_folding_verifier.verify_folding_proof(folding_proof.folding_data);

        // Ensure that the underlying native and recursive folding verification algorithms agree by ensuring the
        // manifestsproduced by each agree.
        auto recursive_folding_manifest = verifier.transcript->get_manifest();
        auto native_folding_manifest = native_folding_verifier.transcript->get_manifest();

        for (size_t i = 0; i < recursive_folding_manifest.size(); ++i) {
            EXPECT_EQ(recursive_folding_manifest[i], native_folding_manifest[i])
                << "Recursive Verifier/Verifier manifest discrepency in round " << i;
        }

        InnerDeciderProver decider_prover(folding_proof.accumulator);
        auto decider_proof = decider_prover.construct_proof();

        OuterBuilder decider_circuit;
        DeciderRecursiveVerifier decider_verifier{ &decider_circuit, native_verifier_acc };
        auto pairing_points = decider_verifier.verify_proof(decider_proof);
        info("Decider Recursive Verifier: num gates = ", decider_circuit.num_gates);
        // Check for a failure flag in the recursive verifier circuit
        EXPECT_EQ(decider_circuit.failed(), false) << decider_circuit.err();

        // Perform native verification then perform the pairing on the outputs of the recursive decider verifier and
        // check that the result agrees.
        InnerDeciderVerifier native_decider_verifier(verifier_accumulator);
        auto native_result = native_decider_verifier.verify_proof(decider_proof);
        auto recursive_result =
            native_decider_verifier.accumulator->verification_key->pcs_verification_key->pairing_check(
                pairing_points[0].get_value(), pairing_points[1].get_value());
        EXPECT_EQ(native_result, recursive_result);

        if constexpr (!IsSimulator<OuterBuilder>) {
            auto instance = std::make_shared<OuterProverInstance>(decider_circuit);
            OuterProver prover(instance);
            auto verification_key = std::make_shared<typename OuterFlavor::VerificationKey>(instance->proving_key);
            OuterVerifier verifier(verification_key);
            auto proof = prover.construct_proof();
            bool verified = verifier.verify_proof(proof);

            ASSERT(verified);
        }
    };

    static void test_tampered_decider_proof()
    {
        // Natively fold two circuits
        auto [prover_accumulator, verifier_accumulator] = fold_and_verify_native();

        // Tamper with the accumulator by changing the target sum
        verifier_accumulator->target_sum = FF::random_element();

        // Create a decider proof for the relaxed instance obtained through folding
        InnerDeciderProver decider_prover(prover_accumulator);
        auto decider_proof = decider_prover.construct_proof();

        // Create a decider verifier circuit for recursively verifying the decider proof
        OuterBuilder decider_circuit;
        DeciderRecursiveVerifier decider_verifier{ &decider_circuit, verifier_accumulator };
        decider_verifier.verify_proof(decider_proof);
        info("Decider Recursive Verifier: num gates = ", decider_circuit.num_gates);

        // We expect the decider circuit check to fail due to the bad proof
        EXPECT_FALSE(CircuitChecker::check(decider_circuit));
    };

    static void test_tampered_accumulator()
    {
        // Fold two circuits natively
        auto [prover_accumulator, verifier_accumulator] = fold_and_verify_native();

        // Create another circuit to do a second round of folding
        InnerBuilder builder;
        create_function_circuit(builder);
        auto prover_inst = std::make_shared<InnerProverInstance>(builder);
        auto verification_key = std::make_shared<InnerVerificationKey>(prover_inst->proving_key);
        auto verifier_inst = std::make_shared<InnerVerifierInstance>(verification_key);

        prover_accumulator->proving_key.polynomials.w_l[1] = FF::random_element();

        // Generate a folding proof with the incorrect polynomials which would result in the prover having the wrong
        // target sum
        InnerFoldingProver folding_prover({ prover_accumulator, prover_inst });
        auto folding_proof = folding_prover.fold_instances();

        // Create a recursive folding verifier circuit for the folding proof of the two instances with the untampered
        // commitments
        OuterBuilder folding_circuit;
        FoldingRecursiveVerifier verifier{ &folding_circuit,
                                           verifier_accumulator,
                                           { verifier_inst->verification_key } };
        auto recursive_verifier_acc = verifier.verify_folding_proof(folding_proof.folding_data);
        // Validate that the target sum between prover and verifier is now different
        EXPECT_FALSE(folding_proof.accumulator->target_sum == recursive_verifier_acc->target_sum.get_value());
    };
};

using FlavorTypes = testing::Types<GoblinUltraRecursiveFlavor_<GoblinUltraCircuitBuilder>,
                                   GoblinUltraRecursiveFlavor_<UltraCircuitBuilder>,
                                   UltraRecursiveFlavor_<UltraCircuitBuilder>,
                                   UltraRecursiveFlavor_<GoblinUltraCircuitBuilder>,
                                   UltraRecursiveFlavor_<CircuitSimulatorBN254>,
                                   GoblinUltraRecursiveFlavor_<CircuitSimulatorBN254>>;
TYPED_TEST_SUITE(ProtoGalaxyRecursiveTests, FlavorTypes);

TYPED_TEST(ProtoGalaxyRecursiveTests, InnerCircuit)
{
    TestFixture::test_circuit();
}

TYPED_TEST(ProtoGalaxyRecursiveTests, NewEvaluate)
{
    TestFixture::test_new_evaluate();
}

TYPED_TEST(ProtoGalaxyRecursiveTests, RecursiveFoldingTest)
{
    TestFixture::test_recursive_folding();
}

TYPED_TEST(ProtoGalaxyRecursiveTests, FullProtogalaxyRecursiveTest)
{

    TestFixture::test_full_protogalaxy_recursive();
}

TYPED_TEST(ProtoGalaxyRecursiveTests, TamperedDeciderProof)
{
    TestFixture::test_tampered_decider_proof();
}

TYPED_TEST(ProtoGalaxyRecursiveTests, TamperedAccumulator)
{
    TestFixture::test_tampered_accumulator();
}

} // namespace bb::stdlib::recursion::honk