#include "barretenberg/common/test.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/flavor/ultra_recursive.hpp"
#include "barretenberg/honk/proof_system/ultra_verifier.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/hash/blake3s/blake3s.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/recursion/honk/verifier/ultra_recursive_verifier.hpp"

namespace proof_system::plonk::stdlib::recursion::honk {

/**
 * @brief Test suite for recursive verification of conventional Ultra Honk proofs
 * @details The recursive verification circuit is arithmetized in two different ways: 1) using the conventional Ultra
 * arithmetization (UltraCircuitBuilder), or 2) a Goblin-style Ultra arithmetization (GoblinUltraCircuitBuilder).
 *
 * @tparam Builder Circuit builder for the recursive verifier circuit
 */
template <typename BuilderType> class GoblinRecursiveVerifierTest : public testing::Test {

    // Define types relevant for inner circuit
    using Flavor = ::proof_system::honk::flavor::GoblinUltra;
    using InnerComposer = ::proof_system::honk::UltraComposer_<Flavor>;
    using InnerBuilder = typename InnerComposer::CircuitBuilder;
    using NativeVerifier = ::proof_system::honk::UltraVerifier_<::proof_system::honk::flavor::Ultra>;
    using InnerCurve = bn254<InnerBuilder>;

    // Types for recursive verifier circuit
    using RecursiveFlavor = ::proof_system::honk::flavor::GoblinUltraRecursive_<BuilderType>;
    using RecursiveVerifier = UltraRecursiveVerifier_<RecursiveFlavor>;
    using OuterBuilder = BuilderType;
    using VerificationKey = typename RecursiveVerifier::VerificationKey;

    /**
     * @brief Create a non-trivial arbitrary inner circuit, the proof of which will be recursively verified
     *
     * @param builder
     * @param public_inputs
     * @param log_num_gates
     */
    static InnerBuilder create_inner_circuit(size_t log_num_gates = 10)
    {
        using fr_ct = InnerCurve::ScalarField;
        using fq_ct = InnerCurve::BaseField;
        using point_ct = InnerCurve::AffineElement;
        using public_witness_ct = InnerCurve::public_witness_ct;
        using witness_ct = InnerCurve::witness_ct;
        using byte_array_ct = InnerCurve::byte_array_ct;
        using fr = typename InnerCurve::ScalarFieldNative;
        using point = typename InnerCurve::GroupNative::affine_element;

        // Instantiate ECC op queue and add mock data to simulate interaction with a previous circuit
        auto op_queue = std::make_shared<ECCOpQueue>();
        op_queue->populate_with_mock_initital_data();

        InnerBuilder builder(op_queue);

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

        // Add some arbitrary goblin-style ECC op gates via a batch mul
        size_t num_points = 5;
        std::vector<point_ct> circuit_points;
        std::vector<fr_ct> circuit_scalars;
        for (size_t i = 0; i < num_points; ++i) {
            circuit_points.push_back(point_ct::from_witness(&builder, point::random_element()));
            circuit_scalars.push_back(fr_ct::from_witness(&builder, fr::random_element()));
        }
        point_ct::batch_mul(circuit_points, circuit_scalars);

        // Define some additional arbitrary convetional circuit logic
        fr_ct a(public_witness_ct(&builder, fr::random_element()));
        fr_ct b(public_witness_ct(&builder, fr::random_element()));
        fr_ct c(public_witness_ct(&builder, fr::random_element()));

        for (size_t i = 0; i < 32; ++i) {
            a = (a * b) + b + a;
            a = a.madd(b, c);
        }
        pedersen_commitment<InnerBuilder>::compress(a, b);
        byte_array_ct to_hash(&builder, "nonsense test data");
        blake3s(to_hash);

        fr bigfield_data = fr::random_element();
        fr bigfield_data_a{ bigfield_data.data[0], bigfield_data.data[1], 0, 0 };
        fr bigfield_data_b{ bigfield_data.data[2], bigfield_data.data[3], 0, 0 };

        fq_ct big_a(fr_ct(witness_ct(&builder, bigfield_data_a.to_montgomery_form())), fr_ct(witness_ct(&builder, 0)));
        fq_ct big_b(fr_ct(witness_ct(&builder, bigfield_data_b.to_montgomery_form())), fr_ct(witness_ct(&builder, 0)));

        big_a* big_b;

        return builder;
    };

  public:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }

    /**
     * @brief Create inner circuit and call check_circuit on it
     *
     */
    static void test_inner_circuit()
    {
        auto inner_circuit = create_inner_circuit();

        bool result = inner_circuit.check_circuit();
        EXPECT_EQ(result, true);
    }

