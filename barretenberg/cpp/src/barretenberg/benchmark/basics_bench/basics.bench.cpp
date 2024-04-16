/**
 * @file parallel.bench.cpp
 * @author Rumata888
 * @brief Simple and not too strict benchmarks for most basic operations used in barretenberg
 * @details Filtered benchmark results (nanoseconds):
 *      cycle_waste:                            0.5
 *      ff_addition:                            3.8
 *      ff_from_montgomery:                     19.1
 *      ff_invert:                              7001.3
 *      ff_multiplication:                      21.3
 *      ff_reduce:                              5.1
 *      ff_sqr:                                 17.9
 *      ff_to_montgomery:                       39.1
 *      parallel_for_field_element_addition:    198000~388000 (The number is somewhat dependent on the number of cores
 * used)
 *      projective_point_accidental_doubling:   347.6
 *      projective_point_addition:              348.6
 *      projective_point_doubling:              194.2
 *      scalar_multiplication:                  50060.1
 *      sequential_copy:                        3.3
 *
 */
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/common/thread.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;
using namespace bb;
namespace {
using Curve = curve::BN254;
using Fr = Curve::ScalarField;
#define MAX_REPETITION_LOG 12

/**
 * @brief Benchmark for evaluating the cost of starting parallel_for
 *
 * @details It seems parallel_for takes ~400 microseconds to start when we use all the cores. When it's just 1 it's 200
 * microseconds. The dependency is not exactly linear, so in code we use the largest value for convenience
 * @param state
 */
void parallel_for_field_element_addition(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    size_t num_cpus = get_num_cpus();
    std::vector<std::vector<Fr>> copy_vector(num_cpus);
    for (size_t i = 0; i < num_cpus; i++) {
        for (size_t j = 0; j < 2; j++) {
            copy_vector[i].emplace_back(Fr::random_element(&engine));
            copy_vector[i].emplace_back(Fr::random_element(&engine));
        }
    }
    for (auto _ : state) {
        state.PauseTiming();
        size_t num_external_cycles = 1 << static_cast<size_t>(state.range(0));
        size_t num_internal_cycles = 1 << (MAX_REPETITION_LOG - static_cast<size_t>(state.range(0)));
        state.ResumeTiming();
        for (size_t i = 0; i < num_external_cycles; i++) {
            parallel_for(num_cpus, [num_internal_cycles, &copy_vector](size_t index) {
                for (size_t i = 0; i < num_internal_cycles; i++) {
                    copy_vector[index][i & 1] += copy_vector[index][1 - (i & 1)];
                }
            });
        }
    }
}

/**
 * @brief Evaluate how much finite addition costs (in cache)
 *
 *@details ~4 ns if we subtract  i++ operation
 * @param state
 */
void ff_addition(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    std::vector<Fr> copy_vector(2);
    for (size_t j = 0; j < 2; j++) {
        copy_vector.emplace_back(Fr::random_element(&engine));
        copy_vector.emplace_back(Fr::random_element(&engine));
    }

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            copy_vector[i & 1] += copy_vector[1 - (i & 1)];
        }
    }
}

/**
 * @brief Evaluate how much finite field multiplication costs (in cache)
 *
 *@details ~21 ns if we subtract i++ operation
 * @param state
 */
void ff_multiplication(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    std::vector<Fr> copy_vector(2);
    for (size_t j = 0; j < 2; j++) {
        copy_vector.emplace_back(Fr::random_element(&engine));
        copy_vector.emplace_back(Fr::random_element(&engine));
    }

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            copy_vector[i & 1] *= copy_vector[1 - (i & 1)];
        }
    }
}

/**
 * @brief Evaluate how much finite field squaring costs (in cache)
 *
 *@details ~18 ns if we subtract i++ operation
 * @param state
 */
void ff_sqr(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    std::vector<Fr> copy_vector(2);
    for (size_t j = 0; j < 2; j++) {
        copy_vector.emplace_back(Fr::random_element(&engine));
        copy_vector.emplace_back(Fr::random_element(&engine));
    }

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            copy_vector[0] = copy_vector[0].sqr();
        }
    }
}

/**
 * @brief Evaluate how much finite field inversion costs (in cache)
 *
 *@details ~7100 ns if we subtract addition and i++ operation
 * @param state
 */
void ff_invert(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    auto element = Fr::random_element(&engine);

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            element = (element + Fr::one()).invert();
        }
    }
}

/**
 * @brief Evaluate how much conversion to montgomery costs (in cache)
 *
 *@details ~39 ns if we subtract i++ operation
 * @param state
 */
void ff_to_montgomery(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    auto element = Fr::random_element(&engine);

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            element = element.to_montgomery_form();
        }
    }
}
/**
 * @brief Evaluate how much conversion from montgomery costs (in cache)
 *
 *@details ~19 ns if we subtract i++ operation
 * @param state
 */
void ff_from_montgomery(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    auto element = Fr::random_element(&engine);

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            element = element.from_montgomery_form();
        }
    }
}

