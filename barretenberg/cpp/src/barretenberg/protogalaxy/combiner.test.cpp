#include "barretenberg/honk/utils/testing.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include <gtest/gtest.h>

using namespace bb;

using Flavor = UltraFlavor;
using Polynomial = typename Flavor::Polynomial;
using FF = typename Flavor::FF;

// TODO(https://github.com/AztecProtocol/barretenberg/issues/780): Improve combiner tests to check more than the
// arithmetic relation so we more than unit test folding relation parameters and alpha as well.
TEST(Protogalaxy, CombinerOn2Instances)
{
    constexpr size_t NUM_INSTANCES = 2;
    using ProverInstance = ProverInstance_<Flavor>;
    using ProverInstances = ProverInstances_<Flavor, NUM_INSTANCES>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<ProverInstances>;

    const auto restrict_to_standard_arithmetic_relation = [](auto& polys) {
        std::fill(polys.q_arith.begin(), polys.q_arith.end(), 1);
        std::fill(polys.q_delta_range.begin(), polys.q_delta_range.end(), 0);
        std::fill(polys.q_elliptic.begin(), polys.q_elliptic.end(), 0);
        std::fill(polys.q_aux.begin(), polys.q_aux.end(), 0);
        std::fill(polys.q_lookup.begin(), polys.q_lookup.end(), 0);
        std::fill(polys.q_4.begin(), polys.q_4.end(), 0);
        std::fill(polys.w_4.begin(), polys.w_4.end(), 0);
        std::fill(polys.w_4_shift.begin(), polys.w_4_shift.end(), 0);
    };

    auto run_test = [&](bool is_random_input) {
        // Combiner test on prover polynomisls containing random values, restricted to only the standard arithmetic
        // relation.
        if (is_random_input) {
            std::vector<std::shared_ptr<ProverInstance>> instance_data(NUM_INSTANCES);
            ProtoGalaxyProver prover;

            for (size_t idx = 0; idx < NUM_INSTANCES; idx++) {
                auto instance = std::make_shared<ProverInstance>();
                auto prover_polynomials = get_sequential_prover_polynomials<Flavor>(
                    /*log_circuit_size=*/1, idx * 128);
                restrict_to_standard_arithmetic_relation(prover_polynomials);
                instance->proving_key.polynomials = std::move(prover_polynomials);
                instance->proving_key.circuit_size = 2;
                instance_data[idx] = instance;
            }

            ProverInstances instances{ instance_data };
            instances.alphas.fill(bb::Univariate<FF, 12>(FF(0))); // focus on the arithmetic relation only
            auto pow_polynomial = PowPolynomial(std::vector<FF>{ 2 });
            auto result = prover.compute_combiner</*OptimisationEnabled=*/false>(instances, pow_polynomial);
            // The expected_result values are computed by running the python script combiner_example_gen.py
            auto expected_result = Univariate<FF, 12>(std::array<FF, 12>{ 8600UL,
                                                                          12679448UL,
                                                                          73617560UL,
                                                                          220571672UL,
                                                                          491290520UL,
                                                                          923522840UL,
                                                                          1555017368UL,
                                                                          2423522840UL,
                                                                          3566787992UL,
                                                                          5022561560UL,
                                                                          6828592280UL,
                                                                          9022628888UL });
            EXPECT_EQ(result, expected_result);
        } else {
            std::vector<std::shared_ptr<ProverInstance>> instance_data(NUM_INSTANCES);
            ProtoGalaxyProver prover;

            for (size_t idx = 0; idx < NUM_INSTANCES; idx++) {
                auto instance = std::make_shared<ProverInstance>();
                auto prover_polynomials = get_zero_prover_polynomials<Flavor>(
                    /*log_circuit_size=*/1);
                restrict_to_standard_arithmetic_relation(prover_polynomials);
                instance->proving_key.polynomials = std::move(prover_polynomials);
                instance->proving_key.circuit_size = 2;
                instance_data[idx] = instance;
            }

            ProverInstances instances{ instance_data };
            instances.alphas.fill(bb::Univariate<FF, 12>(FF(0))); // focus on the arithmetic relation only

            const auto create_add_gate = [](auto& polys, const size_t idx, FF w_l, FF w_r) {
                polys.w_l[idx] = w_l;
                polys.w_r[idx] = w_r;
                polys.w_o[idx] = w_l + w_r;
                polys.q_l[idx] = 1;
                polys.q_r[idx] = 1;
                polys.q_o[idx] = -1;
            };

            const auto create_mul_gate = [](auto& polys, const size_t idx, FF w_l, FF w_r) {
                polys.w_l[idx] = w_l;
                polys.w_r[idx] = w_r;
                polys.w_o[idx] = w_l * w_r;
                polys.q_m[idx] = 1;
                polys.q_o[idx] = -1;
            };

            create_add_gate(instances[0]->proving_key.polynomials, 0, 1, 2);
            create_add_gate(instances[0]->proving_key.polynomials, 1, 0, 4);
            create_add_gate(instances[1]->proving_key.polynomials, 0, 3, 4);
            create_mul_gate(instances[1]->proving_key.polynomials, 1, 1, 4);

            restrict_to_standard_arithmetic_relation(instances[0]->proving_key.polynomials);
            restrict_to_standard_arithmetic_relation(instances[1]->proving_key.polynomials);

            /* Instance 0                                    Instance 1
                w_l w_r w_o q_m q_l q_r q_o q_c               w_l w_r w_o q_m q_l q_r q_o q_c
                1   2   3   0   1   1   -1  0                 3   4   7   0   1   1   -1  0
                0   4   4   0   1   1   -1  0                 1   4   4   1   0   0   -1  0             */

            /* Lagrange-combined values, row index 0         Lagrange-combined values, row index 1
                in    0    1    2    3    4    5    6        in    0    1    2    3    4    5    6
                w_l   1    3    5    7    9   11   13        w_l   0    1    2    3    4    5    6
                w_r   2    4    6    8   10   12   14        w_r   4    4    4    4    4    4    4
                w_o   3    7   11   15   19   23   27        w_o   4    4    4    4    4    4    0
                q_m   0    0    0    0    0    0    0        q_m   0    1    2    3    4    5    6
                q_l   1    1    1    1    1    1    1        q_l   1    0   -1   -2   -3   -4   -5
                q_r   1    1    1    1    1    1    1        q_r   1    0   -1   -2   -3   -4   -5
                q_o  -1   -1   -1   -1   -1   -1   -1        q_o  -1   -1   -1   -1   -1   -1   -1
                q_c   0    0    0    0    0    0    0        q_c   0    0    0    0    0    0    0

            relation value:
                      0    0    0    0    0    0    0              0    0    6   18   36   60   90      */

            auto pow_polynomial = PowPolynomial(std::vector<FF>{ 2 });
            auto result = prover.compute_combiner</*OptimisationEnabled=*/false>(instances, pow_polynomial);
            auto optimised_result = prover.compute_combiner(instances, pow_polynomial);
            auto expected_result =
                Univariate<FF, 12>(std::array<FF, 12>{ 0, 0, 12, 36, 72, 120, 180, 252, 336, 432, 540, 660 });

            EXPECT_EQ(result, expected_result);
            EXPECT_EQ(optimised_result, expected_result);
        }
    };
    run_test(true);
    run_test(false);
};

