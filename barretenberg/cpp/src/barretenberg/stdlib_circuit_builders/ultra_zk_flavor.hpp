#pragma once

#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"

namespace bb {

/*!
\brief Child class of UltraFlavor that runs with ZK Sumcheck.
\details
Most of the properties of UltraFlavor are
inherited without any changes, except for the MAX_PARTIAL_RELATION_LENGTH which is now computed as a maximum of
SUBRELATION_PARTIAL_LENGTHS incremented by the corresponding SUBRELATION_WITNESS_DEGREES over all relations included in
UltraFlavor, which also affects the size of ExtendedEdges univariate containers.
Moreover, the container SumcheckTupleOfTuplesOfUnivariates is resized to reflect that masked
witness polynomials are of degree at most \f$2\f$ in each variable, and hence, for any subrelation, the corresponding
univariate accumuluator size has to be increased by the subrelation's witness degree. See more in
\ref docs/src/sumcheck-outline.md "Sumcheck Outline".
*/
class UltraFlavorWithZK : public bb::UltraFlavor {
  public:
    // This flavor runs with ZK Sumcheck
    static constexpr bool HasZK = true;
    // Compute the maximum over all partial subrelation lengths incremented by the corresponding subrelation witness
    // degrees for the Relations inherited from UltraFlavor
    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_total_relation_length<Relations, HasZK>();
    // Determine the number of evaluations of Prover and Libra Polynomials that the Prover sends to the Verifier in
    // the rounds of ZK Sumcheck.
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    // Construct the container for the subrelations' contributions
    using SumcheckTupleOfTuplesOfUnivariates =
        decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations, HasZK>());
    // Re-define ExtendedEdges to account for the incremented MAX_PARTIAL_RELATION_LENGTH
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;
};

} // namespace bb