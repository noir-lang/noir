#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/honk/utils/testing.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include <gtest/gtest.h>

using namespace proof_system::honk;
namespace bb::test_protogalaxy_prover {
using Flavor = proof_system::honk::flavor::Ultra;
using Polynomial = typename Flavor::Polynomial;
using FF = typename Flavor::FF;
using RelationParameters = proof_system::RelationParameters<FF>;
using PowPolynomial = bb::PowPolynomial<FF>;

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
        std::fill(polys.q_sort.begin(), polys.q_sort.end(), 0);
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
                auto prover_polynomials = proof_system::honk::get_sequential_prover_polynomials<Flavor>(
                    /*log_circuit_size=*/1, idx * 128);
                restrict_to_standard_arithmetic_relation(prover_polynomials);
                instance->prover_polynomials = std::move(prover_polynomials);
                instance->instance_size = 2;
                instance_data[idx] = instance;
            }

            ProverInstances instances{ instance_data };
            instances.alphas.fill(Univariate<FF, 13>(FF(0))); // focus on the arithmetic relation only
            auto pow_polynomial = PowPolynomial(std::vector<FF>{ 2 });
            auto result = prover.compute_combiner(instances, pow_polynomial);
            auto expected_result = bb::Univariate<FF, 13>(std::array<FF, 13>{ 87706,
                                                                              13644570,
                                                                              76451738,
                                                                              226257946,
                                                                              static_cast<uint64_t>(500811930),
                                                                              static_cast<uint64_t>(937862426),
                                                                              static_cast<uint64_t>(1575158170),
                                                                              static_cast<uint64_t>(2450447898),
                                                                              static_cast<uint64_t>(3601480346),
                                                                              static_cast<uint64_t>(5066004250),
                                                                              static_cast<uint64_t>(6881768346),
                                                                              static_cast<uint64_t>(9086521370),
                                                                              static_cast<uint64_t>(11718012058) });
            EXPECT_EQ(result, expected_result);
        } else {
            std::vector<std::shared_ptr<ProverInstance>> instance_data(NUM_INSTANCES);
            ProtoGalaxyProver prover;

            for (size_t idx = 0; idx < NUM_INSTANCES; idx++) {
                auto instance = std::make_shared<ProverInstance>();
                auto prover_polynomials = proof_system::honk::get_zero_prover_polynomials<Flavor>(
                    /*log_circuit_size=*/1);
                restrict_to_standard_arithmetic_relation(prover_polynomials);
                instance->prover_polynomials = std::move(prover_polynomials);
                instance->instance_size = 2;
                instance_data[idx] = instance;
            }

            ProverInstances instances{ instance_data };
            instances.alphas.fill(Univariate<FF, 13>(FF(0))); // focus on the arithmetic relation only

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

            create_add_gate(instances[0]->prover_polynomials, 0, 1, 2);
            create_add_gate(instances[0]->prover_polynomials, 1, 0, 4);
            create_add_gate(instances[1]->prover_polynomials, 0, 3, 4);
            create_mul_gate(instances[1]->prover_polynomials, 1, 1, 4);

            restrict_to_standard_arithmetic_relation(instances[0]->prover_polynomials);
            restrict_to_standard_arithmetic_relation(instances[1]->prover_polynomials);

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
            auto result = prover.compute_combiner(instances, pow_polynomial);
            auto expected_result =
                bb::Univariate<FF, 13>(std::array<FF, 13>{ 0, 0, 12, 36, 72, 120, 180, 252, 336, 432, 540, 660, 792 });

            EXPECT_EQ(result, expected_result);
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
        std::fill(polys.q_sort.begin(), polys.q_sort.end(), 0);
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
            auto prover_polynomials = proof_system::honk::get_zero_prover_polynomials<Flavor>(
                /*log_circuit_size=*/1);
            instance->prover_polynomials = std::move(prover_polynomials);
            instance->instance_size = 2;
            instance_data[idx] = instance;
        }

        ProverInstances instances{ instance_data };
        instances.alphas.fill(Univariate<FF, 43>(FF(0))); // focus on the arithmetic relation only

        zero_all_selectors(instances[0]->prover_polynomials);
        zero_all_selectors(instances[1]->prover_polynomials);
        zero_all_selectors(instances[2]->prover_polynomials);
        zero_all_selectors(instances[3]->prover_polynomials);

        auto pow_polynomial = PowPolynomial(std::vector<FF>{ 2 });
        auto result = prover.compute_combiner(instances, pow_polynomial);
        std::array<FF, 43> zeroes;
        std::fill(zeroes.begin(), zeroes.end(), 0);
        auto expected_result = bb::Univariate<FF, 43>(zeroes);
        EXPECT_EQ(result, expected_result);
    };
    run_test();
};

} // namespace bb::test_protogalaxy_prover