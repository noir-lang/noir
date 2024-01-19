#include "barretenberg/relations/translator_vm/translator_extra_relations.hpp"
#include "barretenberg/flavor/goblin_translator.hpp"

namespace bb {

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
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void GoblinTranslatorOpcodeConstraintRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulators,
                                                                  const AllEntities& in,
                                                                  const Parameters&,
                                                                  const FF& scaling_factor)
{

    using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    auto op = View(in.op);
    static const FF minus_one = FF(-1);
    static const FF minus_two = FF(-2);
    static const FF minus_three = FF(-3);
    static const FF minus_four = FF(-4);
    static const FF minus_eight = FF(-8);

    // Contribution (1) (op(op-1)(op-2)(op-3)(op-4)(op-8))
    auto tmp_1 = op * (op + minus_one);
    tmp_1 *= (op + minus_two);
    tmp_1 *= (op + minus_three);
    tmp_1 *= (op + minus_four);
    tmp_1 *= (op + minus_eight);
    tmp_1 *= scaling_factor;
    std::get<0>(accumulators) += tmp_1;
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
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void GoblinTranslatorAccumulatorTransferRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulators,
                                                                     const AllEntities& in,
                                                                     const Parameters& params,
                                                                     const FF& scaling_factor)
{
    using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;
    // We use combination of lagrange polynomials at even indices in the minicircuit for copying the accumulator
    auto lagrange_even_in_minicircuit = View(in.lagrange_even_in_minicircuit);

    // Lagrange at index 1 is used to confirm the accumulator result
    auto lagrange_second = View(in.lagrange_second);

    // Lagrange at index (size of minicircuit - 2) is used to enforce that it starts with zero
    auto lagrange_second_to_last_in_minicircuit = View(in.lagrange_second_to_last_in_minicircuit);

    auto accumulators_binary_limbs_0 = View(in.accumulators_binary_limbs_0);
    auto accumulators_binary_limbs_1 = View(in.accumulators_binary_limbs_1);
    auto accumulators_binary_limbs_2 = View(in.accumulators_binary_limbs_2);
    auto accumulators_binary_limbs_3 = View(in.accumulators_binary_limbs_3);
    auto accumulators_binary_limbs_0_shift = View(in.accumulators_binary_limbs_0_shift);
    auto accumulators_binary_limbs_1_shift = View(in.accumulators_binary_limbs_1_shift);
    auto accumulators_binary_limbs_2_shift = View(in.accumulators_binary_limbs_2_shift);
    auto accumulators_binary_limbs_3_shift = View(in.accumulators_binary_limbs_3_shift);

    // Contribution (1) (1-4 ensure transfer of accumulator limbs at even indices of the minicircuit)
    auto tmp_1 = accumulators_binary_limbs_0 - accumulators_binary_limbs_0_shift;
    tmp_1 *= lagrange_even_in_minicircuit;
    tmp_1 *= scaling_factor;
    std::get<0>(accumulators) += tmp_1;

    // Contribution (2)
    auto tmp_2 = accumulators_binary_limbs_1 - accumulators_binary_limbs_1_shift;
    tmp_2 *= lagrange_even_in_minicircuit;
    tmp_2 *= scaling_factor;
    std::get<1>(accumulators) += tmp_2;
    // Contribution (3)
    auto tmp_3 = accumulators_binary_limbs_2 - accumulators_binary_limbs_2_shift;
    tmp_3 *= lagrange_even_in_minicircuit;
    tmp_3 *= scaling_factor;
    std::get<2>(accumulators) += tmp_3;
    // Contribution (4)
    auto tmp_4 = accumulators_binary_limbs_3 - accumulators_binary_limbs_3_shift;
    tmp_4 *= lagrange_even_in_minicircuit;
    tmp_4 *= scaling_factor;
    std::get<3>(accumulators) += tmp_4;

    // Contribution (5) (5-9 ensure that accumulator starts with zeroed-out limbs)
    auto tmp_5 = accumulators_binary_limbs_0 * lagrange_second_to_last_in_minicircuit;
    tmp_5 *= scaling_factor;
    std::get<4>(accumulators) += tmp_5;

    // Contribution (6)
    auto tmp_6 = accumulators_binary_limbs_1 * lagrange_second_to_last_in_minicircuit;
    tmp_6 *= scaling_factor;
    std::get<5>(accumulators) += tmp_6;

    // Contribution (7)
    auto tmp_7 = accumulators_binary_limbs_2 * lagrange_second_to_last_in_minicircuit;
    tmp_7 *= scaling_factor;
    std::get<6>(accumulators) += tmp_7;

    // Contribution (8)
    auto tmp_8 = accumulators_binary_limbs_3 * lagrange_second_to_last_in_minicircuit;
    tmp_8 *= scaling_factor;
    std::get<7>(accumulators) += tmp_8;

    // Contribution (9) (9-12 ensure the output is as stated, we basically use this to get the result out of the
    // proof)
    auto tmp_9 = (accumulators_binary_limbs_0 - params.accumulated_result[0]) * lagrange_second;
    tmp_9 *= scaling_factor;
    std::get<8>(accumulators) += tmp_9;

    // Contribution (10)
    auto tmp_10 = (accumulators_binary_limbs_1 - params.accumulated_result[1]) * lagrange_second;
    tmp_10 *= scaling_factor;
    std::get<9>(accumulators) += tmp_10;

    // Contribution (11)
    auto tmp_11 = (accumulators_binary_limbs_2 - params.accumulated_result[2]) * lagrange_second;
    tmp_11 *= scaling_factor;
    std::get<10>(accumulators) += tmp_11;

    // Contribution (12)
    auto tmp_12 = (accumulators_binary_limbs_3 - params.accumulated_result[3]) * lagrange_second;
    tmp_12 *= scaling_factor;
    std::get<11>(accumulators) += tmp_12;
};

template class GoblinTranslatorOpcodeConstraintRelationImpl<bb::fr>;
template class GoblinTranslatorAccumulatorTransferRelationImpl<bb::fr>;

DEFINE_SUMCHECK_RELATION_CLASS(GoblinTranslatorOpcodeConstraintRelationImpl, honk::flavor::GoblinTranslator);
DEFINE_SUMCHECK_RELATION_CLASS(GoblinTranslatorAccumulatorTransferRelationImpl, honk::flavor::GoblinTranslator);

} // namespace bb
