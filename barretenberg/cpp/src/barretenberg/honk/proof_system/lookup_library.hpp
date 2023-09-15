#pragma once
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include <typeinfo>

namespace proof_system::honk::lookup_library {

/**
 * @brief Compute the inverse polynomial I(X) required for logderivative lookups
 * *
 * @details
 * Inverse may be defined in terms of its values  on X_i = 0,1,...,n-1 as Z_perm[0] = 1 and for i = 1:n-1
 *                           1                              1
 * Inverse[i] = ∏ -------------------------- * ∏' --------------------------
 *                  relation::read_term(j)         relation::write_term(j)
 *
 * where ∏ := ∏_{j=0:relation::NUM_READ_TERMS-1} and ∏' := ∏'_{j=0:relation::NUM_WRITE_TERMS-1}
 *
 * If row [i] does not contain a lookup read gate or a write gate, Inverse[i] = 0
 * N.B. by "write gate" we mean; do the lookup table polynomials contain nonzero values at this row?
 * (in the ECCVM, the lookup table is not precomputed, so we have a concept of a "write gate", unlike when precomputed
 * lookup tables are used)
 *
 * The specific algebraic relations that define read terms and write terms are defined in Flavor::LookupRelation
 *
 */
template <typename Flavor, typename Relation>
void compute_logderivative_inverse(auto& polynomials,
                                   proof_system::RelationParameters<typename Flavor::FF>& relation_parameters,
                                   const size_t circuit_size)
{
    using FF = typename Flavor::FF;
    using Accumulator = typename Relation::ValueAccumulatorsAndViews;
    constexpr size_t READ_TERMS = Relation::READ_TERMS;
    constexpr size_t WRITE_TERMS = Relation::WRITE_TERMS;
    auto& inverse_polynomial = polynomials.lookup_inverses;

    auto lookup_relation = Relation();
    for (size_t i = 0; i < circuit_size; ++i) {
        bool has_inverse =
            lookup_relation.template lookup_exists_at_row_index<Accumulator>(polynomials, relation_parameters, i);
        if (!has_inverse) {
            continue;
        }
        FF denominator = 1;
        barretenberg::constexpr_for<0, READ_TERMS, 1>([&]<size_t read_index> {
            auto denominator_term = lookup_relation.template compute_read_term<Accumulator, read_index>(
                polynomials, relation_parameters, i);
            denominator *= denominator_term;
        });
        barretenberg::constexpr_for<0, WRITE_TERMS, 1>([&]<size_t write_index> {
            auto denominator_term = lookup_relation.template compute_write_term<Accumulator, write_index>(
                polynomials, relation_parameters, i);
            denominator *= denominator_term;
        });
        inverse_polynomial[i] = denominator;
    };

    // todo might be inverting zero in field bleh bleh
    FF::batch_invert(inverse_polynomial);
}

} // namespace proof_system::honk::lookup_library