    /**
     * @brief Instantiate a recursive verification key from the native verification key produced by the inner cicuit
     * builder. Check consistency beteen the native and stdlib types.
     *
     */
    static void test_recursive_verification_key_creation()
    {
        // Create an arbitrary inner circuit
        auto inner_circuit = create_inner_circuit();

        // Compute native verification key
        InnerComposer inner_composer;
        auto instance = inner_composer.create_instance(inner_circuit);
        auto prover = inner_composer.create_prover(instance); // A prerequisite for computing VK
        const auto native_verification_key = instance->compute_verification_key();

        // Instantiate the recursive verification key from the native verification key
        OuterBuilder outer_circuit;
        auto verification_key = std::make_shared<VerificationKey>(&outer_circuit, native_verification_key);

        // Spot check some values in the recursive VK to ensure it was constructed correctly
        EXPECT_EQ(verification_key->circuit_size, native_verification_key->circuit_size);
        EXPECT_EQ(verification_key->log_circuit_size, native_verification_key->log_circuit_size);
        EXPECT_EQ(verification_key->num_public_inputs, native_verification_key->num_public_inputs);
        EXPECT_EQ(verification_key->q_m.get_value(), native_verification_key->q_m);
        EXPECT_EQ(verification_key->q_r.get_value(), native_verification_key->q_r);
        EXPECT_EQ(verification_key->sigma_1.get_value(), native_verification_key->sigma_1);
        EXPECT_EQ(verification_key->id_3.get_value(), native_verification_key->id_3);
        EXPECT_EQ(verification_key->lagrange_ecc_op.get_value(), native_verification_key->lagrange_ecc_op);
    }

    /**
     * @brief Construct a recursive verification circuit for the proof of an inner circuit then call check_circuit on it
     *
     */
    static void test_recursive_verification()
    {
        // Create an arbitrary inner circuit
        auto inner_circuit = create_inner_circuit();

        // Generate a proof over the inner circuit
        InnerComposer inner_composer;
        auto instance = inner_composer.create_instance(inner_circuit);
        auto inner_prover = inner_composer.create_prover(instance);
        auto inner_proof = inner_prover.construct_proof();
        const auto native_verification_key = instance->compute_verification_key();

        // Create a recursive verification circuit for the proof of the inner circuit
        OuterBuilder outer_circuit;
        auto verification_key = std::make_shared<VerificationKey>(&outer_circuit, native_verification_key);
        RecursiveVerifier verifier(&outer_circuit, verification_key);
        auto pairing_points = verifier.verify_proof(inner_proof);

        // Check the recursive verifier circuit
        EXPECT_EQ(outer_circuit.failed(), false) << outer_circuit.err();
        EXPECT_TRUE(outer_circuit.check_circuit());

        // Additional check 1: Perform native verification then perform the pairing on the outputs of the recursive
        // verifier and check that the result agrees.
        auto native_verifier = inner_composer.create_verifier(instance);
        auto native_result = native_verifier.verify_proof(inner_proof);
        auto recursive_result = native_verifier.pcs_verification_key->pairing_check(pairing_points[0].get_value(),
                                                                                    pairing_points[1].get_value());
        EXPECT_EQ(recursive_result, native_result);

        // Additional check 2: Ensure that the underlying native and recursive verification algorithms agree by ensuring
        // the manifests produced by each agree.
        auto recursive_manifest = verifier.transcript.get_manifest();
        auto native_manifest = native_verifier.transcript.get_manifest();
        // recursive_manifest.print();
        // native_manifest.print();
        for (size_t i = 0; i < recursive_manifest.size(); ++i) {
            EXPECT_EQ(recursive_manifest[i], native_manifest[i]);
        }
    }
};

// Run the recursive verifier tests with conventional Ultra builder and Goblin builder
using BuilderTypes = testing::Types<UltraCircuitBuilder, GoblinUltraCircuitBuilder>;

TYPED_TEST_SUITE(GoblinRecursiveVerifierTest, BuilderTypes);

HEAVY_TYPED_TEST(GoblinRecursiveVerifierTest, InnerCircuit)
{
    TestFixture::test_inner_circuit();
}

HEAVY_TYPED_TEST(GoblinRecursiveVerifierTest, RecursiveVerificationKey)
{
    TestFixture::test_recursive_verification_key_creation();
}

HEAVY_TYPED_TEST(GoblinRecursiveVerifierTest, SingleRecursiveVerification)
{
    TestFixture::test_recursive_verification();
};

} // namespace proof_system::plonk::stdlib::recursion::honk