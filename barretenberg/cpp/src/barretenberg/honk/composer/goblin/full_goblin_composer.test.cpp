#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/honk/composer/eccvm_composer.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/proof_system/circuit_builder/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

namespace test_full_goblin_composer {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

class FullGoblinComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite()
    {
        barretenberg::srs::init_crs_factory("../srs_db/ignition");
        barretenberg::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Point = Curve::AffineElement;
    using CommitmentKey = proof_system::honk::pcs::CommitmentKey<Curve>;
    using GoblinUltraBuilder = proof_system::GoblinUltraCircuitBuilder;
    using GoblinUltraComposer = proof_system::honk::GoblinUltraComposer;
    using ECCVMFlavor = proof_system::honk::flavor::ECCVMGrumpkin;
    using ECCVMBuilder = proof_system::ECCVMCircuitBuilder<ECCVMFlavor>;
    using ECCVMComposer = proof_system::honk::ECCVMComposer_<ECCVMFlavor>;
    using VMOp = proof_system_eccvm::VMOperation<ECCVMFlavor::CycleGroup>;
    static constexpr size_t NUM_OP_QUEUE_COLUMNS = proof_system::honk::flavor::GoblinUltra::NUM_WIRES;

    /**
     * @brief Generate a simple test circuit with some ECC op gates and conventional arithmetic gates
     *
     * @param builder
     */
    void generate_test_circuit(auto& builder)
    {
        // Add some arbitrary ecc op gates
        for (size_t i = 0; i < 3; ++i) {
            auto point = Point::random_element();
            auto scalar = FF::random_element();
            builder.queue_ecc_add_accum(point);
            builder.queue_ecc_mul_accum(point, scalar);
        }
        builder.queue_ecc_eq();

        // Add some conventional gates that utilize public inputs
        for (size_t i = 0; i < 10; ++i) {
            FF a = FF::random_element();
            FF b = FF::random_element();
            FF c = FF::random_element();
            FF d = a + b + c;
            uint32_t a_idx = builder.add_public_variable(a);
            uint32_t b_idx = builder.add_variable(b);
            uint32_t c_idx = builder.add_variable(c);
            uint32_t d_idx = builder.add_variable(d);

            builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, FF(1), FF(1), FF(1), FF(-1), FF(0) });
        }
    }

    /**
     * @brief Mock the interactions of a simple curcuit with the op_queue
     * @details The transcript aggregation protocol in the Goblin proof system can not yet support an empty "previous
     * transcript" (see issue #723). This function mocks the interactions with the op queue of a fictional "first"
     * circuit. This way, when we go to generate a proof over our first "real" circuit, the transcript aggregation
     * protocol can proceed nominally. The mock data is valid in the sense that it can be processed by all stages of
     * Goblin as if it came from a genuine circuit.
     *
     * @param op_queue
     */
    static void perform_op_queue_interactions_for_mock_first_circuit(
        std::shared_ptr<proof_system::ECCOpQueue>& op_queue)
    {
        auto builder = GoblinUltraBuilder(op_queue);

        // Add a mul accum op and an equality op
        auto point = Point::one() * FF::random_element();
        auto scalar = FF::random_element();
        builder.queue_ecc_mul_accum(point, scalar);
        builder.queue_ecc_eq();

        op_queue->set_size_data();

        // Manually compute the op queue transcript commitments (which would normally be done by the prover)
        auto crs_factory_ = barretenberg::srs::get_crs_factory();
        auto commitment_key = CommitmentKey(op_queue->get_current_size(), crs_factory_);
        std::array<Point, NUM_OP_QUEUE_COLUMNS> op_queue_commitments;
        size_t idx = 0;
        for (auto& entry : op_queue->get_aggregate_transcript()) {
            op_queue_commitments[idx++] = commitment_key.commit(entry);
        }
        // Store the commitment data for use by the prover of the next circuit
        op_queue->set_commitment_data(op_queue_commitments);
    }
};

/**
 * @brief Test proof construction/verification for a circuit with ECC op gates, public inputs, and basic arithmetic
 * gates
 * @note We simulate op queue interactions with a previous circuit so the actual circuit under test utilizes an op queue
 * with non-empty 'previous' data. This avoid complications with zero-commitments etc.
 *
 */
TEST_F(FullGoblinComposerTests, SimpleCircuit)
{
    auto op_queue = std::make_shared<proof_system::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a "first" circuit
    perform_op_queue_interactions_for_mock_first_circuit(op_queue);

    // Construct a series of simple Goblin circuits; generate and verify their proofs
    size_t NUM_CIRCUITS = 3;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        auto builder = GoblinUltraBuilder(op_queue);

        generate_test_circuit(builder);

        auto composer = GoblinUltraComposer();
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto verifier = composer.create_verifier(instance);
        auto proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);
        EXPECT_EQ(verified, true);
    }

    // Construct an ECCVM circuit then generate and verify its proof
    {
        // Instantiate an ECCVM builder with the vm ops stored in the op queue
        auto builder = ECCVMBuilder(op_queue->raw_ops);

        // // Can fiddle with one of the operands to trigger a failure
        // builder.vm_operations[0].z1 *= 2;

        auto composer = ECCVMComposer();
        auto prover = composer.create_prover(builder);
        auto proof = prover.construct_proof();
        auto verifier = composer.create_verifier(builder);
        bool verified = verifier.verify_proof(proof);
        ASSERT_TRUE(verified);
    }
}

/**
 * @brief Check that ECCVM verification fails if ECC op queue operands are tampered with
 *
 */
TEST_F(FullGoblinComposerTests, SimpleCircuitFailureCase)
{
    auto op_queue = std::make_shared<proof_system::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a "first" circuit
    perform_op_queue_interactions_for_mock_first_circuit(op_queue);

    // Construct a series of simple Goblin circuits; generate and verify their proofs
    size_t NUM_CIRCUITS = 3;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        auto builder = GoblinUltraBuilder(op_queue);

        generate_test_circuit(builder);

        auto composer = GoblinUltraComposer();
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto verifier = composer.create_verifier(instance);
        auto proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);
        EXPECT_EQ(verified, true);
    }

    // Construct an ECCVM circuit then generate and verify its proof
    {
        // Instantiate an ECCVM builder with the vm ops stored in the op queue
        auto builder = ECCVMBuilder(op_queue->raw_ops);

        // Fiddle with one of the operands to trigger a failure
        builder.vm_operations[0].z1 += 1;

        auto composer = ECCVMComposer();
        auto prover = composer.create_prover(builder);
        auto proof = prover.construct_proof();
        auto verifier = composer.create_verifier(builder);
        bool verified = verifier.verify_proof(proof);
        EXPECT_EQ(verified, false);
    }
}

} // namespace test_full_goblin_composer
