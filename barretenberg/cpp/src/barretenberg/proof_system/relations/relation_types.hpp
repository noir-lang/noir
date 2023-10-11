#pragma once
#include "nested_containers.hpp"
#include "relation_parameters.hpp"
#include <algorithm>

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
 * @brief Check whether a given subrelation is linearly independent from the other subrelations.
 *
 * @details More often than not, we want multiply each subrelation contribution by a power of the relation separator
 * challenge. In cases where we wish to define a subrelation that merges into another, we encode this in a boolean array
 * `SUBRELATION_LINEARLY_INDEPENDENT` in the relation. If no such array is defined, then the default case where all
 * subrelations are independent is engaged.
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
 * @brief A wrapper for Relations to expose methods used by the Sumcheck prover or verifier to add the contribution of
 * a given relation to the corresponding accumulator.
 *
 * @tparam FF
 * @tparam RelationImpl Base class that implements the arithmetic for a given relation (or set of sub-relations)
 */
template <typename RelationImpl> class Relation : public RelationImpl {
  public:
    using FF = typename RelationImpl::FF;

    static constexpr size_t RELATION_LENGTH =
        *std::max_element(RelationImpl::SUBRELATION_LENGTHS.begin(), RelationImpl::SUBRELATION_LENGTHS.end());

    using TupleOfUnivariatesOverSubrelations = TupleOfUnivariates<FF, RelationImpl::SUBRELATION_LENGTHS>;
    using ArrayOfValuesOverSubrelations = ArrayOfValues<FF, RelationImpl::SUBRELATION_LENGTHS>;

    // These are commonly needed, most importantly, for explicitly instantiating compute_foo_numerator/denomintor.
    using UnivariateAccumulator0 = std::tuple_element_t<0, TupleOfUnivariatesOverSubrelations>;
    using ValueAccumulator0 = std::tuple_element_t<0, ArrayOfValuesOverSubrelations>;
};

} // namespace proof_system
