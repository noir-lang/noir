#include "barretenberg/stdlib/recursion/honk/verifier/protogalaxy_recursive_verifier.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/flavor/ultra_recursive.hpp"
#include "barretenberg/stdlib/hash/blake3s/blake3s.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/recursion/honk/verifier/decider_recursive_verifier.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

namespace bb::stdlib::recursion::honk {
class ProtogalaxyRecursiveTest : public testing::Test {
  public:
    // Define types relevant for testing
    using UltraFlavor = ::bb::honk::flavor::Ultra;
    using GoblinUltraFlavor = ::bb::honk::flavor::GoblinUltra;
    using UltraComposer = ::bb::honk::UltraComposer_<UltraFlavor>;
    using GoblinUltraComposer = ::bb::honk::UltraComposer_<GoblinUltraFlavor>;

    using InnerFlavor = UltraFlavor;
    using InnerComposer = UltraComposer;
    using Instance = ::bb::honk::ProverInstance_<InnerFlavor>;
    using InnerBuilder = typename InnerComposer::CircuitBuilder;
    using InnerCurve = bn254<InnerBuilder>;
    using Commitment = InnerFlavor::Commitment;
    using FF = InnerFlavor::FF;

    // Types for recursive verifier circuit
    // cannot do on Goblin
    using OuterBuilder = GoblinUltraCircuitBuilder;
    using RecursiveFlavor = ::bb::honk::flavor::UltraRecursive_<OuterBuilder>;
    using RecursiveVerifierInstances = ::bb::honk::VerifierInstances_<RecursiveFlavor, 2>;
    using FoldingRecursiveVerifier = ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances>;
    using DeciderRecursiveVerifier = DeciderRecursiveVerifier_<RecursiveFlavor>;
    using DeciderVerifier = ::bb::honk::DeciderVerifier_<InnerFlavor>;
    using NativeVerifierInstances = ::bb::honk::VerifierInstances_<InnerFlavor, 2>;
    using NativeFoldingVerifier = bb::honk::ProtoGalaxyVerifier_<NativeVerifierInstances>;

