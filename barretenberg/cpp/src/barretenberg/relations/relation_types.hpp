#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "nested_containers.hpp"
#include <algorithm>

template <typename T>
concept IsField = std::same_as<T, bb::fr> /* || std::same_as<T, grumpkin::fr> */;

namespace bb {

/**
 * @brief A type to optionally extract a view of a relation parameter in a relation.
 *
 * @details In sumcheck, challenges in relations are always field elements, but in folding we need univariate
 * challenges. This template inspecting the underlying type of a RelationParameters instance. When this type is a field
 * type, do nothing, otherwise apply the provided view type.
 * @tparam Params
 * @tparam View
 * @todo TODO(https://github.com/AztecProtocol/barretenberg/issues/759): Optimize
 */
template <typename Params, typename View>
using GetParameterView = std::conditional_t<IsField<typename Params::DataType>, typename Params::DataType, View>;

template <typename T, size_t subrelation_idx>
concept HasSubrelationLinearlyIndependentMember = requires(T) {
                                                      {
                                                          std::get<subrelation_idx>(T::SUBRELATION_LINEARLY_INDEPENDENT)
                                                          } -> std::convertible_to<bool>;
                                                  };

template <typename T>
concept HasParameterLengthAdjustmentsMember = requires { T::TOTAL_LENGTH_ADJUSTMENTS; };

/**
 * @brief Check whether a given subrelation is linearly independent from the other subrelations.
 *
 * @details More often than not, we want multiply each subrelation contribution by a power of the relation
 * separator challenge. In cases where we wish to define a subrelation that merges into another, we encode this
 * in a boolean array `SUBRELATION_LINEARLY_INDEPENDENT` in the relation. If no such array is defined, then the
 * default case where all subrelations are independent is engaged.
 */
template <typename Relation, size_t subrelation_index> constexpr bool subrelation_is_linearly_independent()
{
    if constexpr (HasSubrelationLinearlyIndependentMember<Relation, subrelation_index>) {
        return std::get<subrelation_index>(Relation::SUBRELATION_LINEARLY_INDEPENDENT);
    } else {
        return true;
    }
}

/**
 * @brief Compute the total subrelation lengths, i.e., the lengths when regarding the challenges as
 * variables.
 */
template <typename RelationImpl>
consteval std::array<size_t, RelationImpl::SUBRELATION_PARTIAL_LENGTHS.size()> compute_total_subrelation_lengths()
{
    if constexpr (HasParameterLengthAdjustmentsMember<RelationImpl>) {
        constexpr size_t NUM_SUBRELATIONS = RelationImpl::SUBRELATION_PARTIAL_LENGTHS.size();
        std::array<size_t, NUM_SUBRELATIONS> result;
        for (size_t idx = 0; idx < NUM_SUBRELATIONS; idx++) {
            result[idx] = RelationImpl::SUBRELATION_PARTIAL_LENGTHS[idx] + RelationImpl::TOTAL_LENGTH_ADJUSTMENTS[idx];
        }
        return result;
    } else {
        return RelationImpl::SUBRELATION_PARTIAL_LENGTHS;
    }
};

/**
 * @brief Get the subrelation accumulators for the Protogalaxy combiner calculation.
 * @details A subrelation of degree D, when evaluated on polynomials of degree N, gives a polynomial of degree D
 * * N. In the context of Protogalaxy, N = NUM_INSTANCES-1. Hence, given a subrelation of length x, its
 * evaluation on such polynomials will have degree (x-1) * (NUM_INSTANCES-1), and the length of this evaluation
 * will be one greater than this.
 * @tparam NUM_INSTANCES
 * @tparam NUM_SUBRELATIONS
 * @param SUBRELATION_PARTIAL_LENGTHS The array of subrelation lengths supplied by a relation.
 * @return The transformed subrelation lenths
 */
template <size_t NUM_INSTANCES, size_t NUM_SUBRELATIONS>
consteval std::array<size_t, NUM_SUBRELATIONS> compute_composed_subrelation_partial_lengths(
    std::array<size_t, NUM_SUBRELATIONS> SUBRELATION_PARTIAL_LENGTHS)
{
    std::transform(SUBRELATION_PARTIAL_LENGTHS.begin(),
                   SUBRELATION_PARTIAL_LENGTHS.end(),
                   SUBRELATION_PARTIAL_LENGTHS.begin(),
                   [](const size_t x) { return (x - 1) * (NUM_INSTANCES - 1) + 1; });
    return SUBRELATION_PARTIAL_LENGTHS;
};

/**
 * @brief The templates defined herein facilitate sharing the relation arithmetic between the prover and the
 * verifier.
 *
 * @details The sumcheck prover and verifier accumulate the contributions from each relation (really, each sub-relation)
 * into, respectively, Univariates and individual field elements. When performing relation arithmetic on
 * Univariates, we introduce UnivariateViews to reduce full length Univariates to the minimum required length
 * and to avoid unnecessary copies.
 *
 * To share the relation arithmetic, we introduce simple structs that specify two types: Accumulators and
 * AccumulatorViews. For the prover, who accumulates Univariates, these are respectively std::tuple<Univariate>
 * and std::tuple<UnivariateView>. For the verifier, who accumulates FFs, both types are simply aliases for
 * std::array<FF> (since no "view" type is necessary). The containers std::tuple and std::array are needed to
 * accommodate multiple sub-relations within each relation, where, for efficiency, each sub-relation has its own
 * specified degree.
 *
 * @note We use some funny terminology: we use the term "length" for 1 + the degree of a relation. When the relation is
 * regarded as a polynomial in all of its arguments, we refer to this length as the "total length", and when we
 * hold the relation parameters constant we refer to it as a "partial length."
 *
 */

/**
 * @brief A wrapper for Relations to expose methods used by the Sumcheck prover or verifier to add the
 * contribution of a given relation to the corresponding accumulator.
 *
 * @tparam FF
 * @tparam RelationImpl Base class that implements the arithmetic for a given relation (or set of sub-relations)
 */
template <typename RelationImpl> class Relation : public RelationImpl {
  public:
    using FF = typename RelationImpl::FF;

    static constexpr std::array<size_t, RelationImpl::SUBRELATION_PARTIAL_LENGTHS.size()> SUBRELATION_TOTAL_LENGTHS =
        compute_total_subrelation_lengths<RelationImpl>();

    static constexpr size_t RELATION_LENGTH = *std::max_element(RelationImpl::SUBRELATION_PARTIAL_LENGTHS.begin(),
                                                                RelationImpl::SUBRELATION_PARTIAL_LENGTHS.end());

    static constexpr size_t TOTAL_RELATION_LENGTH =
        *std::max_element(SUBRELATION_TOTAL_LENGTHS.begin(), SUBRELATION_TOTAL_LENGTHS.end());

    template <size_t NUM_INSTANCES>
    using ProtogalaxyTupleOfUnivariatesOverSubrelations =
        TupleOfUnivariates<FF, compute_composed_subrelation_partial_lengths<NUM_INSTANCES>(SUBRELATION_TOTAL_LENGTHS)>;
    using SumcheckTupleOfUnivariatesOverSubrelations =
        TupleOfUnivariates<FF, RelationImpl::SUBRELATION_PARTIAL_LENGTHS>;
    using SumcheckArrayOfValuesOverSubrelations = ArrayOfValues<FF, RelationImpl::SUBRELATION_PARTIAL_LENGTHS>;

    // These are commonly needed, most importantly, for explicitly instantiating
    // compute_foo_numerator/denomintor.
    using UnivariateAccumulator0 = std::tuple_element_t<0, SumcheckTupleOfUnivariatesOverSubrelations>;
    using ValueAccumulator0 = std::tuple_element_t<0, SumcheckArrayOfValuesOverSubrelations>;
};
} // namespace bb