// Check that the optimized combiner computation yields a result consistent with the unoptimized version
TEST(Protogalaxy, CombinerOptimizationConsistency)
{
    constexpr size_t NUM_INSTANCES = 2;
    using ProverInstance = ProverInstance_<Flavor>;
    using ProverInstances = ProverInstances_<Flavor, NUM_INSTANCES>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<ProverInstances>;
    using UltraArithmeticRelation = UltraArithmeticRelation<FF>;

    constexpr size_t UNIVARIATE_LENGTH = 12;
    const auto restrict_to_standard_arithmetic_relation = [](auto& polys) {
        std::fill(polys.q_arith.begin(), polys.q_arith.end(), 1);
        std::fill(polys.q_delta_range.begin(), polys.q_delta_range.end(), 0);
        std::fill(polys.q_elliptic.begin(), polys.q_elliptic.end(), 0);
        std::fill(polys.q_aux.begin(), polys.q_aux.end(), 0);
        std::fill(polys.q_lookup.begin(), polys.q_lookup.end(), 0);
        std::fill(polys.q_4.begin(), polys.q_4.end(), 0);
        std::fill(polys.w_4.begin(), polys.w_4.end(), 0);
        std::fill(polys.w_4_shift.begin(), polys.w_4_shift.end(), 0);
    };

    auto run_test = [&](bool is_random_input) {
        // Combiner test on prover polynomisls containing random values, restricted to only the standard arithmetic
        // relation.
        if (is_random_input) {
            std::vector<std::shared_ptr<ProverInstance>> instance_data(NUM_INSTANCES);
            ASSERT(NUM_INSTANCES == 2); // Don't want to handle more here
            ProtoGalaxyProver prover;

            for (size_t idx = 0; idx < NUM_INSTANCES; idx++) {
                auto instance = std::make_shared<ProverInstance>();
                auto prover_polynomials = get_sequential_prover_polynomials<Flavor>(
                    /*log_circuit_size=*/1, idx * 128);
                restrict_to_standard_arithmetic_relation(prover_polynomials);
                instance->proving_key.polynomials = std::move(prover_polynomials);
                instance->proving_key.circuit_size = 2;
                instance_data[idx] = instance;
            }

            ProverInstances instances{ instance_data };
            instances.alphas.fill(
                bb::Univariate<FF, UNIVARIATE_LENGTH>(FF(0))); // focus on the arithmetic relation only
            auto pow_polynomial = PowPolynomial(std::vector<FF>{ 2 });
            pow_polynomial.compute_values();

            // Relation parameters are all zeroes
            RelationParameters<FF> relation_parameters;
            // Temporary accumulator to compute the sumcheck on the second instance
            typename Flavor::TupleOfArraysOfValues temporary_accumulator;

            // Accumulate arithmetic relation over 2 rows on the second instance
            for (size_t i = 0; i < 2; i++) {
                UltraArithmeticRelation::accumulate(
                    std::get<0>(temporary_accumulator),
                    instance_data[NUM_INSTANCES - 1]->proving_key.polynomials.get_row(i),
                    relation_parameters,
                    pow_polynomial[i]);
            }
            // Get the result of the 0th subrelation of the arithmetic relation
            FF instance_offset = std::get<0>(temporary_accumulator)[0];
            // Subtract it from q_c[0] (it directly affect the target sum, making it zero and enabling the optimisation)
            instance_data[1]->proving_key.polynomials.q_c[0] -= instance_offset;
            std::vector<typename Flavor::ProverPolynomials>
                extended_polynomials; // These hold the extensions of prover polynomials

            // Manually extend all polynomials. Create new ProverPolynomials from extended values
            for (size_t idx = NUM_INSTANCES; idx < UNIVARIATE_LENGTH; idx++) {

                auto instance = std::make_shared<ProverInstance>();
                auto prover_polynomials = get_zero_prover_polynomials<Flavor>(1);
                for (auto [instance_0_polynomial, instance_1_polynomial, new_polynomial] :
                     zip_view(instance_data[0]->proving_key.polynomials.get_all(),
                              instance_data[1]->proving_key.polynomials.get_all(),
                              prover_polynomials.get_all())) {
                    for (size_t i = 0; i < /*circuit_size*/ 2; i++) {
                        new_polynomial[i] =
                            instance_0_polynomial[i] + ((instance_1_polynomial[i] - instance_0_polynomial[i]) * idx);
                    }
                }
                extended_polynomials.push_back(std::move(prover_polynomials));
            }
            std::array<FF, UNIVARIATE_LENGTH> precomputed_result{ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
            // Compute the sum for each index separately, treating each extended instance independently
            for (size_t idx = 0; idx < UNIVARIATE_LENGTH; idx++) {

                typename Flavor::TupleOfArraysOfValues accumulator;
                if (idx < NUM_INSTANCES) {
                    for (size_t i = 0; i < 2; i++) {
                        UltraArithmeticRelation::accumulate(std::get<0>(accumulator),
                                                            instance_data[idx]->proving_key.polynomials.get_row(i),
                                                            relation_parameters,
                                                            pow_polynomial[i]);
                    }
                } else {
                    for (size_t i = 0; i < 2; i++) {
                        UltraArithmeticRelation::accumulate(std::get<0>(accumulator),
                                                            extended_polynomials[idx - NUM_INSTANCES].get_row(i),
                                                            relation_parameters,
                                                            pow_polynomial[i]);
                    }
                }
                precomputed_result[idx] = std::get<0>(accumulator)[0];
            }
            auto expected_result = Univariate<FF, UNIVARIATE_LENGTH>(precomputed_result);
            auto result = prover.compute_combiner</*OptimisationEnabled=*/false>(instances, pow_polynomial);
            auto optimised_result = prover.compute_combiner(instances, pow_polynomial);

            EXPECT_EQ(result, expected_result);
            EXPECT_EQ(optimised_result, expected_result);
        } else {
            std::vector<std::shared_ptr<ProverInstance>> instance_data(NUM_INSTANCES);
            ProtoGalaxyProver prover;

            for (size_t idx = 0; idx < NUM_INSTANCES; idx++) {
                auto instance = std::make_shared<ProverInstance>();
                auto prover_polynomials = get_zero_prover_polynomials<Flavor>(
                    /*log_circuit_size=*/1);
                restrict_to_standard_arithmetic_relation(prover_polynomials);
                instance->proving_key.polynomials = std::move(prover_polynomials);
                instance->proving_key.circuit_size = 2;
                instance_data[idx] = instance;
            }

            ProverInstances instances{ instance_data };
            instances.alphas.fill(bb::Univariate<FF, 12>(FF(0))); // focus on the arithmetic relation only

            const auto create_add_gate = [](auto& polys, const size_t idx, FF w_l, FF w_r) {
                polys.w_l[idx] = w_l;
                polys.w_r[idx] = w_r;
                polys.w_o[idx] = w_l + w_r;
                polys.q_l[idx] = 1;
                polys.q_r[idx] = 1;
                polys.q_o[idx] = -1;
            };

            const auto create_mul_gate = [](auto& polys, const size_t idx, FF w_l, FF w_r) {
                polys.w_l[idx] = w_l;
                polys.w_r[idx] = w_r;
                polys.w_o[idx] = w_l * w_r;
                polys.q_m[idx] = 1;
                polys.q_o[idx] = -1;
            };

            create_add_gate(instances[0]->proving_key.polynomials, 0, 1, 2);
            create_add_gate(instances[0]->proving_key.polynomials, 1, 0, 4);
            create_add_gate(instances[1]->proving_key.polynomials, 0, 3, 4);
            create_mul_gate(instances[1]->proving_key.polynomials, 1, 1, 4);

            restrict_to_standard_arithmetic_relation(instances[0]->proving_key.polynomials);
            restrict_to_standard_arithmetic_relation(instances[1]->proving_key.polynomials);

            /* Instance 0                                    Instance 1
                w_l w_r w_o q_m q_l q_r q_o q_c               w_l w_r w_o q_m q_l q_r q_o q_c
                1   2   3   0   1   1   -1  0                 3   4   7   0   1   1   -1  0
                0   4   4   0   1   1   -1  0                 1   4   4   1   0   0   -1  0             */

            /* Lagrange-combined values, row index 0         Lagrange-combined values, row index 1
                in    0    1    2    3    4    5    6        in    0    1    2    3    4    5    6
                w_l   1    3    5    7    9   11   13        w_l   0    1    2    3    4    5    6
                w_r   2    4    6    8   10   12   14        w_r   4    4    4    4    4    4    4
                w_o   3    7   11   15   19   23   27        w_o   4    4    4    4    4    4    0
                q_m   0    0    0    0    0    0    0        q_m   0    1    2    3    4    5    6
                q_l   1    1    1    1    1    1    1        q_l   1    0   -1   -2   -3   -4   -5
                q_r   1    1    1    1    1    1    1        q_r   1    0   -1   -2   -3   -4   -5
                q_o  -1   -1   -1   -1   -1   -1   -1        q_o  -1   -1   -1   -1   -1   -1   -1
                q_c   0    0    0    0    0    0    0        q_c   0    0    0    0    0    0    0

            relation value:
                      0    0    0    0    0    0    0              0    0    6   18   36   60   90      */

            auto pow_polynomial = PowPolynomial(std::vector<FF>{ 2 });
            auto result = prover.compute_combiner</*OptimisationEnabled=*/false>(instances, pow_polynomial);
            auto optimised_result = prover.compute_combiner(instances, pow_polynomial);
            auto expected_result =
                Univariate<FF, 12>(std::array<FF, 12>{ 0, 0, 12, 36, 72, 120, 180, 252, 336, 432, 540, 660 });

            EXPECT_EQ(result, expected_result);
            EXPECT_EQ(optimised_result, expected_result);
        }
    };
    run_test(true);
    run_test(false);
};

TEST(Protogalaxy, CombinerOn4Instances)
{
    constexpr size_t NUM_INSTANCES = 4;
    using ProverInstance = ProverInstance_<Flavor>;
    using ProverInstances = ProverInstances_<Flavor, NUM_INSTANCES>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<ProverInstances>;

    const auto zero_all_selectors = [](auto& polys) {
        std::fill(polys.q_arith.begin(), polys.q_arith.end(), 0);
        std::fill(polys.q_delta_range.begin(), polys.q_delta_range.end(), 0);
        std::fill(polys.q_elliptic.begin(), polys.q_elliptic.end(), 0);
        std::fill(polys.q_aux.begin(), polys.q_aux.end(), 0);
        std::fill(polys.q_lookup.begin(), polys.q_lookup.end(), 0);
        std::fill(polys.q_4.begin(), polys.q_4.end(), 0);
        std::fill(polys.w_4.begin(), polys.w_4.end(), 0);
        std::fill(polys.w_4_shift.begin(), polys.w_4_shift.end(), 0);
    };

    auto run_test = [&]() {
        std::vector<std::shared_ptr<ProverInstance>> instance_data(NUM_INSTANCES);
        ProtoGalaxyProver prover;

        for (size_t idx = 0; idx < NUM_INSTANCES; idx++) {
            auto instance = std::make_shared<ProverInstance>();
            auto prover_polynomials = get_zero_prover_polynomials<Flavor>(
                /*log_circuit_size=*/1);
            instance->proving_key.polynomials = std::move(prover_polynomials);
            instance->proving_key.circuit_size = 2;
            instance_data[idx] = instance;
        }

        ProverInstances instances{ instance_data };
        instances.alphas.fill(bb::Univariate<FF, 40>(FF(0))); // focus on the arithmetic relation only

        zero_all_selectors(instances[0]->proving_key.polynomials);
        zero_all_selectors(instances[1]->proving_key.polynomials);
        zero_all_selectors(instances[2]->proving_key.polynomials);
        zero_all_selectors(instances[3]->proving_key.polynomials);

        auto pow_polynomial = PowPolynomial(std::vector<FF>{ 2 });
        auto result = prover.compute_combiner</*OptimisationEnabled=*/false>(instances, pow_polynomial);
        auto optimised_result = prover.compute_combiner(instances, pow_polynomial);
        std::array<FF, 40> zeroes;
        std::fill(zeroes.begin(), zeroes.end(), 0);
        auto expected_result = Univariate<FF, 40>(zeroes);
        EXPECT_EQ(result, expected_result);
        EXPECT_EQ(optimised_result, expected_result);
    };
    run_test();
};