/**
 * @brief Evaluate how much reduction costs (in cache)
 *
 *@details ~5 ns if we subtract addition and i++ operation
 * @param state
 */
void ff_reduce(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    auto element = Fr::random_element(&engine);

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            element = (element + element).reduce_once();
        }
    }
}

/**
 * @brief Evaluate how much projective point addition costs (in cache)
 *
 *@details ~350 ns if we subtract  i++ operation
 * @param state
 */
void projective_point_addition(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    std::vector<Curve::Element> copy_vector(2);
    for (size_t j = 0; j < 2; j++) {
        copy_vector.emplace_back(Curve::Element::random_element(&engine));
        copy_vector.emplace_back(Curve::Element::random_element(&engine));
    }

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            copy_vector[i & 1] += copy_vector[1 - (i & 1)];
        }
    }
}

/**
 * @brief Evaluate how much projective point doubling costs when we trigger it through addition (in cache)
 *
 *@details ~354 ns if we subtract  i++ operation
 * @param state
 */
void projective_point_accidental_doubling(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    std::vector<Curve::Element> copy_vector(2);
    for (size_t j = 0; j < 2; j++) {
        copy_vector.emplace_back(Curve::Element::random_element(&engine));
        copy_vector.emplace_back(Curve::Element::random_element(&engine));
    }

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            copy_vector[0] += copy_vector[0];
        }
    }
}

/**
 * @brief Evaluate how much projective point doubling costs (in cache)
 *
 *@details ~195 ns if we subtract  i++ operation
 * @param state
 */
void projective_point_doubling(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    std::vector<Curve::Element> copy_vector(2);
    for (size_t j = 0; j < 2; j++) {
        copy_vector.emplace_back(Curve::Element::random_element(&engine));
        copy_vector.emplace_back(Curve::Element::random_element(&engine));
    }

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            copy_vector[0] = copy_vector[0].dbl();
        }
    }
}

/**
 * @brief Evaluate how much scalar multiplication costs (in cache)
 *
 *@details ~50000 ns
 * @param state
 */
void scalar_multiplication(State& state)
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    Curve::Element element = Curve::Element::random_element(&engine);
    Fr scalar = Fr::random_element(&engine);

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            element = element * scalar;
            scalar += scalar;
        }
    }
}
/**
 * @brief Evaluate how much running the loop costs in benchmarks
 *
 * @details 0.6~0.7 ns per cycle
 * @param state
 */
void cycle_waste(State& state)
{

    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        state.ResumeTiming();
        for (volatile size_t i = 0; i < num_cycles;) {
            i = i + 1;
        }
    }
}

/**
 * @brief Evaluate how much copying memory for large vectors costs
 *
 * @details 5 ns per cycle
 * @param state
 */
void sequential_copy(State& state)
{

    numeric::RNG& engine = numeric::get_debug_randomness();
    for (auto _ : state) {
        state.PauseTiming();
        size_t num_cycles = 1 << static_cast<size_t>(state.range(0));
        std::vector<Fr> input(num_cycles);
        for (size_t i = 0; i < num_cycles; i++) {
            *(uint256_t*)&input[i] = engine.get_random_uint256();
        }
        std::vector<Fr> output(num_cycles);

        state.ResumeTiming();
        for (size_t i = 0; i < num_cycles; i++) {
            output[i] = input[i];
        }
    }
}
} // namespace

BENCHMARK(parallel_for_field_element_addition)->Unit(kMicrosecond)->DenseRange(0, MAX_REPETITION_LOG);
BENCHMARK(ff_addition)->Unit(kMicrosecond)->DenseRange(12, 30);
BENCHMARK(ff_multiplication)->Unit(kMicrosecond)->DenseRange(12, 27);
BENCHMARK(ff_sqr)->Unit(kMicrosecond)->DenseRange(12, 27);
BENCHMARK(ff_invert)->Unit(kMicrosecond)->DenseRange(12, 19);
BENCHMARK(ff_to_montgomery)->Unit(kMicrosecond)->DenseRange(12, 27);
BENCHMARK(ff_from_montgomery)->Unit(kMicrosecond)->DenseRange(12, 27);
BENCHMARK(ff_reduce)->Unit(kMicrosecond)->DenseRange(12, 29);
BENCHMARK(projective_point_addition)->Unit(kMicrosecond)->DenseRange(12, 22);
BENCHMARK(projective_point_accidental_doubling)->Unit(kMicrosecond)->DenseRange(12, 22);
BENCHMARK(projective_point_doubling)->Unit(kMicrosecond)->DenseRange(12, 22);
BENCHMARK(scalar_multiplication)->Unit(kMicrosecond)->DenseRange(12, 18);
BENCHMARK(cycle_waste)->Unit(kMicrosecond)->DenseRange(20, 30);
BENCHMARK(sequential_copy)->Unit(kMicrosecond)->DenseRange(20, 25);
BENCHMARK_MAIN();