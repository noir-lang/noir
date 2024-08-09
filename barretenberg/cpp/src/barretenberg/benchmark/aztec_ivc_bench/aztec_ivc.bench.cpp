
#include <benchmark/benchmark.h>

#include "barretenberg/aztec_ivc/aztec_ivc.hpp"
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/common/op_count_google_bench.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

using namespace benchmark;
using namespace bb;

namespace {

/**
 * @brief Benchmark suite for the aztec client PG-Goblin IVC scheme
 *
 */
class AztecIVCBench : public benchmark::Fixture {
  public:
    using Builder = MegaCircuitBuilder;
    using VerifierInstance = VerifierInstance_<MegaFlavor>;
    using Proof = AztecIVC::Proof;

    // Number of function circuits to accumulate(based on Zacs target numbers)
    static constexpr size_t NUM_ITERATIONS_MEDIUM_COMPLEXITY = 6;

    void SetUp([[maybe_unused]] const ::benchmark::State& state) override
    {
        bb::srs::init_crs_factory("../srs_db/ignition");
        bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    /**
     * @brief Verify an IVC proof
     *
     */
    static bool verify_ivc(Proof& proof, AztecIVC& ivc)
    {
        auto verifier_inst = std::make_shared<VerifierInstance>(ivc.verification_queue[0].instance_vk);
        bool verified = ivc.verify(proof, { ivc.verifier_accumulator, verifier_inst });

        // This is a benchmark, not a test, so just print success or failure to the log
        if (verified) {
            info("IVC successfully verified!");
        } else {
            info("IVC failed to verify.");
        }
        return verified;
    }

    /**
     * @brief Precompute the verification keys for the bench given the number of circuits in the IVC
     *
     * @param ivc
     * @param num_function_circuits
     * @return auto
     */
    static auto precompute_verification_keys(AztecIVC& ivc, const size_t num_circuits)
    {
        // Produce the set of mocked circuits to be accumulated
        MockCircuitMaker mock_circuit_maker;
        std::vector<Builder> circuits;
        for (size_t circuit_idx = 0; circuit_idx < num_circuits; ++circuit_idx) {
            circuits.emplace_back(mock_circuit_maker.create_next_circuit(ivc));
        }

        // Compute and return the corresponding set of verfication keys
        return ivc.precompute_folding_verification_keys(circuits);
    }

    /**
     * @brief Manage the construction of mock app/kernel circuits
     * @details Per the medium complexity benchmark spec, the first app circuit is size 2^19. Subsequent app and kernel
     * circuits are size 2^17. Circuits produced are alternatingly app and kernel.
     */
    class MockCircuitMaker {
        size_t circuit_counter = 0;

      public:
        Builder create_next_circuit(AztecIVC& ivc)
        {
            circuit_counter++;

            bool is_kernel = (circuit_counter % 2 == 0); // Every other circuit is a kernel, starting from the second

            Builder circuit{ ivc.goblin.op_queue };
            if (is_kernel) { // construct mock kernel
                GoblinMockCircuits::construct_mock_folding_kernel(circuit);
            } else { // construct mock app
                bool use_large_circuit = (circuit_counter == 1);
                GoblinMockCircuits::construct_mock_app_circuit(circuit, use_large_circuit);
            }
            return circuit;
        }
    };

    /**
     * @brief Perform a specified number of circuit accumulation rounds
     *
     * @param NUM_CIRCUITS Number of circuits to accumulate (apps + kernels)
     */
    static void perform_ivc_accumulation_rounds(size_t NUM_CIRCUITS, AztecIVC& ivc, auto& precomputed_vks)
    {
        ASSERT(precomputed_vks.size() == NUM_CIRCUITS); // ensure presence of a precomputed VK for each circuit

        MockCircuitMaker mock_circuit_maker;

        for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
            Builder circuit;
            {
                BB_OP_COUNT_TIME_NAME("construct_circuits");
                circuit = mock_circuit_maker.create_next_circuit(ivc);
            }

            ivc.accumulate(circuit, precomputed_vks[circuit_idx]);
        }
    }
};

/**
 * @brief Benchmark the prover work for the full PG-Goblin IVC protocol
 *
 */
BENCHMARK_DEFINE_F(AztecIVCBench, FullStructured)(benchmark::State& state)
{
    AztecIVC ivc;
    ivc.trace_structure = TraceStructure::AZTEC_IVC_BENCH;

    auto total_num_circuits = 2 * static_cast<size_t>(state.range(0)); // 2x accounts for kernel circuits

    // Precompute the verification keys for the benchmark circuits
    auto precomputed_vkeys = precompute_verification_keys(ivc, total_num_circuits);

    Proof proof;
    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        perform_ivc_accumulation_rounds(total_num_circuits, ivc, precomputed_vkeys);
        proof = ivc.prove();
    }

    // For good measure, ensure the IVC verifies
    verify_ivc(proof, ivc);
}

#define ARGS Arg(AztecIVCBench::NUM_ITERATIONS_MEDIUM_COMPLEXITY)

BENCHMARK_REGISTER_F(AztecIVCBench, FullStructured)->Unit(benchmark::kMillisecond)->ARGS;

} // namespace

BENCHMARK_MAIN();
