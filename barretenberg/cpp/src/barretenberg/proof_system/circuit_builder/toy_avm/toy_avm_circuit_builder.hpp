/**
 * @file avm_template_circuit_builder.hpp
 * @author Rumata888
 * @brief A circuit builder for the AVM toy version used to showcase permutation and lookup mechanisms for PIL
 *
 */
#pragma once

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/flavor/toy_avm.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/toy_avm/generic_permutation_relation.hpp"

namespace proof_system {

/**
 * @brief Circuit builder for the ToyAVM that is used to explain generic permutation settings
 *
 * @tparam Flavor
 */
template <typename Flavor> class ToyAVMCircuitBuilder {
  public:
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

    static constexpr size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    static constexpr size_t NUM_WIRES = Flavor::NUM_WIRES;

    using AllPolynomials = typename Flavor::AllPolynomials;
    size_t num_gates = 0;
    std::array<std::vector<FF>, NUM_WIRES> wires;
    ToyAVMCircuitBuilder() = default;

    void add_row(const std::array<FF, NUM_WIRES> row)
    {
        for (size_t i = 0; i < NUM_WIRES; i++) {
            wires[i].emplace_back(row[i]);
        }
        num_gates = wires[0].size();
    }

    /**
     * @brief Compute the AVM Template flavor polynomial data required to generate a proof
     *
     * @return AllPolynomials
     */
    AllPolynomials compute_polynomials()
    {

        const auto num_gates_log2 = static_cast<size_t>(numeric::get_msb64(num_gates));
        size_t num_gates_pow2 = 1UL << (num_gates_log2 + (1UL << num_gates_log2 == num_gates ? 0 : 1));

        AllPolynomials polys;
        for (auto& poly : polys.get_all()) {
            poly = Polynomial(num_gates_pow2);
        }

        polys.lagrange_first[0] = 1;

        for (size_t i = 0; i < num_gates; ++i) {
            // Fill out the witness polynomials
            polys.permutation_set_column_1[i] = wires[0][i];
            polys.permutation_set_column_2[i] = wires[1][i];
            polys.permutation_set_column_3[i] = wires[2][i];
            polys.permutation_set_column_4[i] = wires[3][i];
            polys.self_permutation_column[i] = wires[4][i];
            // By default the permutation is over all rows where we place data
            polys.enable_tuple_set_permutation[i] = 1;
            // The same column permutation alternates between even and odd values
            polys.enable_single_column_permutation[i] = 1;
            polys.enable_first_set_permutation[i] = i & 1;
            polys.enable_second_set_permutation[i] = 1 - (i & 1);
        }
        return polys;
    }

    /**
     * @brief Check that the circuit is correct (proof should work)
     *
     */
    bool check_circuit()
    {
        //        using FirstPermutationRelation = typename std::tuple_element_t<0, Flavor::Relations>;
        // For now only gamma and beta are used
        const FF gamma = FF::random_element();
        const FF beta = FF::random_element();
        proof_system::RelationParameters<typename Flavor::FF> params{
            .eta = 0,
            .beta = beta,
            .gamma = gamma,
            .public_input_delta = 0,
            .lookup_grand_product_delta = 0,
            .beta_sqr = 0,
            .beta_cube = 0,
            .eccvm_set_permutation_delta = 0,
        };

        // Compute polynomial values
        auto polynomials = compute_polynomials();
        const size_t num_rows = polynomials.get_polynomial_size();

        // Check the tuple permutation relation
        proof_system::honk::logderivative_library::compute_logderivative_inverse<
            Flavor,
            honk::sumcheck::GenericPermutationRelation<honk::sumcheck::ExampleTuplePermutationSettings, FF>>(
            polynomials, params, num_rows);

        using PermutationRelation =
            honk::sumcheck::GenericPermutationRelation<honk::sumcheck::ExampleTuplePermutationSettings, FF>;
        typename honk::sumcheck::GenericPermutationRelation<honk::sumcheck::ExampleTuplePermutationSettings,
                                                            typename Flavor::FF>::SumcheckArrayOfValuesOverSubrelations
            permutation_result;
        for (auto& r : permutation_result) {
            r = 0;
        }
        for (size_t i = 0; i < num_rows; ++i) {
            PermutationRelation::accumulate(permutation_result, polynomials.get_row(i), params, 1);
        }
        for (auto r : permutation_result) {
            if (r != 0) {
                info("Tuple GenericPermutationRelation failed.");
                return false;
            }
        }
        // Check the single permutation relation
        proof_system::honk::logderivative_library::compute_logderivative_inverse<
            Flavor,
            honk::sumcheck::GenericPermutationRelation<honk::sumcheck::ExampleSameWirePermutationSettings, FF>>(
            polynomials, params, num_rows);

        using SameWirePermutationRelation =
            honk::sumcheck::GenericPermutationRelation<honk::sumcheck::ExampleSameWirePermutationSettings, FF>;
        typename honk::sumcheck::GenericPermutationRelation<honk::sumcheck::ExampleSameWirePermutationSettings,
                                                            typename Flavor::FF>::SumcheckArrayOfValuesOverSubrelations
            second_permutation_result;
        for (auto& r : second_permutation_result) {
            r = 0;
        }
        for (size_t i = 0; i < num_rows; ++i) {
            SameWirePermutationRelation::accumulate(second_permutation_result, polynomials.get_row(i), params, 1);
        }
        for (auto r : second_permutation_result) {
            if (r != 0) {
                info("Same wire  GenericPermutationRelation failed.");
                return false;
            }
        }
        return true;
    }

    [[nodiscard]] size_t get_num_gates() const { return num_gates; }

    [[nodiscard]] size_t get_circuit_subgroup_size(const size_t num_rows) const
    {

        const auto num_rows_log2 = static_cast<size_t>(numeric::get_msb64(num_rows));
        size_t num_rows_pow2 = 1UL << (num_rows_log2 + (1UL << num_rows_log2 == num_rows ? 0 : 1));
        return num_rows_pow2;
    }
};
} // namespace proof_system
