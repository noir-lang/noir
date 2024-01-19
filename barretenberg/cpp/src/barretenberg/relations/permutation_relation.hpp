#pragma once
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class UltraPermutationRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        6, // grand product construction sub-relation
        3  // left-shiftable polynomial sub-relation
    };

    static constexpr std::array<size_t, 2> TOTAL_LENGTH_ADJUSTMENTS{
        5, // grand product construction sub-relation
        0  // left-shiftable polynomial sub-relation
    };

    inline static auto& get_grand_product_polynomial(auto& in) { return in.z_perm; }
    inline static auto& get_shifted_grand_product_polynomial(auto& in) { return in.z_perm_shift; }

    template <typename Accumulator, typename AllEntities, typename Parameters>
    inline static Accumulator compute_grand_product_numerator(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;
        using ParameterView = GetParameterView<Parameters, View>;

        auto w_1 = View(in.w_l);
        auto w_2 = View(in.w_r);
        auto w_3 = View(in.w_o);
        auto w_4 = View(in.w_4);
        auto id_1 = View(in.id_1);
        auto id_2 = View(in.id_2);
        auto id_3 = View(in.id_3);
        auto id_4 = View(in.id_4);

        const auto& beta = ParameterView(params.beta);
        const auto& gamma = ParameterView(params.gamma);

        // witness degree 4; fully degree 8
        return (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) * (w_3 + id_3 * beta + gamma) *
               (w_4 + id_4 * beta + gamma);
    }

    template <typename Accumulator, typename AllEntities, typename Parameters>
    inline static Accumulator compute_grand_product_denominator(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;
        using ParameterView = GetParameterView<Parameters, View>;

        auto w_1 = View(in.w_l);
        auto w_2 = View(in.w_r);
        auto w_3 = View(in.w_o);
        auto w_4 = View(in.w_4);

        auto sigma_1 = View(in.sigma_1);
        auto sigma_2 = View(in.sigma_2);
        auto sigma_3 = View(in.sigma_3);
        auto sigma_4 = View(in.sigma_4);

        const auto& beta = ParameterView(params.beta);
        const auto& gamma = ParameterView(params.gamma);

        // witness degree 4; fully degree 8
        return (w_1 + sigma_1 * beta + gamma) * (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma) *
               (w_4 + sigma_4 * beta + gamma);
    }

    /**
     * @brief Compute contribution of the permutation relation for a given edge (internal function)
     *
     * @details This the relation confirms faithful calculation of the grand
     * product polynomial Z_perm.
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    inline static void accumulate(ContainerOverSubrelations& accumulators,
                                  const AllEntities& in,
                                  const Parameters& params,
                                  const FF& scaling_factor)
    {
        // Contribution (1)
        [&]() {
            using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
            using View = typename Accumulator::View;
            using ParameterView = GetParameterView<Parameters, View>;
            const auto public_input_delta = ParameterView(params.public_input_delta);
            const auto z_perm = View(in.z_perm);
            const auto z_perm_shift = View(in.z_perm_shift);
            const auto lagrange_first = View(in.lagrange_first);
            const auto lagrange_last = View(in.lagrange_last);

            // witness degree: deg 5 - deg 5 = deg 5
            // total degree: deg 9 - deg 10 = deg 10
            std::get<0>(accumulators) +=
                (((z_perm + lagrange_first) * compute_grand_product_numerator<Accumulator>(in, params)) -
                 ((z_perm_shift + lagrange_last * public_input_delta) *
                  compute_grand_product_denominator<Accumulator>(in, params))) *
                scaling_factor;
        }();

        // Contribution (2)
        [&]() {
            using Accumulator = std::tuple_element_t<1, ContainerOverSubrelations>;
            using View = typename Accumulator::View;
            auto z_perm_shift = View(in.z_perm_shift);
            auto lagrange_last = View(in.lagrange_last);

            std::get<1>(accumulators) += (lagrange_last * z_perm_shift) * scaling_factor;
        }();
    };
};

template <typename FF> using UltraPermutationRelation = Relation<UltraPermutationRelationImpl<FF>>;

} // namespace bb
