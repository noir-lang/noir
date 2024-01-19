#pragma once
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class GoblinTranslatorOpcodeConstraintRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 7; // degree(op(op - 1)(op - 2)(op - 3)(op - 4)(op - 8)) = 6
    static constexpr std::array<size_t, 1> SUBRELATION_PARTIAL_LENGTHS{
        7 // opcode constraint relation
    };

    /**
     * @brief Expression for enforcing the value of the Opcode to be {0,1,2,3,4,8}
     * @details This relation enforces the opcode to be one of described values. Since we don't care about even
     * values in the opcode wire and usually just set them to zero, we don't use a lagrange polynomial to specify
     * the relation to be enforced just at odd indices, which brings the degree down by 1.
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulators,
                           const AllEntities& in,
                           const Parameters&,
                           const FF& scaling_factor);
};

template <typename FF_> class GoblinTranslatorAccumulatorTransferRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 3; // degree((SOME_LAGRANGE)(A-B)) = 2
    static constexpr std::array<size_t, 12> SUBRELATION_PARTIAL_LENGTHS{
        3, // transfer accumulator limb 0 at even index subrelation
        3, // transfer accumulator limb 1 at even index subrelation
        3, // transfer accumulator limb 2 at even index subrelation
        3, // transfer accumulator limb 3 at even index subrelation
        3, // accumulator limb 0 is zero at the start of accumulation subrelation
        3, // accumulator limb 1 is zero at the start of accumulation subrelation
        3, // accumulator limb 2 is zero at the start of accumulation subrelation
        3, // accumulator limb 3 is zero at the start of accumulation subrelation
        3, // accumulator limb 0 is equal to given result at the end of accumulation subrelation
        3, // accumulator limb 1 is equal to given result at the end of accumulation subrelation
        3, // accumulator limb 2 is equal to given result at the end of accumulation subrelation
        3  // accumulator limb 3 is equal to given result at the end of accumulation subrelation

    };

    /**
     * @brief Relation enforcing non-arithmetic transitions of accumulator (value that is tracking the batched
     * evaluation of polynomials in non-native field)
     * @details This relation enforces three pieces of logic:
     * 1) Accumulator starts as zero before we start accumulating stuff
     * 2) Accumulator limbs stay the same if accumulation is not occurring (at even indices)
     * 3) Accumulator limbs result in the values specified by relation parameters after accumulation
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulators,
                           const AllEntities& in,
                           const Parameters& params,
                           const FF& scaling_factor);
};

template <typename FF>
using GoblinTranslatorOpcodeConstraintRelation = Relation<GoblinTranslatorOpcodeConstraintRelationImpl<FF>>;

template <typename FF>
using GoblinTranslatorAccumulatorTransferRelation = Relation<GoblinTranslatorAccumulatorTransferRelationImpl<FF>>;

} // namespace bb
