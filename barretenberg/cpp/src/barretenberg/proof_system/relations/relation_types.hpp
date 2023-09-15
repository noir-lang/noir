#pragma once
#include "barretenberg/polynomials/univariate.hpp"
#include "relation_parameters.hpp"

namespace barretenberg {
template <typename FF> class Polynomial;
}
namespace proof_system {

// forward-declare Polynomial so we can use in a concept

template <typename T, size_t subrelation_idx>
concept HasSubrelationLinearlyIndependentMember = requires(T) {
                                                      {
                                                          std::get<subrelation_idx>(T::SUBRELATION_LINEARLY_INDEPENDENT)
                                                          } -> std::convertible_to<bool>;
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
 *
 * @todo TODO(https://github.com/AztecProtocol/barretenberg/issues/720)
 *
 */

/**
 * @brief Getter method that will return `input[index]` iff `input` is a std::span container
 *
 * @return requires
 */
template <typename FF, typename AccumulatorTypes, typename T>
    requires std::is_same<std::span<FF>, T>::value
inline typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type get_view(const T& input,
                                                                                                  const size_t index)
{
    return input[index];
}

/**
 * @brief Getter method that will return `input[index]` iff `input` is a Polynomial container
 *
 * @tparam FF
 * @tparam TypeMuncher
 * @tparam T
 * @param input
 * @param index
 * @return requires
 */
template <typename FF, typename AccumulatorTypes, typename T>
    requires std::is_same<barretenberg::Polynomial<FF>, T>::value
inline typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type get_view(const T& input,
                                                                                                  const size_t index)
{
    return input[index];
}

/**
 * @brief Getter method that will return `input[index]` iff `input` is not a std::span or a Polynomial container
 *
 * @return requires
 */
template <typename FF, typename AccumulatorTypes, typename T>
inline typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type get_view(
    const T& input, const size_t /*unused*/)
{
    return typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type(input);
}

/**
 * @brief A wrapper for Relations to expose methods used by the Sumcheck prover or verifier to add the contribution of
 * a given relation to the corresponding accumulator.
 *
 * @tparam FF
 * @tparam RelationImpl Base class that implements the arithmetic for a given relation (or set of sub-relations)
 */
template <typename RelationImpl> class Relation : public RelationImpl {
  private:
    using FF = typename RelationImpl::FF;
    template <size_t... subrelation_lengths> struct UnivariateAccumulatorsAndViewsTemplate {
        using Accumulators = std::tuple<barretenberg::Univariate<FF, subrelation_lengths>...>;
        using AccumulatorViews = std::tuple<barretenberg::UnivariateView<FF, subrelation_lengths>...>;
    };
    template <size_t... subrelation_lengths> struct ValueAccumulatorsAndViewsTemplate {
        using Accumulators = std::array<FF, sizeof...(subrelation_lengths)>;
        using AccumulatorViews = std::array<FF, sizeof...(subrelation_lengths)>; // there is no "view" type here
    };

  public:
    // Each `RelationImpl` defines a template `GetAccumulatorTypes` that supplies the `subrelation_lengths` parameters
    // of the different `AccumulatorsAndViewsTemplate`s.
    using UnivariateAccumulatorsAndViews =
        typename RelationImpl::template GetAccumulatorTypes<UnivariateAccumulatorsAndViewsTemplate>;
    // In the case of the value accumulator types, only the number of subrelations (not their lengths) has an effect.
    using ValueAccumulatorsAndViews =
        typename RelationImpl::template GetAccumulatorTypes<ValueAccumulatorsAndViewsTemplate>;

    using RelationUnivariates = typename UnivariateAccumulatorsAndViews::Accumulators;
    using RelationValues = typename ValueAccumulatorsAndViews::Accumulators;
    static constexpr size_t RELATION_LENGTH = RelationImpl::RELATION_LENGTH;

    static inline void add_edge_contribution(RelationUnivariates& accumulator,
                                             const auto& input,
                                             const RelationParameters<FF>& relation_parameters,
                                             const FF& scaling_factor)
    {
        Relation::template accumulate<UnivariateAccumulatorsAndViews>(
            accumulator, input, relation_parameters, scaling_factor);
    }

    static void add_full_relation_value_contribution(RelationValues& accumulator,
                                                     auto& input,
                                                     const RelationParameters<FF>& relation_parameters,
                                                     const FF& scaling_factor = 1)
    {
        Relation::template accumulate<ValueAccumulatorsAndViews>(
            accumulator, input, relation_parameters, scaling_factor);
    }
    /**
     * @brief Check is subrelation is linearly independent
     * Method is active if relation has SUBRELATION_LINEARLY_INDEPENDENT array defined
     * @tparam size_t
     */
    template <size_t subrelation_index>
    static constexpr bool is_subrelation_linearly_independent()
        requires(!HasSubrelationLinearlyIndependentMember<Relation, subrelation_index>)
    {
        return true;
    }

    /**
     * @brief Check is subrelation is linearly independent
     * Method is active if relation has SUBRELATION_LINEARLY_INDEPENDENT array defined
     * @tparam size_t
     */
    template <size_t subrelation_index>
    static constexpr bool is_subrelation_linearly_independent()
        requires(HasSubrelationLinearlyIndependentMember<Relation, subrelation_index>)
    {
        return std::get<subrelation_index>(Relation::SUBRELATION_LINEARLY_INDEPENDENT);
    }
};

} // namespace proof_system
