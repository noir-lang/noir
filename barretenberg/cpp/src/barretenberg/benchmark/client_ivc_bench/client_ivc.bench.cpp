
#include <benchmark/benchmark.h>

#include "barretenberg/client_ivc/client_ivc.hpp"
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
class ClientIVCBench : public benchmark::Fixture {
  public:
    using Builder = MegaCircuitBuilder;

    // Number of function circuits to accumulate(based on Zacs target numbers)
    static constexpr size_t NUM_ITERATIONS_MEDIUM_COMPLEXITY = 6;

    void SetUp([[maybe_unused]] const ::benchmark::State& state) override
    {
        bb::srs::init_crs_factory("../srs_db/ignition");
        bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    /**
     * @brief Compute verification key for each circuit in the IVC based on the number of desired function circuits
     * @details Assumes the following circuit ordering: one initial function circuit followed by pairs of {function,
     * kernel} until the desired number of function circuits has been reached.
     *
     * @param ivc
     * @param num_function_circuits
     */
    static auto precompute_verification_keys(ClientIVC& ivc, const size_t num_function_circuits)
    {
        // Populate the set of mock function and kernel circuits to be accumulated in the IVC
        std::vector<Builder> circuits;
        Builder function_circuit{ ivc.goblin.op_queue };
        GoblinMockCircuits::construct_mock_function_circuit(function_circuit);
        circuits.emplace_back(function_circuit);

        for (size_t idx = 1; idx < num_function_circuits; ++idx) {
            Builder function_circuit{ ivc.goblin.op_queue };
            GoblinMockCircuits::construct_mock_function_circuit(function_circuit);
            circuits.emplace_back(function_circuit);

            Builder kernel_circuit{ ivc.goblin.op_queue };
            GoblinMockCircuits::construct_mock_folding_kernel(kernel_circuit);
            circuits.emplace_back(kernel_circuit);
        }

        // Compute and return the verfication keys corresponding to this set of circuits
        return ivc.precompute_folding_verification_keys(circuits);
    }

    /**
     * @brief Perform a specified number of function circuit accumulation rounds
     * @details Each round "accumulates" a mock function circuit and a mock kernel circuit. Each round thus consists of
     * the generation of two circuits, two folding proofs and two Merge proofs. To match the sizes called out in the
     * spec
     * (https://github.com/AztecProtocol/aztec-packages/blob/master/yellow-paper/docs/cryptography/performance-targets.md)
     * we set the size of the function circuit to be 2^17. The first one should be 2^19 but we can't currently support
     * folding circuits of unequal size.
     *
     * @param NUM_CIRCUITS Number of function circuits to accumulate
     */
    static void perform_ivc_accumulation_rounds(size_t NUM_CIRCUITS, ClientIVC& ivc, auto& precomputed_vks)
    {
        size_t TOTAL_NUM_CIRCUITS = NUM_CIRCUITS * 2 - 1;     // need one less kernel than number of function circuits
        ASSERT(precomputed_vks.size() == TOTAL_NUM_CIRCUITS); // ensure presence of a precomputed VK for each circuit

        const size_t size_hint = 1 << 17; // Size hint for reserving wires/selector vector memory in builders
        std::vector<Builder> initial_function_circuits(2);

        // Construct 2 starting function circuits in parallel
        {
            BB_OP_COUNT_TIME_NAME("construct_circuits");
            parallel_for(2, [&](size_t circuit_index) {
                GoblinMockCircuits::construct_mock_function_circuit(initial_function_circuits[circuit_index]);
            });
        };

        // Prepend queue to the first circuit
        initial_function_circuits[0].op_queue->prepend_previous_queue(*ivc.goblin.op_queue);
        // Initialize ivc
        ivc.accumulate(initial_function_circuits[0], precomputed_vks[0]);
        // Retrieve the queue
        std::swap(*ivc.goblin.op_queue, *initial_function_circuits[0].op_queue);

        // Prepend queue to the second circuit
        initial_function_circuits[1].op_queue->prepend_previous_queue(*ivc.goblin.op_queue);
        // Accumulate another function circuit
        ivc.accumulate(initial_function_circuits[1], precomputed_vks[1]);
        // Retrieve the queue
        std::swap(*ivc.goblin.op_queue, *initial_function_circuits[1].op_queue);

        // Free memory
        initial_function_circuits.clear();

        for (size_t circuit_idx = 2; circuit_idx < TOTAL_NUM_CIRCUITS - 1; circuit_idx += 2) {
            Builder kernel_circuit{ size_hint, ivc.goblin.op_queue };
            Builder function_circuit{ size_hint };
            // Construct function and kernel circuits in parallel
            {
                BB_OP_COUNT_TIME_NAME("construct_circuits");
                parallel_for(2, [&](size_t workload_idx) {
                    // workload index is 0 for kernel and 1 for function
                    if (workload_idx == 0) {
                        GoblinMockCircuits::construct_mock_folding_kernel(kernel_circuit);
                    } else {
                        GoblinMockCircuits::construct_mock_function_circuit(function_circuit);
                    }
                });
            };

            // No need to prepend queue, it's the same after last swap
            // Accumulate kernel circuit
            ivc.accumulate(kernel_circuit, precomputed_vks[circuit_idx]);

            // Prepend queue to function circuit
            function_circuit.op_queue->prepend_previous_queue(*ivc.goblin.op_queue);

            // Accumulate function circuit
            ivc.accumulate(function_circuit, precomputed_vks[circuit_idx + 1]);

            // Retrieve queue
            std::swap(*ivc.goblin.op_queue, *function_circuit.op_queue);
        }

        // Final kernel
        Builder kernel_circuit{ size_hint, ivc.goblin.op_queue };
        {
            BB_OP_COUNT_TIME_NAME("construct_circuits");
            GoblinMockCircuits::construct_mock_folding_kernel(kernel_circuit);
        }
        ivc.accumulate(kernel_circuit, precomputed_vks.back());
    }
};

/**
 * @brief Benchmark the prover work for the full PG-Goblin IVC protocol
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, Full)(benchmark::State& state)
{
    ClientIVC ivc;

    auto num_circuits = static_cast<size_t>(state.range(0));
    auto precomputed_vks = precompute_verification_keys(ivc, num_circuits);

    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        // Perform a specified number of iterations of function/kernel accumulation
        perform_ivc_accumulation_rounds(num_circuits, ivc, precomputed_vks);

        // Construct IVC scheme proof (fold, decider, merge, eccvm, translator)
        ivc.prove();
    }
}

/**
 * @brief Benchmark the prover work for the full PG-Goblin IVC protocol
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, FullStructured)(benchmark::State& state)
{
    ClientIVC ivc;
    ivc.trace_structure = TraceStructure::CLIENT_IVC_BENCH;

    auto num_circuits = static_cast<size_t>(state.range(0));
    auto precomputed_vks = precompute_verification_keys(ivc, num_circuits);

    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        // Perform a specified number of iterations of function/kernel accumulation
        perform_ivc_accumulation_rounds(num_circuits, ivc, precomputed_vks);

        // Construct IVC scheme proof (fold, decider, merge, eccvm, translator)
        ivc.prove();
    }
}

/**
 * @brief Benchmark only the accumulation rounds
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, Accumulate)(benchmark::State& state)
{
    ClientIVC ivc;

    auto num_circuits = static_cast<size_t>(state.range(0));
    auto precomputed_vks = precompute_verification_keys(ivc, num_circuits);

    // Perform a specified number of iterations of function/kernel accumulation
    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        perform_ivc_accumulation_rounds(num_circuits, ivc, precomputed_vks);
    }
}

/**
 * @brief Benchmark only the Decider component
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, Decide)(benchmark::State& state)
{
    ClientIVC ivc;

    auto num_circuits = static_cast<size_t>(state.range(0));
    auto precomputed_vks = precompute_verification_keys(ivc, num_circuits);

    // Perform a specified number of iterations of function/kernel accumulation
    perform_ivc_accumulation_rounds(num_circuits, ivc, precomputed_vks);

    // Construct eccvm proof, measure only translator proof construction
    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        ivc.decider_prove();
    }
}

/**
 * @brief Benchmark only the ECCVM component
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, ECCVM)(benchmark::State& state)
{
    ClientIVC ivc;

    auto num_circuits = static_cast<size_t>(state.range(0));
    auto precomputed_vks = precompute_verification_keys(ivc, num_circuits);

    // Perform a specified number of iterations of function/kernel accumulation
    perform_ivc_accumulation_rounds(num_circuits, ivc, precomputed_vks);

    // Construct and measure eccvm only
    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        ivc.goblin.prove_eccvm();
    }
}

/**
 * @brief Benchmark only the Translator component
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, Translator)(benchmark::State& state)
{
    ClientIVC ivc;
    auto num_circuits = static_cast<size_t>(state.range(0));
    auto precomputed_vks = precompute_verification_keys(ivc, num_circuits);

    // Perform a specified number of iterations of function/kernel accumulation
    perform_ivc_accumulation_rounds(num_circuits, ivc, precomputed_vks);

    BB_REPORT_OP_COUNT_IN_BENCH(state);
    // Perform a specified number of iterations of function/kernel accumulation
    perform_ivc_accumulation_rounds(num_circuits, ivc, precomputed_vks);

    // Construct eccvm proof, measure only translator proof construction
    ivc.goblin.prove_eccvm();
    for (auto _ : state) {
        ivc.goblin.prove_translator();
    }
}

#define ARGS                                                                                                           \
    Arg(ClientIVCBench::NUM_ITERATIONS_MEDIUM_COMPLEXITY)                                                              \
        ->Arg(1 << 1)                                                                                                  \
        ->Arg(1 << 2)                                                                                                  \
        ->Arg(1 << 3)                                                                                                  \
        ->Arg(1 << 4)                                                                                                  \
        ->Arg(1 << 5)                                                                                                  \
        ->Arg(1 << 6)

BENCHMARK_REGISTER_F(ClientIVCBench, Full)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, FullStructured)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, Accumulate)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, Decide)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, ECCVM)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, Translator)->Unit(benchmark::kMillisecond)->ARGS;

} // namespace

BENCHMARK_MAIN();
