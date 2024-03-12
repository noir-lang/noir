
#include <benchmark/benchmark.h>

#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/common/op_count_google_bench.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
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
    using Builder = GoblinUltraCircuitBuilder;
    using VerifierFoldData = GoblinMockCircuits::VerifierFoldData;

    // Number of function circuits to accumulate(based on Zacs target numbers)
    static constexpr size_t NUM_ITERATIONS_MEDIUM_COMPLEXITY = 6;

    void SetUp([[maybe_unused]] const ::benchmark::State& state) override
    {
        bb::srs::init_crs_factory("../srs_db/ignition");
        bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
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
     */
    static void perform_ivc_accumulation_rounds(State& state, ClientIVC& ivc)
    {
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
        ivc.initialize(initial_function_circuits[0]);
        // Retrieve the queue
        std::swap(*ivc.goblin.op_queue, *initial_function_circuits[0].op_queue);

        // Prepend queue to the second circuit
        initial_function_circuits[1].op_queue->prepend_previous_queue(*ivc.goblin.op_queue);
        // Accumulate another function circuit
        auto function_fold_proof = ivc.accumulate(initial_function_circuits[1]);
        // Retrieve the queue
        std::swap(*ivc.goblin.op_queue, *initial_function_circuits[1].op_queue);
        VerifierFoldData function_fold_output = { function_fold_proof, ivc.vks.func_vk };

        // Free memory
        initial_function_circuits.clear();

        auto NUM_CIRCUITS = static_cast<size_t>(state.range(0));
        // Subtract two to account for the "initialization" round above i.e. we have already folded two function
        // circuits
        NUM_CIRCUITS -= 2;

        // The accumulator for kernel uses the function accumulation verification key
        auto kernel_verifier_accumulator = std::make_shared<ClientIVC::VerifierInstance>(ivc.vks.first_func_vk);

        VerifierFoldData kernel_fold_output;
        for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
            Builder kernel_circuit{ size_hint, ivc.goblin.op_queue };
            Builder function_circuit{ size_hint };
            // Construct function and kernel circuits in parallel
            {
                BB_OP_COUNT_TIME_NAME("construct_circuits");
                parallel_for(2, [&](size_t workload_idx) {
                    // workload index is 0 for kernel and 1 for function
                    if (workload_idx == 0) {
                        if (circuit_idx == 0) {

                            // Create the first folding kernel which only verifies the accumulation of a
                            // function circuit
                            kernel_verifier_accumulator = GoblinMockCircuits::construct_mock_folding_kernel(
                                kernel_circuit, function_fold_output, {}, kernel_verifier_accumulator);
                        } else {
                            // Create kernel circuit containing the recursive folding verification of a function circuit
                            // and a kernel circuit
                            kernel_verifier_accumulator = GoblinMockCircuits::construct_mock_folding_kernel(
                                kernel_circuit, function_fold_output, kernel_fold_output, kernel_verifier_accumulator);
                        }
                    } else {
                        GoblinMockCircuits::construct_mock_function_circuit(function_circuit);
                    }
                });
            };

            // No need to prepend queue, it's the same after last swap
            // Accumulate kernel circuit
            auto kernel_fold_proof = ivc.accumulate(kernel_circuit);

            // First iteration and the following ones differ
            if (circuit_idx == 0) {
                kernel_fold_output = { kernel_fold_proof, ivc.vks.first_kernel_vk };
            } else {
                kernel_fold_output = { kernel_fold_proof, ivc.vks.kernel_vk };
            }

            // Prepend queue to function circuit
            function_circuit.op_queue->prepend_previous_queue(*ivc.goblin.op_queue);

            // Accumulate function circuit
            auto function_fold_proof = ivc.accumulate(function_circuit);
            function_fold_output = { function_fold_proof, ivc.vks.func_vk };

            // Retrieve queue
            std::swap(*ivc.goblin.op_queue, *function_circuit.op_queue);
        }
        // If we haven't entered the cycle, the kernel proof accumulates just function proofs
        if (NUM_CIRCUITS == 0) {
            // Create and accumulate the first folding kernel which only verifies the accumulation of a function circuit
            Builder kernel_circuit{ size_hint, ivc.goblin.op_queue };
            auto kernel_verifier_accumulator = std::make_shared<ClientIVC::VerifierInstance>(ivc.vks.first_func_vk);
            {
                BB_OP_COUNT_TIME_NAME("construct_circuits");
                kernel_verifier_accumulator = GoblinMockCircuits::construct_mock_folding_kernel(
                    kernel_circuit, function_fold_output, {}, kernel_verifier_accumulator);
            }
            auto kernel_fold_proof = ivc.accumulate(kernel_circuit);
            kernel_fold_output = { kernel_fold_proof, ivc.vks.first_kernel_vk };
        } else {
            Builder kernel_circuit{ size_hint, ivc.goblin.op_queue };
            {
                BB_OP_COUNT_TIME_NAME("construct_circuits");
                kernel_verifier_accumulator = GoblinMockCircuits::construct_mock_folding_kernel(
                    kernel_circuit, function_fold_output, kernel_fold_output, kernel_verifier_accumulator);
            }

            auto kernel_fold_proof = ivc.accumulate(kernel_circuit);
            kernel_fold_output = { kernel_fold_proof, ivc.vks.kernel_vk };
        }
    }
};

/**
 * @brief Benchmark the prover work for the full PG-Goblin IVC protocol
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, Full)(benchmark::State& state)
{
    ClientIVC ivc;
    ivc.precompute_folding_verification_keys();
    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        // Perform a specified number of iterations of function/kernel accumulation
        perform_ivc_accumulation_rounds(state, ivc);

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
    ivc.precompute_folding_verification_keys();
    // Perform a specified number of iterations of function/kernel accumulation
    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        perform_ivc_accumulation_rounds(state, ivc);
    }
}

/**
 * @brief Benchmark only the Decider component
 *
 */
BENCHMARK_DEFINE_F(ClientIVCBench, Decide)(benchmark::State& state)
{
    ClientIVC ivc;
    // Perform a specified number of iterations of function/kernel accumulation
    perform_ivc_accumulation_rounds(state, ivc);

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

    // Perform a specified number of iterations of function/kernel accumulation
    perform_ivc_accumulation_rounds(state, ivc);

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
    ivc.precompute_folding_verification_keys();
    BB_REPORT_OP_COUNT_IN_BENCH(state);
    // Perform a specified number of iterations of function/kernel accumulation
    perform_ivc_accumulation_rounds(state, ivc);

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
BENCHMARK_REGISTER_F(ClientIVCBench, Accumulate)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, Decide)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, ECCVM)->Unit(benchmark::kMillisecond)->ARGS;
BENCHMARK_REGISTER_F(ClientIVCBench, Translator)->Unit(benchmark::kMillisecond)->ARGS;

} // namespace

BENCHMARK_MAIN();
