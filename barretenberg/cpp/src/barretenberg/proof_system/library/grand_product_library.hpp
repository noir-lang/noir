#pragma once
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/common/thread.hpp"
#include "barretenberg/common/zip_view.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include <typeinfo>

namespace bb {

// TODO(luke): This contains utilities for grand product computation and is not specific to the permutation grand
// product. Update comments accordingly.
/**
 * @brief Compute a permutation grand product polynomial Z_perm(X)
 * *
 * @details
 * Z_perm may be defined in terms of its values  on X_i = 0,1,...,n-1 as Z_perm[0] = 1 and for i = 1:n-1
 *                  relation::numerator(j)
 * Z_perm[i] = ∏ --------------------------------------------------------------------------------
 *                  relation::denominator(j)
 *
 * where ∏ := ∏_{j=0:i-1}
 *
 * The specific algebraic relation used by Z_perm is defined by Flavor::GrandProductRelations
 *
 * For example, in Flavor::Standard the relation describes:
 *
 *                  (w_1(j) + β⋅id_1(j) + γ) ⋅ (w_2(j) + β⋅id_2(j) + γ) ⋅ (w_3(j) + β⋅id_3(j) + γ)
 * Z_perm[i] = ∏ --------------------------------------------------------------------------------
 *                  (w_1(j) + β⋅σ_1(j) + γ) ⋅ (w_2(j) + β⋅σ_2(j) + γ) ⋅ (w_3(j) + β⋅σ_3(j) + γ)
 * where ∏ := ∏_{j=0:i-1} and id_i(X) = id(X) + n*(i-1)
 *
 * For Flavor::Ultra both the UltraPermutation and Lookup grand products are computed by this method.
 *
 * The grand product is constructed over the course of three steps.
 *
 * For expositional simplicity, write Z_perm[i] as
 *
 *                A(j)
 * Z_perm[i] = ∏ --------------------------
 *                B(h)
 *
 * Step 1) Compute 2 length-n polynomials A, B
 * Step 2) Compute 2 length-n polynomials numerator = ∏ A(j), nenominator = ∏ B(j)
 * Step 3) Compute Z_perm[i + 1] = numerator[i] / denominator[i] (recall: Z_perm[0] = 1)
 *
 * Note: Step (3) utilizes Montgomery batch inversion to replace n-many inversions with
 */
template <typename Flavor, typename GrandProdRelation>
void compute_grand_product(const size_t circuit_size,
                           typename Flavor::ProverPolynomials& full_polynomials,
                           bb::RelationParameters<typename Flavor::FF>& relation_parameters)
{
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;
    using Accumulator = std::tuple_element_t<0, typename GrandProdRelation::SumcheckArrayOfValuesOverSubrelations>;

    // Allocate numerator/denominator polynomials that will serve as scratch space
    // TODO(zac) we can re-use the permutation polynomial as the numerator polynomial. Reduces readability
    Polynomial numerator{ circuit_size };
    Polynomial denominator{ circuit_size };

    // Step (1)
    // Populate `numerator` and `denominator` with the algebra described by Relation
    const size_t num_threads = circuit_size >= get_num_cpus_pow2() ? get_num_cpus_pow2() : 1;
    const size_t block_size = circuit_size / num_threads;
    auto full_polynomials_view = full_polynomials.get_all();
    parallel_for(num_threads, [&](size_t thread_idx) {
        const size_t start = thread_idx * block_size;
        const size_t end = (thread_idx + 1) * block_size;
        typename Flavor::AllValues evaluations;
        auto evaluations_view = evaluations.get_all();
        for (size_t i = start; i < end; ++i) {
            for (auto [eval, full_poly] : zip_view(evaluations_view, full_polynomials_view)) {
                eval = full_poly.size() > i ? full_poly[i] : 0;
            }
            numerator[i] = GrandProdRelation::template compute_grand_product_numerator<Accumulator>(
                evaluations, relation_parameters);
            denominator[i] = GrandProdRelation::template compute_grand_product_denominator<Accumulator>(
                evaluations, relation_parameters);
        }
    });

    // Step (2)
    // Compute the accumulating product of the numerator and denominator terms.
    // This step is split into three parts for efficient multithreading:
    // (i) compute ∏ A(j), ∏ B(j) subproducts for each thread
    // (ii) compute scaling factor required to convert each subproduct into a single running product
    // (ii) combine subproducts into a single running product
    //
    // For example, consider 4 threads and a size-8 numerator { a0, a1, a2, a3, a4, a5, a6, a7 }
    // (i)   Each thread computes 1 element of N = {{ a0, a0a1 }, { a2, a2a3 }, { a4, a4a5 }, { a6, a6a7 }}
    // (ii)  Take partial products P = { 1, a0a1, a2a3, a4a5 }
    // (iii) Each thread j computes N[i][j]*P[j]=
    //      {{a0,a0a1},{a0a1a2,a0a1a2a3},{a0a1a2a3a4,a0a1a2a3a4a5},{a0a1a2a3a4a5a6,a0a1a2a3a4a5a6a7}}
    std::vector<FF> partial_numerators(num_threads);
    std::vector<FF> partial_denominators(num_threads);

    parallel_for(num_threads, [&](size_t thread_idx) {
        const size_t start = thread_idx * block_size;
        const size_t end = (thread_idx + 1) * block_size;
        for (size_t i = start; i < end - 1; ++i) {
            numerator[i + 1] *= numerator[i];
            denominator[i + 1] *= denominator[i];
        }
        partial_numerators[thread_idx] = numerator[end - 1];
        partial_denominators[thread_idx] = denominator[end - 1];
    });

    parallel_for(num_threads, [&](size_t thread_idx) {
        const size_t start = thread_idx * block_size;
        const size_t end = (thread_idx + 1) * block_size;
        if (thread_idx > 0) {
            FF numerator_scaling = 1;
            FF denominator_scaling = 1;

            for (size_t j = 0; j < thread_idx; ++j) {
                numerator_scaling *= partial_numerators[j];
                denominator_scaling *= partial_denominators[j];
            }
            for (size_t i = start; i < end; ++i) {
                numerator[i] *= numerator_scaling;
                denominator[i] *= denominator_scaling;
            }
        }

        // Final step: invert denominator
        FF::batch_invert(std::span{ &denominator[start], block_size });
    });

    // Step (3) Compute z_perm[i] = numerator[i] / denominator[i]
    auto& grand_product_polynomial = GrandProdRelation::get_grand_product_polynomial(full_polynomials);
    grand_product_polynomial[0] = 0;
    parallel_for(num_threads, [&](size_t thread_idx) {
        const size_t start = thread_idx * block_size;
        const size_t end = (thread_idx == num_threads - 1) ? circuit_size - 1 : (thread_idx + 1) * block_size;
        for (size_t i = start; i < end; ++i) {
            grand_product_polynomial[i + 1] = numerator[i] * denominator[i];
        }
    });
}

template <typename Flavor>
void compute_grand_products(std::shared_ptr<typename Flavor::ProvingKey>& key,
                            typename Flavor::ProverPolynomials& full_polynomials,
                            bb::RelationParameters<typename Flavor::FF>& relation_parameters)
{
    using GrandProductRelations = typename Flavor::GrandProductRelations;
    using FF = typename Flavor::FF;

    constexpr size_t NUM_RELATIONS = std::tuple_size<GrandProductRelations>{};
    bb::constexpr_for<0, NUM_RELATIONS, 1>([&]<size_t i>() {
        using GrandProdRelation = typename std::tuple_element<i, GrandProductRelations>::type;

        // Assign the grand product polynomial to the relevant std::span member of `full_polynomials` (and its shift)
        // For example, for UltraPermutationRelation, this will be `full_polynomials.z_perm`
        // For example, for LookupRelation, this will be `full_polynomials.z_lookup`
        bb::Polynomial<FF>& full_polynomial = GrandProdRelation::get_grand_product_polynomial(full_polynomials);
        auto& key_polynomial = GrandProdRelation::get_grand_product_polynomial(*key);
        full_polynomial = key_polynomial.share();

        compute_grand_product<Flavor, GrandProdRelation>(key->circuit_size, full_polynomials, relation_parameters);
        bb::Polynomial<FF>& full_polynomial_shift =
            GrandProdRelation::get_shifted_grand_product_polynomial(full_polynomials);
        full_polynomial_shift = key_polynomial.shifted();
    });
}

} // namespace bb