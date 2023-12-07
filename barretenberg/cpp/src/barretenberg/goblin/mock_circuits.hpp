#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/srs/global_crs.hpp"

namespace barretenberg {
class GoblinTestingUtils {
  public:
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Fbase = Curve::BaseField;
    using Point = Curve::AffineElement;
    using CommitmentKey = proof_system::honk::pcs::CommitmentKey<Curve>;
    using OpQueue = proof_system::ECCOpQueue;
    using GoblinUltraBuilder = proof_system::GoblinUltraCircuitBuilder;
    using Flavor = proof_system::honk::flavor::GoblinUltra;
    static constexpr size_t NUM_OP_QUEUE_COLUMNS = Flavor::NUM_WIRES;

    static void construct_arithmetic_circuit(GoblinUltraBuilder& builder)
    {
        // Add some arithmetic gates that utilize public inputs
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
     * @todo The transcript aggregation protocol in the Goblin proof system can not yet support an empty "previous
     * transcript" (see issue #723) because the corresponding commitments are zero / the point at infinity. This
     * function mocks the interactions with the op queue of a fictional "first" circuit. This way, when we go to
     * generate a proof over our first "real" circuit, the transcript aggregation protocol can proceed nominally. The
     * mock data is valid in the sense that it can be processed by all stages of Goblin as if it came from a genuine
     * circuit.
     *
     *
     * @param op_queue
     */
    static void perform_op_queue_interactions_for_mock_first_circuit(
        std::shared_ptr<proof_system::ECCOpQueue>& op_queue)
    {
        proof_system::GoblinUltraCircuitBuilder builder{ op_queue };

        // Add a mul accum op and an equality op
        auto point = Point::one() * FF::random_element();
        auto scalar = FF::random_element();
        builder.queue_ecc_mul_accum(point, scalar);
        builder.queue_ecc_eq();

        op_queue->set_size_data();

        // Manually compute the op queue transcript commitments (which would normally be done by the merge prover)
        auto crs_factory_ = barretenberg::srs::get_crs_factory();
        auto commitment_key = CommitmentKey(op_queue->get_current_size(), crs_factory_);
        std::array<Point, Flavor::NUM_WIRES> op_queue_commitments;
        size_t idx = 0;
        for (auto& entry : op_queue->get_aggregate_transcript()) {
            op_queue_commitments[idx++] = commitment_key.commit(entry);
        }
        // Store the commitment data for use by the prover of the next circuit
        op_queue->set_commitment_data(op_queue_commitments);
    }

    /**
     * @brief Generate a simple test circuit with some ECC op gates and conventional arithmetic gates
     *
     * @param builder
     */
    static void construct_simple_initial_circuit(GoblinUltraBuilder& builder)
    {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/800) Testing cleanup
        perform_op_queue_interactions_for_mock_first_circuit(builder.op_queue);

        // Add some arbitrary ecc op gates
        for (size_t i = 0; i < 3; ++i) {
            auto point = Point::random_element();
            auto scalar = FF::random_element();
            builder.queue_ecc_add_accum(point);
            builder.queue_ecc_mul_accum(point, scalar);
        }
        // queues the result of the preceding ECC
        builder.queue_ecc_eq(); // should be eq and reset

        construct_arithmetic_circuit(builder);
    }
};
} // namespace barretenberg