#pragma once
#include <benchmark/benchmark.h>

#include "barretenberg/crypto/merkle_tree/membership.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/hash/keccak/keccak.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"

namespace bb::mock_circuits {

/**
 * @brief Generate test circuit with basic arithmetic operations
 *
 * @param composer
 * @param num_iterations
 */
template <typename Builder> void generate_basic_arithmetic_circuit(Builder& builder, size_t log2_num_gates)
{
    stdlib::field_t a(stdlib::witness_t(&builder, fr::random_element()));
    stdlib::field_t b(stdlib::witness_t(&builder, fr::random_element()));
    stdlib::field_t c(&builder);
    size_t passes = (1UL << log2_num_gates) / 4 - 4;
    if (static_cast<int>(passes) <= 0) {
        throw_or_abort("too few gates");
    }

    for (size_t i = 0; i < passes; ++i) {
        c = a + b;
        c = a * c;
        a = b * b;
        b = c * c;
    }
}

template <typename Prover>
Prover get_prover(void (*test_circuit_function)(typename Prover::Flavor::CircuitBuilder&, size_t),
                  size_t num_iterations)
{
    using Flavor = typename Prover::Flavor;
    using Builder = typename Flavor::CircuitBuilder;

    Builder builder;
    test_circuit_function(builder, num_iterations);
    // This is gross but it's going away soon.
    if constexpr (IsPlonkFlavor<Flavor>) {
        // If Flavor is Ultra, alias UltraComposer, otherwise alias StandardComposer
        using Composer = std::
            conditional_t<std::same_as<Flavor, plonk::flavor::Ultra>, plonk::UltraComposer, plonk::StandardComposer>;
        Composer composer;
        return composer.create_prover(builder);
    } else {
        return Prover(builder);
    }
};

/**
 * @brief Performs proof constuction for benchmarks based on a provided circuit function
 *
 * @details This function assumes state.range refers to num_iterations which is the number of times to perform a given
 * basic operation in the circuit, e.g. number of hashes
 *
 * @tparam Builder
 * @param state
 * @param test_circuit_function
 */
template <typename Prover>
void construct_proof_with_specified_num_iterations(
    benchmark::State& state,
    void (*test_circuit_function)(typename Prover::Flavor::CircuitBuilder&, size_t),
    size_t num_iterations)
{
    srs::init_crs_factory("../srs_db/ignition");

    for (auto _ : state) {
        // Construct circuit and prover; don't include this part in measurement
        state.PauseTiming();
        Prover prover = get_prover<Prover>(test_circuit_function, num_iterations);
        state.ResumeTiming();

        // Construct proof
        auto proof = prover.construct_proof();
    }
}

} // namespace bb::mock_circuits
