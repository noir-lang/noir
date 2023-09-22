#pragma once
#include "relation_parameters.hpp"
#include "relation_types.hpp"

namespace proof_system {

template <typename FF_> class UltraPermutationRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6;

    static constexpr size_t LEN_1 = 6; // grand product construction sub-relation
    static constexpr size_t LEN_2 = 3; // left-shiftable polynomial sub-relation
    template <template <size_t...> typename SubrelationAccumulatorsTemplate>
    using GetAccumulatorTypes = SubrelationAccumulatorsTemplate<LEN_1, LEN_2>;
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
    inline static void accumulate(typename AccumulatorTypes::Accumulators& accumulators,
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

template <typename FF> using UltraPermutationRelation = Relation<UltraPermutationRelationImpl<FF>>;

} // namespace proof_system
