#include "barretenberg/honk/flavor/ecc_vm.hpp"
#include "barretenberg/honk/sumcheck/relation_definitions_fwd.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"
#include "ecc_msm_relation.hpp"

namespace proof_system::honk::sumcheck {

/**
 * @brief Expression for ECCVM lookup tables.
 * @details We use log-derivative lookup tables for the following case:
 * Table writes: ECCVMPointTable columns: we define Straus point table:
 * { {0, -15[P]}, {1, -13[P]}, ..., {15, 15[P]} }
 * write source: { precompute_round, precompute_tx, precompute_ty }
 * Table reads: ECCVMMSM columns. Each row adds up to 4 points into MSM accumulator
 * read source: { msm_slice1, msm_x1, msm_y1 }, ..., { msm_slice4, msm_x4, msm_y4 }
 * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
 * @param extended_edges an std::array containing the fully extended Accumulator edges.
 * @param parameters contains beta, gamma, and public_input_delta, ....
 * @param scaling_factor optional term to scale the evaluation before adding to evals.
 */
template <typename FF>
template <typename AccumulatorTypes>
void ECCVMLookupRelationBase<FF>::accumulate(typename AccumulatorTypes::Accumulators& accumulator,
                                             const auto& extended_edges,
                                             const RelationParameters<FF>& relation_params,
                                             const FF& scaling_factor)
{
    using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
    using Accumulator = typename std::tuple_element<0, typename AccumulatorTypes::Accumulators>::type;

    auto lookup_inverses = View(extended_edges.lookup_inverses);

    constexpr size_t NUM_TOTAL_TERMS = READ_TERMS + WRITE_TERMS;
    std::array<Accumulator, NUM_TOTAL_TERMS> lookup_terms;
    std::array<Accumulator, NUM_TOTAL_TERMS> denominator_accumulator;

    // The lookup relation = \sum_j (1 / read_term[j]) - \sum_k (read_counts[k] / write_term[k])
    // To get the inverses (1 / read_term[i]), (1 / write_term[i]), we have a commitment to the product of all inverses
    // i.e. lookup_inverse = \prod_j (1 / read_term[j]) * \prod_k (1 / write_term[k])
    // The purpose of this next section is to derive individual inverse terms using `lookup_inverses`
    // i.e. (1 / read_term[i]) = lookup_inverse * \prod_{j /ne i} (read_term[j]) * \prod_k (write_term[k])
    //      (1 / write_term[i]) = lookup_inverse * \prod_j (read_term[j]) * \prod_{k ne i} (write_term[k])
    barretenberg::constexpr_for<0, READ_TERMS, 1>([&]<size_t i>() {
        lookup_terms[i] = compute_read_term<AccumulatorTypes, i>(extended_edges, relation_params, 0);
    });
    barretenberg::constexpr_for<0, WRITE_TERMS, 1>([&]<size_t i>() {
        lookup_terms[i + READ_TERMS] = compute_write_term<AccumulatorTypes, i>(extended_edges, relation_params, 0);
    });

    barretenberg::constexpr_for<0, NUM_TOTAL_TERMS, 1>(
        [&]<size_t i>() { denominator_accumulator[i] = lookup_terms[i]; });

    barretenberg::constexpr_for<0, NUM_TOTAL_TERMS - 1, 1>(
        [&]<size_t i>() { denominator_accumulator[i + 1] *= denominator_accumulator[i]; });

    auto inverse_accumulator = Accumulator(lookup_inverses); // denominator_accumulator[NUM_TOTAL_TERMS - 1];

    const auto row_has_write = View(extended_edges.precompute_select);
    const auto row_has_read = View(extended_edges.msm_add) + View(extended_edges.msm_skew);
    const auto inverse_exists = row_has_write + row_has_read - (row_has_write * row_has_read);

    std::get<0>(accumulator) +=
        (denominator_accumulator[NUM_TOTAL_TERMS - 1] * lookup_inverses - inverse_exists) * scaling_factor;

    // After this algo, total degree of denominator_accumulator = NUM_TOTAL_TERMA
    for (size_t i = 0; i < NUM_TOTAL_TERMS - 1; ++i) {
        denominator_accumulator[NUM_TOTAL_TERMS - 1 - i] =
            denominator_accumulator[NUM_TOTAL_TERMS - 2 - i] * inverse_accumulator;
        inverse_accumulator = inverse_accumulator * lookup_terms[NUM_TOTAL_TERMS - 1 - i];
    }
    denominator_accumulator[0] = inverse_accumulator;

    // each predicate is degree-1
    // degree of relation at this point = NUM_TOTAL_TERMS + 1
    barretenberg::constexpr_for<0, READ_TERMS, 1>([&]<size_t i>() {
        std::get<1>(accumulator) +=
            compute_read_term_predicate<AccumulatorTypes, i>(extended_edges, relation_params, 0) *
            denominator_accumulator[i];
    });

    // each predicate is degree-1, `lookup_read_counts` is degree-1
    // degree of relation = NUM_TOTAL_TERMS + 2 = 6 + 2
    barretenberg::constexpr_for<0, WRITE_TERMS, 1>([&]<size_t i>() {
        const auto p = compute_write_term_predicate<AccumulatorTypes, i>(extended_edges, relation_params, 0);
        const auto lookup_read_count = View(extended_edges.template lookup_read_counts<i>());
        std::get<1>(accumulator) -= p * (denominator_accumulator[i + READ_TERMS] * lookup_read_count);
    });
}
template class ECCVMLookupRelationBase<barretenberg::fr>;
template class ECCVMLookupRelationBase<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationBase, flavor::ECCVM);
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationBase, flavor::ECCVMGrumpkin);

} // namespace proof_system::honk::sumcheck
