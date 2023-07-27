#pragma once
#include <array>
#include <tuple>

#include "../polynomials/univariate.hpp"
#include "relation_parameters.hpp"

namespace proof_system::honk::sumcheck {
template <typename T> concept HasSubrelationLinearlyIndependentMember = requires(T)
{
    T::Relation::SUBRELATION_LINEARLY_INDEPENDENT;
};
/**
 * @brief The templates defined herein facilitate sharing the relation arithmetic between the prover and the verifier.
 *
 * The sumcheck prover and verifier accumulate the contributions from each relation (really, each sub-relation) into,
 * respectively, Univariates and individual field elements. When performing relation arithmetic on Univariates, we
 * introduce UnivariateViews to reduce full length Univariates to the minimum required length and to avoid unnecessary
 * copies.
 *
 * To share the relation arithmetic, we introduce simple structs that specify two types: Accumulators and
 * AccumulatorViews. For the prover, who accumulates Univariates, these are respectively std::tuple<Univariate> and
 * std::tuple<UnivariateView>. For the verifier, who accumulates FFs, both types are simply aliases for std::array<FF>
 * (since no "view" type is necessary). The containers std::tuple and std::array are needed to accommodate multiple
 * sub-relations within each relation, where, for efficiency, each sub-relation has its own specified degree.
 */

/**
 * @brief Getter method that will return `input[index]` iff `input` is a std::span container
 *
 * @tparam FF
 * @tparam TypeMuncher
 * @tparam T
 * @param input
 * @param index
 * @return requires
 */
template <typename FF, typename AccumulatorTypes, typename T>
requires std::is_same<std::span<FF>, T>::value inline
    typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type
    get_view(const T& input, const size_t index)
{
    return input[index];
}

/**
 * @brief Getter method that will return `input[index]` iff `input` is not a std::span container
 *
 * @tparam FF
 * @tparam TypeMuncher
 * @tparam T
 * @param input
 * @param index
 * @return requires
 */
template <typename FF, typename AccumulatorTypes, typename T>
inline typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type get_view(const T& input,
                                                                                                  const size_t)
{
    return typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type(input);
}

/**
 * @brief A wrapper for Relations to expose methods used by the Sumcheck prover or verifier to add the contribution of
 * a given relation to the corresponding accumulator.
 *
 * @tparam FF
 * @tparam RelationBase Base class that implements the arithmetic for a given relation (or set of sub-relations)
 */
template <typename FF, template <typename> typename RelationBase> class RelationWrapper : public RelationBase<FF> {
  private:
    template <size_t... Values> struct UnivariateAccumulatorTypes {
        using Accumulators = std::tuple<Univariate<FF, Values>...>;
        using AccumulatorViews = std::tuple<UnivariateView<FF, Values>...>;
    };
    template <size_t... Values> struct ValueAccumulatorTypes {
        using Accumulators = std::array<FF, sizeof...(Values)>;
        using AccumulatorViews = std::array<FF, sizeof...(Values)>; // there is no "view" type here
    };

  public:
    using Relation = RelationBase<FF>;
    using UnivariateAccumTypes = typename Relation::template AccumulatorTypesBase<UnivariateAccumulatorTypes>;
    using ValueAccumTypes = typename Relation::template AccumulatorTypesBase<ValueAccumulatorTypes>;

    using RelationUnivariates = typename UnivariateAccumTypes::Accumulators;
    using RelationValues = typename ValueAccumTypes::Accumulators;
    static constexpr size_t RELATION_LENGTH = Relation::RELATION_LENGTH;

    inline void add_edge_contribution(auto& accumulator,
                                      const auto& input,
                                      const RelationParameters<FF>& relation_parameters,
                                      const FF& scaling_factor) const
    {
        Relation::template add_edge_contribution_impl<UnivariateAccumTypes>(
            accumulator, input, relation_parameters, scaling_factor);
    }

    void add_full_relation_value_contribution(RelationValues& accumulator,
                                              auto& input,
                                              const RelationParameters<FF>& relation_parameters,
                                              const FF& scaling_factor = 1) const
    {
        Relation::template add_edge_contribution_impl<ValueAccumTypes>(
            accumulator, input, relation_parameters, scaling_factor);
    }

    /**
     * @brief Check is subrelation is linearly independent
     * Method always returns true if relation has no SUBRELATION_LINEARLY_INDEPENDENT std::array
     * (i.e. default is to make linearly independent)
     * @tparam size_t
     */
    template <size_t>
    static constexpr bool is_subrelation_linearly_independent() requires(
        !HasSubrelationLinearlyIndependentMember<Relation>)
    {
        return true;
    }

    /**
     * @brief Check is subrelation is linearly independent
     * Method is active if relation has SUBRELATION_LINEARLY_INDEPENDENT array defined
     * @tparam size_t
     */
    template <size_t subrelation_index>
    static constexpr bool is_subrelation_linearly_independent() requires(
        HasSubrelationLinearlyIndependentMember<Relation>)
    {
        return std::get<subrelation_index>(Relation::SUBRELATION_LINEARLY_INDEPENDENT);
    }
};

} // namespace proof_system::honk::sumcheck
