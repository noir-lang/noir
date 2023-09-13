#include "barretenberg/honk/flavor/ultra_recursive.hpp"
#include "barretenberg/honk/proof_system/ultra_verifier.hpp"

#include "barretenberg/common/test.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/stdlib/hash/blake3s/blake3s.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/recursion/honk/verifier/ultra_recursive_verifier.hpp"
#include "barretenberg/stdlib/recursion/verification_key/verification_key.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace proof_system::plonk::stdlib::recursion::honk {

/**
 * @brief Test suite for Ultra Honk Recursive Verifier
 * @details Construct and check a recursive verifier circuit for an UltraHonk proof using 1) the conventional Ultra
 * arithmetization, or 2) a Goblin-style Ultra arithmetization
 *
 * @tparam UseGoblinFlag whether or not to use goblin-style arithmetization for group operations
 */
template <typename UseGoblinFlag> class RecursiveVerifierTest : public testing::Test {

    static constexpr bool goblin_flag = UseGoblinFlag::value;

    using InnerComposer = ::proof_system::honk::UltraComposer;
    using InnerBuilder = typename InnerComposer::CircuitBuilder;

    using OuterBuilder = ::proof_system::UltraCircuitBuilder;

    using NativeVerifier = ::proof_system::honk::UltraVerifier_<::proof_system::honk::flavor::Ultra>;
    // Arithmetization of group operations in recursive verifier circuit (goblin or not) is determined by goblin_flag
    using RecursiveVerifier = UltraRecursiveVerifier_<::proof_system::honk::flavor::UltraRecursive, goblin_flag>;
    using VerificationKey = ::proof_system::honk::flavor::UltraRecursive::VerificationKey;

    using inner_curve = bn254<InnerBuilder>;
    using inner_scalar_field_ct = inner_curve::ScalarField;
    using inner_ground_field_ct = inner_curve::BaseField;
    using public_witness_ct = inner_curve::public_witness_ct;
    using witness_ct = inner_curve::witness_ct;
    using byte_array_ct = inner_curve::byte_array_ct;

    using inner_scalar_field = typename inner_curve::ScalarFieldNative;

    /**
     * @brief Create an inner circuit, the proof of which will be recursively verified
     *
     * @param builder
     * @param public_inputs
     * @param log_num_gates
     */
    static void create_inner_circuit(InnerBuilder& builder,
                                     const std::vector<inner_scalar_field>& public_inputs,
                                     size_t log_num_gates = 10)
    {
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

        // Create some additional "circuity" gates as an example
        inner_scalar_field_ct a(public_witness_ct(&builder, public_inputs[0]));
        inner_scalar_field_ct b(public_witness_ct(&builder, public_inputs[1]));
        inner_scalar_field_ct c(public_witness_ct(&builder, public_inputs[2]));

        for (size_t i = 0; i < 32; ++i) {
            a = (a * b) + b + a;
            a = a.madd(b, c);
        }
        pedersen_commitment<InnerBuilder>::compress(a, b);
        byte_array_ct to_hash(&builder, "nonsense test data");
        blake3s(to_hash);

        inner_scalar_field bigfield_data = fr::random_element();
        inner_scalar_field bigfield_data_a{ bigfield_data.data[0], bigfield_data.data[1], 0, 0 };
        inner_scalar_field bigfield_data_b{ bigfield_data.data[2], bigfield_data.data[3], 0, 0 };

        inner_ground_field_ct big_a(inner_scalar_field_ct(witness_ct(&builder, bigfield_data_a.to_montgomery_form())),
                                    inner_scalar_field_ct(witness_ct(&builder, 0)));
        inner_ground_field_ct big_b(inner_scalar_field_ct(witness_ct(&builder, bigfield_data_b.to_montgomery_form())),
                                    inner_scalar_field_ct(witness_ct(&builder, 0)));

        big_a* big_b;
    };

    /**
     * @brief Create a recursive verifier circuit and perform some native consistency checks
     * @details Given an arbitrary inner circuit, construct a proof then consturct a recursive verifier circuit for that
     * proof using the provided outer circuit builder.
     * @note: The output of recursive verification is the two points which could be used in a pairing to do final
     * verification. As a consistency check, we check that the outcome of performing this pairing (natively, no
     * constraints) matches the outcome of running the full native verifier.
     *
     * @param inner_circuit Builder of the circuit for which a proof is recursively verified
     * @param outer_builder Builder for the recursive verifier circuit
     */
    static void create_outer_circuit(InnerBuilder& inner_circuit, OuterBuilder& outer_builder)
    {
        // Create proof of inner circuit
        InnerComposer inner_composer;
        auto prover = inner_composer.create_prover(inner_circuit);
        auto proof_to_recursively_verify = prover.construct_proof();

        info("Inner circuit size = ", prover.key->circuit_size);

        // Compute native verification key
        const auto native_verification_key = inner_composer.compute_verification_key(inner_circuit);

        // Instantiate the recursive verification key from the native verification key
        auto verification_key = std::make_shared<VerificationKey>(&outer_builder, native_verification_key);

        // Instantiate the recursive verifier and construct the recusive verification circuit
        RecursiveVerifier verifier(&outer_builder, verification_key);
        auto pairing_points = verifier.verify_proof(proof_to_recursively_verify);

        // For testing purposes only, perform native verification and compare the result
        auto native_verifier = inner_composer.create_verifier(inner_circuit);
        auto native_result = native_verifier.verify_proof(proof_to_recursively_verify);

        // Extract the pairing points from the recursive verifier output and perform the pairing natively. The result
        // should match that of native verification.
        auto lhs = pairing_points[0].get_value();
        auto rhs = pairing_points[1].get_value();
        auto recursive_result = native_verifier.pcs_verification_key->pairing_check(lhs, rhs);
        EXPECT_EQ(recursive_result, native_result);

        // Confirm that the manifests produced by the recursive and native verifiers agree
        auto recursive_manifest = verifier.transcript.get_manifest();
        auto native_manifest = native_verifier.transcript.get_manifest();
        // recursive_manifest.print();
        // native_manifest.print();
        for (size_t i = 0; i < recursive_manifest.size(); ++i) {
            EXPECT_EQ(recursive_manifest[i], native_manifest[i]);
        }
    };

  public:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }

    /**
     * @brief Create inner circuit and call check_circuit on it
     *
     */
    static void test_inner_circuit()
    {
        InnerBuilder builder;
        std::vector<inner_scalar_field> inputs{ inner_scalar_field::random_element(),
                                                inner_scalar_field::random_element(),
                                                inner_scalar_field::random_element() };

        create_inner_circuit(builder, inputs);

        bool result = builder.check_circuit();
        EXPECT_EQ(result, true);
    }

    /**
     * @brief Instantiate a recursive verification key from the native verification key produced by the inner cicuit
     * builder. Check consistency beteen the native and stdlib types.
     *
     */
    static void test_recursive_verification_key_creation()
    {
        InnerBuilder inner_circuit;
        OuterBuilder outer_circuit;

        std::vector<inner_scalar_field> inner_public_inputs{ inner_scalar_field::random_element(),
                                                             inner_scalar_field::random_element(),
                                                             inner_scalar_field::random_element() };

        // Create an arbitrary inner circuit
        create_inner_circuit(inner_circuit, inner_public_inputs);

        // Compute native verification key
        InnerComposer inner_composer;
        auto prover = inner_composer.create_prover(inner_circuit); // A prerequisite for computing VK
        const auto native_verification_key = inner_composer.compute_verification_key(inner_circuit);

        // Instantiate the recursive verification key from the native verification key
        auto verification_key = std::make_shared<VerificationKey>(&outer_circuit, native_verification_key);

        // Spot check some values in the recursive VK to ensure it was constructed correctly
        EXPECT_EQ(verification_key->circuit_size, native_verification_key->circuit_size);
        EXPECT_EQ(verification_key->log_circuit_size, native_verification_key->log_circuit_size);
        EXPECT_EQ(verification_key->num_public_inputs, native_verification_key->num_public_inputs);
        EXPECT_EQ(verification_key->q_m.get_value(), native_verification_key->q_m);
        EXPECT_EQ(verification_key->q_r.get_value(), native_verification_key->q_r);
        EXPECT_EQ(verification_key->sigma_1.get_value(), native_verification_key->sigma_1);
        EXPECT_EQ(verification_key->id_3.get_value(), native_verification_key->id_3);
    }

    /**
     * @brief Construct a recursive verification circuit for the proof of an inner circuit then call check_circuit on it
     *
     */
    static void test_recursive_proof_composition()
    {
        InnerBuilder inner_circuit;
        OuterBuilder outer_circuit;

        std::vector<inner_scalar_field> inner_public_inputs{ inner_scalar_field::random_element(),
                                                             inner_scalar_field::random_element(),
                                                             inner_scalar_field::random_element() };

        // Create an arbitrary inner circuit
        create_inner_circuit(inner_circuit, inner_public_inputs);

        // Create a recursive verification circuit for the proof of the inner circuit
        create_outer_circuit(inner_circuit, outer_circuit);

        EXPECT_EQ(outer_circuit.failed(), false) << outer_circuit.err();
        EXPECT_TRUE(outer_circuit.check_circuit());
    }
};

// Run the recursive verifier tests twice, once without using a goblin style arithmetization of group operations and
// once with.
using UseGoblinFlag = testing::Types<std::false_type, std::true_type>;

TYPED_TEST_SUITE(RecursiveVerifierTest, UseGoblinFlag);

HEAVY_TYPED_TEST(RecursiveVerifierTest, InnerCircuit)
{
    TestFixture::test_inner_circuit();
}

HEAVY_TYPED_TEST(RecursiveVerifierTest, RecursiveVerificationKey)
{
    TestFixture::test_recursive_verification_key_creation();
}

HEAVY_TYPED_TEST(RecursiveVerifierTest, RecursiveProofComposition)
{
    TestFixture::test_recursive_proof_composition();
};

} // namespace proof_system::plonk::stdlib::recursion::honk