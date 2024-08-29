#include "barretenberg/vm/avm/recursion/avm_recursive_verifier.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"
#include "barretenberg/vm/avm/generated/circuit_builder.hpp"
#include "barretenberg/vm/avm/generated/composer.hpp"
#include "barretenberg/vm/avm/recursion/avm_recursive_flavor.hpp"
#include "barretenberg/vm/avm/tests/helpers.test.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/helper.hpp"
#include "barretenberg/vm/avm/trace/trace.hpp"
#include <gtest/gtest.h>

namespace tests_avm {

using namespace bb;
using namespace bb::avm_trace;

class AvmRecursiveTests : public ::testing::Test {
  public:
    using RecursiveFlavor = AvmRecursiveFlavor_<UltraCircuitBuilder>;

    using InnerFlavor = typename RecursiveFlavor::NativeFlavor;
    using InnerBuilder = AvmCircuitBuilder;
    using InnerProver = AvmProver;
    using InnerVerifier = AvmVerifier;
    using InnerG1 = InnerFlavor::Commitment;
    using InnerFF = InnerFlavor::FF;

    using Transcript = InnerFlavor::Transcript;

    // Note: removed templating from eccvm one
    using RecursiveVerifier = AvmRecursiveVerifier_<RecursiveFlavor>;

    using OuterBuilder = typename RecursiveFlavor::CircuitBuilder;
    using OuterProver = UltraProver_<UltraFlavor>;
    using OuterVerifier = UltraVerifier_<UltraFlavor>;
    using OuterProverInstance = ProverInstance_<UltraFlavor>;

    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    // Generate an extremely simple avm trace
    static AvmCircuitBuilder generate_avm_circuit()
    {
        AvmTraceBuilder trace_builder(generate_base_public_inputs());
        AvmCircuitBuilder builder;

        trace_builder.op_set(0, 1, 1, AvmMemoryTag::U8);
        trace_builder.op_set(0, 1, 2, AvmMemoryTag::U8);
        trace_builder.op_add(0, 1, 2, 3, AvmMemoryTag::U8);
        trace_builder.op_return(0, 0, 0);
        auto trace = trace_builder.finalize(); // Passing true enables a longer trace with lookups

        builder.set_trace(std::move(trace));
        builder.check_circuit();
        vinfo("inner builder - num gates: ", builder.get_num_gates());

        return builder;
    }
};

TEST_F(AvmRecursiveTests, recursion)
{
    AvmCircuitBuilder circuit_builder = generate_avm_circuit();
    AvmComposer composer = AvmComposer();
    AvmProver prover = composer.create_prover(circuit_builder);
    AvmVerifier verifier = composer.create_verifier(circuit_builder);

    HonkProof proof = prover.construct_proof();

    auto public_inputs = generate_base_public_inputs();
    std::vector<std::vector<InnerFF>> public_inputs_vec =
        bb::avm_trace::copy_public_inputs_columns(public_inputs, {}, {});

    bool verified = verifier.verify_proof(proof, public_inputs_vec);
    ASSERT_TRUE(verified) << "native proof verification failed";

    // Create the outer verifier, to verify the proof
    const std::shared_ptr<AvmFlavor::VerificationKey> verification_key = verifier.key;
    OuterBuilder outer_circuit;
    RecursiveVerifier recursive_verifier{ &outer_circuit, verification_key };

    auto pairing_points = recursive_verifier.verify_proof(proof);

    bool pairing_points_valid = verification_key->pcs_verification_key->pairing_check(pairing_points[0].get_value(),
                                                                                      pairing_points[1].get_value());

    ASSERT_TRUE(pairing_points_valid) << "Pairing points are not valid.";

    vinfo("Recursive verifier: num gates = ", outer_circuit.num_gates);
    ASSERT_FALSE(outer_circuit.failed()) << "Outer circuit has failed.";

    bool outer_circuit_checked = CircuitChecker::check(outer_circuit);
    ASSERT_TRUE(outer_circuit_checked) << "outer circuit check failed";

    auto manifest = verifier.transcript->get_manifest();
    auto recursive_manifest = recursive_verifier.transcript->get_manifest();

    EXPECT_EQ(manifest.size(), recursive_manifest.size());
    for (size_t i = 0; i < recursive_manifest.size(); ++i) {
        EXPECT_EQ(recursive_manifest[i], manifest[i]);
    }

    for (auto const [key_el, rec_key_el] : zip_view(verifier.key->get_all(), recursive_verifier.key->get_all())) {
        EXPECT_EQ(key_el, rec_key_el.get_value());
    }

    EXPECT_EQ(verifier.key->circuit_size, recursive_verifier.key->circuit_size);
    EXPECT_EQ(verifier.key->num_public_inputs, recursive_verifier.key->num_public_inputs);

    // Make a proof of the verification of an AVM proof
    const size_t srs_size = 1 << 23;
    auto ultra_instance = std::make_shared<OuterProverInstance>(
        outer_circuit, TraceStructure::NONE, std::make_shared<bb::CommitmentKey<curve::BN254>>(srs_size));
    OuterProver ultra_prover(ultra_instance);
    auto ultra_verification_key = std::make_shared<UltraFlavor::VerificationKey>(ultra_instance->proving_key);
    OuterVerifier ultra_verifier(ultra_verification_key);

    auto recursion_proof = ultra_prover.construct_proof();
    bool recursion_verified = ultra_verifier.verify_proof(recursion_proof);
    EXPECT_TRUE(recursion_verified) << "recursion proof verification failed";
}
} // namespace tests_avm
