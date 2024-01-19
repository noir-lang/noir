
#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/ultra_bench/benchmark_utilities.hpp"
#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

using namespace benchmark;
using namespace bb;
using namespace bb;

namespace {
void goblin_full(State& state) noexcept
{
    bb::srs::init_crs_factory("../srs_db/ignition");
    bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");

    Goblin goblin;

    // Construct an initial circuit; its proof will be recursively verified by the first kernel
    GoblinUltraCircuitBuilder initial_circuit{ goblin.op_queue };
    GoblinMockCircuits::construct_simple_initial_circuit(initial_circuit);
    Goblin::AccumulationOutput kernel_input = goblin.accumulate(initial_circuit);

    Goblin::Proof proof;
    for (auto _ : state) {
        // Construct a series of simple Goblin circuits; generate and verify their proofs
        size_t NUM_CIRCUITS = 1 << static_cast<size_t>(state.range(0));
        for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
            // Construct a circuit with logic resembling that of the "kernel circuit"
            GoblinUltraCircuitBuilder circuit_builder{ goblin.op_queue };
            GoblinMockCircuits::construct_mock_kernel_circuit(circuit_builder, kernel_input);

            // Construct proof of the current kernel circuit to be recursively verified by the next one
            kernel_input = goblin.accumulate(circuit_builder);
        }

        proof = goblin.prove();
        // Verify the final ultra proof
    }
    honk::GoblinUltraVerifier ultra_verifier{ kernel_input.verification_key };
    ultra_verifier.verify_proof(kernel_input.proof);
    // Verify the goblin proof (eccvm, translator, merge)
    goblin.verify(proof);
}

void goblin_accumulate(State& state) noexcept
{
    bb::srs::init_crs_factory("../srs_db/ignition");
    bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");

    Goblin goblin;

    // Construct an initial circuit; its proof will be recursively verified by the first kernel
    GoblinUltraCircuitBuilder initial_circuit{ goblin.op_queue };
    GoblinMockCircuits::construct_simple_initial_circuit(initial_circuit);
    Goblin::AccumulationOutput kernel_input = goblin.accumulate(initial_circuit);

    // Construct a series of simple Goblin circuits; generate and verify their proofs
    size_t NUM_CIRCUITS = 1 << static_cast<size_t>(state.range(0));
    for (auto _ : state) {
        for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
            // Construct a circuit with logic resembling that of the "kernel circuit"
            GoblinUltraCircuitBuilder circuit_builder{ goblin.op_queue };
            GoblinMockCircuits::construct_mock_kernel_circuit(circuit_builder, kernel_input);

            // Construct proof of the current kernel circuit to be recursively verified by the next one
            kernel_input = goblin.accumulate(circuit_builder);
        }
    }
}

void goblin_eccvm_prove(State& state) noexcept
{
    bb::srs::init_crs_factory("../srs_db/ignition");
    bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");

    Goblin goblin;

    // Construct an initial circuit; its proof will be recursively verified by the first kernel
    GoblinUltraCircuitBuilder initial_circuit{ goblin.op_queue };
    GoblinMockCircuits::construct_simple_initial_circuit(initial_circuit);
    Goblin::AccumulationOutput kernel_input = goblin.accumulate(initial_circuit);

    // Construct a series of simple Goblin circuits; generate and verify their proofs
    size_t NUM_CIRCUITS = 1 << static_cast<size_t>(state.range(0));
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        // Construct a circuit with logic resembling that of the "kernel circuit"
        GoblinUltraCircuitBuilder circuit_builder{ goblin.op_queue };
        GoblinMockCircuits::construct_mock_kernel_circuit(circuit_builder, kernel_input);

        // Construct proof of the current kernel circuit to be recursively verified by the next one
        kernel_input = goblin.accumulate(circuit_builder);
    }

    for (auto _ : state) {
        goblin.prove_eccvm();
    }
}

void goblin_translator_prove(State& state) noexcept
{
    bb::srs::init_crs_factory("../srs_db/ignition");
    bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");

    Goblin goblin;

    // Construct an initial circuit; its proof will be recursively verified by the first kernel
    GoblinUltraCircuitBuilder initial_circuit{ goblin.op_queue };
    GoblinMockCircuits::construct_simple_initial_circuit(initial_circuit);
    Goblin::AccumulationOutput kernel_input = goblin.accumulate(initial_circuit);

    // Construct a series of simple Goblin circuits; generate and verify their proofs
    size_t NUM_CIRCUITS = 1 << static_cast<size_t>(state.range(0));
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        // Construct a circuit with logic resembling that of the "kernel circuit"
        GoblinUltraCircuitBuilder circuit_builder{ goblin.op_queue };
        GoblinMockCircuits::construct_mock_kernel_circuit(circuit_builder, kernel_input);

        // Construct proof of the current kernel circuit to be recursively verified by the next one
        kernel_input = goblin.accumulate(circuit_builder);
    }

    goblin.prove_eccvm();
    for (auto _ : state) {
        goblin.prove_translator();
    }
}

} // namespace

BENCHMARK(goblin_full)->Unit(kMillisecond)->DenseRange(0, 7);
BENCHMARK(goblin_accumulate)->Unit(kMillisecond)->DenseRange(0, 7);
BENCHMARK(goblin_eccvm_prove)->Unit(kMillisecond)->DenseRange(0, 7);
BENCHMARK(goblin_translator_prove)->Unit(kMillisecond)->DenseRange(0, 7);