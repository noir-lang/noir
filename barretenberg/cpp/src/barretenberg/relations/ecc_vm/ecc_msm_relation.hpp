#pragma once
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

/**
 * @brief MSM relations that evaluate the Strauss multiscalar multiplication algorithm.
 *
 * @details
 * The Strauss algorithm for a size-k MSM takes scalars/points (a_i, [P_i]) for i = 0 to k-1.
 * The specific algoritm we use is the following:
 *
 * PHASE 1: Precomputation (performed in ecc_wnaf_relation.hpp, ecc_point_table_relation.hpp)
 * Each scalar a_i is split into 4-bit WNAF slices s_{j, i} for j = 0 to 31, and a skew bool skew_i
 * For each point [P_i] a size-16 lookup table of points, T_i, is computed { [-15 P_i], [-13 P_i], ..., [15 P_i] }
 *
 * PHASE 2: MSM evaluation
 * MSM evaluation is split into 32 rounds that operate on an accumulator point [Acc]
 * The first 31 rounds are composed of an ADDITION round and a DOUBLE round.
 * The final 32nd round is composed of an ADDITION round and a SKEW round.
 *
 * ADDITION round (round = j):
 * [Acc] = [Acc] + T_i[a_{i, j}] for all i in [0, ... k-1]
 *
 * DOUBLE round:
 * [Acc] = 16 * [Acc] (four point doublings)
 *
 * SKEW round:
 * If skew_i == 1, [Acc] = [Acc] - [P_i] for all i in [0, ..., k - 1]
 *
 * The relations in ECCVMMSMRelationImpl constrain the ADDITION, DOUBLE and SKEW rounds
 * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
 * @param in an std::array containing the fully extended Accumulator edges.
 * @param parameters contains beta, gamma, and public_input_delta, ....
 * @param scaling_factor optional term to scale the evaluation before adding to evals.
 */
template <typename FF_> class ECCVMMSMRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 36> SUBRELATION_PARTIAL_LENGTHS{ 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                                                                         8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                                                                         8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8 };
    /**
     * @brief For ZK-Flavors: Upper bound on the degrees of subrelations considered as polynomials only in witness
polynomials,
     * i.e. all selectors and public polynomials are treated as constants. The subrelation witness degree does not
     * exceed the subrelation partial degree given by SUBRELATION_PARTIAL_LENGTH - 1.
     */
    static constexpr std::array<size_t, 36> SUBRELATION_WITNESS_DEGREES{ 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
                                                                         7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
                                                                         7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7 };

    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& /* unused */,
                           const FF& scaling_factor);
};

template <typename FF> using ECCVMMSMRelation = Relation<ECCVMMSMRelationImpl<FF>>;

} // namespace bb