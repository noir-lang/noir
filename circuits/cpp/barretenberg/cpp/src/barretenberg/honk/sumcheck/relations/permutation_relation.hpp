#pragma once
#include "../polynomials/univariate.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "relation_parameters.hpp"
#include "relation_types.hpp"
// TODO(luke): change name of this file to permutation_grand_product_relation(s).hpp and move 'init' relation into it.

namespace proof_system::honk::sumcheck {

template <typename FF> class PermutationRelationBase {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 5;

    static constexpr size_t LEN_1 = 5; // grand product construction sub-relation
    static constexpr size_t LEN_2 = 3; // left-shiftable polynomial sub-relation
    template <template <size_t...> typename AccumulatorTypesContainer>
    using AccumulatorTypesBase = AccumulatorTypesContainer<LEN_1, LEN_2>;
    template <typename T> using Accumulator = typename std::tuple_element<0, typename T::Accumulators>::type;

    inline static auto& get_grand_product_polynomial(auto& input) { return input.z_perm; }
    inline static auto& get_shifted_grand_product_polynomial(auto& input) { return input.z_perm_shift; }

    template <typename AccumulatorTypes>
    inline static Accumulator<AccumulatorTypes> compute_grand_product_numerator(
        const auto& input, const RelationParameters<FF>& relation_parameters, const size_t index)
    {
        auto w_1 = get_view<FF, AccumulatorTypes>(input.w_l, index);
        auto w_2 = get_view<FF, AccumulatorTypes>(input.w_r, index);
        auto w_3 = get_view<FF, AccumulatorTypes>(input.w_o, index);
        auto id_1 = get_view<FF, AccumulatorTypes>(input.id_1, index);
        auto id_2 = get_view<FF, AccumulatorTypes>(input.id_2, index);
        auto id_3 = get_view<FF, AccumulatorTypes>(input.id_3, index);

        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;

        return (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) * (w_3 + id_3 * beta + gamma);
    }

    template <typename AccumulatorTypes>
    inline static Accumulator<AccumulatorTypes> compute_grand_product_denominator(
        const auto& input, const RelationParameters<FF>& relation_parameters, const size_t index)
    {
        //  using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
        auto w_1 = get_view<FF, AccumulatorTypes>(input.w_l, index);
        auto w_2 = get_view<FF, AccumulatorTypes>(input.w_r, index);
        auto w_3 = get_view<FF, AccumulatorTypes>(input.w_o, index);
        auto sigma_1 = get_view<FF, AccumulatorTypes>(input.sigma_1, index);
        auto sigma_2 = get_view<FF, AccumulatorTypes>(input.sigma_2, index);
        auto sigma_3 = get_view<FF, AccumulatorTypes>(input.sigma_3, index);

        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;

        return (w_1 + sigma_1 * beta + gamma) * (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma);
    }
    /**
     * @brief Compute contribution of the permutation relation for a given edge (internal function)
     *
     * @details There are 2 relations associated with enforcing the wire copy relations
     * This file handles the relation that confirms faithful calculation of the grand
     * product polynomial Z_perm. (Initialization relation Z_perm(0) = 1 is handled elsewhere).
     *
     *  C(extended_edges(X)...) =
     *      ( z_perm(X) + lagrange_first(X) )*P(X)
     *         - ( z_perm_shift(X) + delta * lagrange_last(X))*Q(X),
     * where P(X) = Prod_{i=1:3} w_i(X) + β*(n*(i-1) + idx(X)) + γ
     *       Q(X) = Prod_{i=1:3} w_i(X) + β*σ_i(X) + γ
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename AccumulatorTypes>
    inline static void add_edge_contribution_impl(typename AccumulatorTypes::Accumulators& accumulator,
                                                  const auto& input,
                                                  const RelationParameters<FF>& relation_parameters,
                                                  const FF& scaling_factor)
    {
        const auto& public_input_delta = relation_parameters.public_input_delta;

        {
            using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
            auto z_perm = View(input.z_perm);
            auto z_perm_shift = View(input.z_perm_shift);
            auto lagrange_first = View(input.lagrange_first);
            auto lagrange_last = View(input.lagrange_last);

            // Contribution (1)
            std::get<0>(accumulator) +=
                (((z_perm + lagrange_first) *
                  compute_grand_product_numerator<AccumulatorTypes>(input, relation_parameters, 0)) -
                 ((z_perm_shift + lagrange_last * public_input_delta) *
                  compute_grand_product_denominator<AccumulatorTypes>(input, relation_parameters, 0))) *
                scaling_factor;
        }
        {
            using View = typename std::tuple_element<1, typename AccumulatorTypes::AccumulatorViews>::type;
            auto z_perm_shift = View(input.z_perm_shift);
            auto lagrange_last = View(input.lagrange_last);

            // Contribution (2)
            std::get<1>(accumulator) += (lagrange_last * z_perm_shift) * scaling_factor;
        }
    };
};

// TODO(luke): With Cody's Flavor work it should be easier to create a simple templated relation
// for handling arbitrary width. For now I'm duplicating the width 3 logic for width 4.
template <typename FF> class UltraPermutationRelationBase {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6;

    static constexpr size_t LEN_1 = 6; // grand product construction sub-relation
    static constexpr size_t LEN_2 = 3; // left-shiftable polynomial sub-relation
    template <template <size_t...> typename AccumulatorTypesContainer>
    using AccumulatorTypesBase = AccumulatorTypesContainer<LEN_1, LEN_2>;
    template <typename T> using Accumulator = typename std::tuple_element<0, typename T::Accumulators>::type;

    inline static auto& get_grand_product_polynomial(auto& input) { return input.z_perm; }
    inline static auto& get_shifted_grand_product_polynomial(auto& input) { return input.z_perm_shift; }

    template <typename AccumulatorTypes>
    inline static Accumulator<AccumulatorTypes> compute_grand_product_numerator(
        const auto& input, const RelationParameters<FF>& relation_parameters, const size_t index)
    {
        auto w_1 = get_view<FF, AccumulatorTypes>(input.w_l, index);
        auto w_2 = get_view<FF, AccumulatorTypes>(input.w_r, index);
        auto w_3 = get_view<FF, AccumulatorTypes>(input.w_o, index);
        auto w_4 = get_view<FF, AccumulatorTypes>(input.w_4, index);
        auto id_1 = get_view<FF, AccumulatorTypes>(input.id_1, index);
        auto id_2 = get_view<FF, AccumulatorTypes>(input.id_2, index);
        auto id_3 = get_view<FF, AccumulatorTypes>(input.id_3, index);
        auto id_4 = get_view<FF, AccumulatorTypes>(input.id_4, index);

        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;

        return (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) * (w_3 + id_3 * beta + gamma) *
               (w_4 + id_4 * beta + gamma);
    }

    template <typename AccumulatorTypes>
    inline static Accumulator<AccumulatorTypes> compute_grand_product_denominator(
        const auto& input, const RelationParameters<FF>& relation_parameters, const size_t index)
    {
        auto w_1 = get_view<FF, AccumulatorTypes>(input.w_l, index);
        auto w_2 = get_view<FF, AccumulatorTypes>(input.w_r, index);
        auto w_3 = get_view<FF, AccumulatorTypes>(input.w_o, index);
        auto w_4 = get_view<FF, AccumulatorTypes>(input.w_4, index);

        auto sigma_1 = get_view<FF, AccumulatorTypes>(input.sigma_1, index);
        auto sigma_2 = get_view<FF, AccumulatorTypes>(input.sigma_2, index);
        auto sigma_3 = get_view<FF, AccumulatorTypes>(input.sigma_3, index);
        auto sigma_4 = get_view<FF, AccumulatorTypes>(input.sigma_4, index);

        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;

        return (w_1 + sigma_1 * beta + gamma) * (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma) *
               (w_4 + sigma_4 * beta + gamma);
    }

    /**
     * @brief Compute contribution of the permutation relation for a given edge (internal function)
     *
     * @details This the relation confirms faithful calculation of the grand
     * product polynomial Z_perm.
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename AccumulatorTypes>
    inline static void add_edge_contribution_impl(typename AccumulatorTypes::Accumulators& accumulators,
                                                  const auto& extended_edges,
                                                  const RelationParameters<FF>& relation_parameters,
                                                  const FF& scaling_factor)
    {
        const auto& public_input_delta = relation_parameters.public_input_delta;

        // Contribution (1)
        {
            using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
            auto z_perm = View(extended_edges.z_perm);
            auto z_perm_shift = View(extended_edges.z_perm_shift);
            auto lagrange_first = View(extended_edges.lagrange_first);
            auto lagrange_last = View(extended_edges.lagrange_last);

            // Contribution (1)
            std::get<0>(accumulators) +=
                (((z_perm + lagrange_first) *
                  compute_grand_product_numerator<AccumulatorTypes>(extended_edges, relation_parameters, 0)) -
                 ((z_perm_shift + lagrange_last * public_input_delta) *
                  compute_grand_product_denominator<AccumulatorTypes>(extended_edges, relation_parameters, 0))) *
                scaling_factor;
        }
        // Contribution (2)
        {
            using View = typename std::tuple_element<1, typename AccumulatorTypes::AccumulatorViews>::type;
            auto z_perm_shift = View(extended_edges.z_perm_shift);
            auto lagrange_last = View(extended_edges.lagrange_last);

            std::get<1>(accumulators) += (lagrange_last * z_perm_shift) * scaling_factor;
        }
    };
};

template <typename FF> using PermutationRelation = RelationWrapper<FF, PermutationRelationBase>;

template <typename FF> using UltraPermutationRelation = RelationWrapper<FF, UltraPermutationRelationBase>;
} // namespace proof_system::honk::sumcheck