    // Helper for getting composer for prover/verifier of recursive (outer) circuit
    template <typename BuilderT> static auto get_outer_composer()
    {
        if constexpr (IsGoblinBuilder<BuilderT>) {
            return GoblinUltraComposer();
        } else {
            return UltraComposer();
        }
    }

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
    static void create_inner_circuit(InnerBuilder& builder, size_t log_num_gates = 10)
    {
        using fr_ct = InnerCurve::ScalarField;
        using fq_ct = InnerCurve::BaseField;
        using public_witness_ct = InnerCurve::public_witness_ct;
        using witness_ct = InnerCurve::witness_ct;
        using byte_array_ct = InnerCurve::byte_array_ct;
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

  public:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    static std::shared_ptr<Instance> fold_and_verify(const std::vector<std::shared_ptr<Instance>>& instances,
                                                     InnerComposer& inner_composer)
    {
        // Generate a folding proof
        auto inner_folding_prover = inner_composer.create_folding_prover(instances);
        auto inner_folding_proof = inner_folding_prover.fold_instances();

        // Create a recursive folding verifier circuit for the folding proof of the two instances
        OuterBuilder outer_folding_circuit;
        FoldingRecursiveVerifier verifier{ &outer_folding_circuit };
        verifier.verify_folding_proof(inner_folding_proof.folding_data);
        info("Recursive Verifier with Ultra instances: num gates = ", outer_folding_circuit.num_gates);

        // Perform native folding verification and ensure it returns the same result (either true or false) as calling
        // check_circuit on the recursive folding verifier
        auto native_folding_verifier = inner_composer.create_folding_verifier();
        auto native_folding_result = native_folding_verifier.verify_folding_proof(inner_folding_proof.folding_data);
        EXPECT_EQ(native_folding_result, outer_folding_circuit.check_circuit());

        // Ensure that the underlying native and recursive folding verification algorithms agree by ensuring
        // the manifests produced by each agree.
        auto recursive_folding_manifest = verifier.transcript->get_manifest();
        auto native_folding_manifest = native_folding_verifier.transcript->get_manifest();

        for (size_t i = 0; i < recursive_folding_manifest.size(); ++i) {
            EXPECT_EQ(recursive_folding_manifest[i], native_folding_manifest[i]);
        }

        // Check for a failure flag in the recursive verifier circuit
        EXPECT_EQ(outer_folding_circuit.failed(), false) << outer_folding_circuit.err();

        return inner_folding_proof.accumulator;
    }
};
/**
 * @brief Create inner circuit and call check_circuit on it
 *
 */
TEST_F(ProtogalaxyRecursiveTest, InnerCircuit)
{
    InnerBuilder builder;

    create_inner_circuit(builder);

    bool result = builder.check_circuit();
    EXPECT_EQ(result, true);
}

/**
 * @brief Ensure that evaluating the perturbator in the recursive folding verifier returns the same result as
 * evaluating in Polynomial class.
 *
 */
TEST_F(ProtogalaxyRecursiveTest, NewEvaluate)
{
    OuterBuilder builder;
    using fr_ct = bn254<OuterBuilder>::ScalarField;
    using fr = bn254<OuterBuilder>::ScalarFieldNative;

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
}

/**
 * @brief Tests a simple recursive fold that is valid works as expected.
 *
 */
TEST_F(ProtogalaxyRecursiveTest, RecursiveFoldingTest)
{
    // Create two arbitrary circuits for the first round of folding
    InnerBuilder builder1;

    create_inner_circuit(builder1);
    InnerBuilder builder2;
    builder2.add_public_variable(FF(1));
    create_inner_circuit(builder2);

    InnerComposer inner_composer = InnerComposer();
    auto instance1 = inner_composer.create_instance(builder1);
    auto instance2 = inner_composer.create_instance(builder2);
    auto instances = std::vector<std::shared_ptr<Instance>>{ instance1, instance2 };

    fold_and_verify(instances, inner_composer);
}

/**
 * @brief Recursively verify two rounds of folding valid circuits and then recursive verify the final decider proof,
 * make sure the verifer circuits pass check_circuit(). Ensure that the algorithm of the recursive and native verifiers
 * are identical by checking the manifests

 */
TEST_F(ProtogalaxyRecursiveTest, FullProtogalaxyRecursiveTest)
{

    // Create two arbitrary circuits for the first round of folding
    InnerBuilder builder1;

    create_inner_circuit(builder1);
    InnerBuilder builder2;
    builder2.add_public_variable(FF(1));
    create_inner_circuit(builder2);

    InnerComposer inner_composer = InnerComposer();
    auto instance1 = inner_composer.create_instance(builder1);
    auto instance2 = inner_composer.create_instance(builder2);
    auto instances = std::vector<std::shared_ptr<Instance>>{ instance1, instance2 };

    auto accumulator = fold_and_verify(instances, inner_composer);

    // Create another circuit to do a second round of folding
    InnerBuilder builder3;
    create_inner_circuit(builder3);
    auto instance3 = inner_composer.create_instance(builder3);
    instances = std::vector<std::shared_ptr<Instance>>{ accumulator, instance3 };

    accumulator = fold_and_verify(instances, inner_composer);

    // Create a decider proof for the relaxed instance obtained through folding
    auto inner_decider_prover = inner_composer.create_decider_prover(accumulator);
    auto inner_decider_proof = inner_decider_prover.construct_proof();

    // Create a decider verifier circuit for recursively verifying the decider proof
    OuterBuilder outer_decider_circuit;
    DeciderRecursiveVerifier decider_verifier{ &outer_decider_circuit };
    auto pairing_points = decider_verifier.verify_proof(inner_decider_proof);
    info("Decider Recursive Verifier: num gates = ", outer_decider_circuit.num_gates);
    // Check for a failure flag in the recursive verifier circuit
    EXPECT_EQ(outer_decider_circuit.failed(), false) << outer_decider_circuit.err();

    // Perform native verification then perform the pairing on the outputs of the recursive
    //  decider verifier and check that the result agrees.
    DeciderVerifier native_decider_verifier = inner_composer.create_decider_verifier(accumulator);
    auto native_result = native_decider_verifier.verify_proof(inner_decider_proof);
    auto recursive_result = native_decider_verifier.pcs_verification_key->pairing_check(pairing_points[0].get_value(),
                                                                                        pairing_points[1].get_value());
    EXPECT_EQ(native_result, recursive_result);

    // Ensure that the underlying native and recursive decider verification algorithms agree by ensuring
    // the manifests produced are the same.
    auto recursive_decider_manifest = decider_verifier.transcript->get_manifest();
    auto native_decider_manifest = native_decider_verifier.transcript->get_manifest();
    for (size_t i = 0; i < recursive_decider_manifest.size(); ++i) {
        EXPECT_EQ(recursive_decider_manifest[i], native_decider_manifest[i]);
    }

    // Construct and verify a proof of the recursive decider verifier circuit
    {
        auto composer = get_outer_composer<OuterBuilder>();
        auto instance = composer.create_instance(outer_decider_circuit);
        auto prover = composer.create_prover(instance);
        auto verifier = composer.create_verifier(instance);
        auto proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);

        ASSERT(verified);
    }
}

TEST_F(ProtogalaxyRecursiveTest, TamperedDeciderProof)
{
    // Create two arbitrary circuits for the first round of folding
    InnerBuilder builder1;

    create_inner_circuit(builder1);
    InnerBuilder builder2;
    builder2.add_public_variable(FF(1));
    create_inner_circuit(builder2);

    InnerComposer inner_composer = InnerComposer();
    auto instance1 = inner_composer.create_instance(builder1);
    auto instance2 = inner_composer.create_instance(builder2);
    auto instances = std::vector<std::shared_ptr<Instance>>{ instance1, instance2 };

    auto accumulator = fold_and_verify(instances, inner_composer);

    // Tamper with the accumulator by changing the target sum
    accumulator->target_sum = FF::random_element();

    // Create a decider proof for the relaxed instance obtained through folding
    auto inner_decider_prover = inner_composer.create_decider_prover(accumulator);
    auto inner_decider_proof = inner_decider_prover.construct_proof();

    // Create a decider verifier circuit for recursively verifying the decider proof
    OuterBuilder outer_decider_circuit;
    DeciderRecursiveVerifier decider_verifier{ &outer_decider_circuit };
    decider_verifier.verify_proof(inner_decider_proof);
    info("Decider Recursive Verifier: num gates = ", outer_decider_circuit.num_gates);

    // We expect the decider circuit check to fail due to the bad proof
    EXPECT_FALSE(outer_decider_circuit.check_circuit());
}

TEST_F(ProtogalaxyRecursiveTest, TamperedAccumulator)
{
    // Create two arbitrary circuits for the first round of folding
    InnerBuilder builder1;

    create_inner_circuit(builder1);
    InnerBuilder builder2;
    builder2.add_public_variable(FF(1));
    create_inner_circuit(builder2);

    InnerComposer inner_composer = InnerComposer();
    auto instance1 = inner_composer.create_instance(builder1);
    auto instance2 = inner_composer.create_instance(builder2);
    auto instances = std::vector<std::shared_ptr<Instance>>{ instance1, instance2 };

    auto accumulator = fold_and_verify(instances, inner_composer);

    // Create another circuit to do a second round of folding
    InnerBuilder builder3;
    create_inner_circuit(builder3);
    auto instance3 = inner_composer.create_instance(builder3);

    // Tamper with the accumulator
    instances = std::vector<std::shared_ptr<Instance>>{ accumulator, instance3 };
    accumulator->prover_polynomials.w_l[1] = FF::random_element();

    // Generate a folding proof
    auto inner_folding_prover = inner_composer.create_folding_prover(instances);
    auto inner_folding_proof = inner_folding_prover.fold_instances();

    // Create a recursive folding verifier circuit for the folding proof of the two instances
    OuterBuilder outer_folding_circuit;
    FoldingRecursiveVerifier verifier{ &outer_folding_circuit };
    verifier.verify_folding_proof(inner_folding_proof.folding_data);
    EXPECT_EQ(outer_folding_circuit.check_circuit(), false);
}

} // namespace bb::stdlib::recursion::honk