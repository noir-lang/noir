#pragma once
#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include <typeinfo>

namespace bb::honk::permutation_library {

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
void compute_permutation_grand_product(const size_t circuit_size,
                                       auto& full_polynomials,
                                       bb::RelationParameters<typename Flavor::FF>& relation_parameters)
{
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;
    using Accumulator = std::tuple_element_t<0, typename GrandProdRelation::SumcheckArrayOfValuesOverSubrelations>;

    // Allocate numerator/denominator polynomials that will serve as scratch space
    // TODO(zac) we can re-use the permutation polynomial as the numerator polynomial.
    // Reduces readability (issue #2215)
    Polynomial numerator = Polynomial{ circuit_size };
    Polynomial denominator = Polynomial{ circuit_size };

    // Step (1)
    // Populate `numerator` and `denominator` with the algebra described by GrandProdRelation
    static constexpr size_t MIN_CIRCUIT_SIZE_TO_MULTITHREAD = 64;
    const size_t num_threads = circuit_size >= MIN_CIRCUIT_SIZE_TO_MULTITHREAD
                                   ? (circuit_size >= get_num_cpus_pow2() ? get_num_cpus_pow2() : 1)
                                   : 1;
    const size_t block_size = circuit_size / num_threads;
    parallel_for(num_threads, [&](size_t thread_idx) {
        const size_t start = thread_idx * block_size;
        const size_t end = (thread_idx + 1) * block_size;
        for (size_t i = start; i < end; ++i) {

            typename Flavor::AllValues evaluations;
            for (auto [eval, poly] : zip_view(evaluations.get_all(), full_polynomials.get_all())) {
                eval = poly.size() > i ? poly[i] : 0;
            }
            numerator[i] = GrandProdRelation::template compute_permutation_numerator<Accumulator>(evaluations,
                                                                                                  relation_parameters);
            denominator[i] = GrandProdRelation::template compute_permutation_denominator<Accumulator>(
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
void compute_permutation_grand_products(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                        typename Flavor::ProverPolynomials& full_polynomials,
                                        bb::RelationParameters<typename Flavor::FF>& relation_parameters)
{
    using GrandProductRelations = typename Flavor::GrandProductRelations;
    using FF = typename Flavor::FF;

    constexpr size_t NUM_RELATIONS = std::tuple_size<GrandProductRelations>{};
    bb::constexpr_for<0, NUM_RELATIONS, 1>([&]<size_t i>() {
        using PermutationRelation = typename std::tuple_element<i, GrandProductRelations>::type;

        // Assign the grand product polynomial to the relevant std::span member of `full_polynomials` (and its shift)
        // For example, for UltraPermutationRelation, this will be `full_polynomials.z_perm`
        // For example, for LookupRelation, this will be `full_polynomials.z_lookup`
        bb::Polynomial<FF>& full_polynomial = PermutationRelation::get_grand_product_polynomial(full_polynomials);
        bb::Polynomial<FF>& key_polynomial = PermutationRelation::get_grand_product_polynomial(*key);
        full_polynomial = key_polynomial.share();

        compute_permutation_grand_product<Flavor, PermutationRelation>(
            key->circuit_size, full_polynomials, relation_parameters);
        bb::Polynomial<FF>& full_polynomial_shift =
            PermutationRelation::get_shifted_grand_product_polynomial(full_polynomials);
        full_polynomial_shift = key_polynomial.shifted();
    });
}

/**
 * @brief Compute new polynomials which are the concatenated versions of other polynomials
 *
 * @details Multilinear PCS allow to provide openings for concatenated polynomials in an easy way by combining
 * commitments. This method creates concatenated version of polynomials we won't need to commit to. Used in Goblin
 * Translator
 *
 * Concatenation in Goblin Translator mean the action of constructing a new Polynomial from existing ones by writing
 * their multilinear representations sequentially. For example, if we have f(x₁,x₂)={0, 1, 0, 1} and
 * g(x₁,x₂)={1, 0, 0, 1} then h(x₁ ,x₂ ,x₃ )=concatenation(f(x₁,x₂),g(x₁,x₂))={0, 1, 0, 1, 1, 0, 0, 1}
 *
 * Since we commit to multilinear polynomials with KZG, which treats evaluations as monomial coefficients, in univariate
 * form h(x)=f(x)+x⁴⋅g(x)Fr
 * @tparam Flavor
 * @tparam StorageHandle
 * @param proving_key Can be a proving_key or an AllEntities object
 */
template <typename Flavor, typename StorageHandle> void compute_concatenated_polynomials(StorageHandle* proving_key)
{
    // Concatenation groups are vectors of polynomials that are concatenated together
    auto concatenation_groups = proving_key->get_concatenation_groups();

    // Resulting concatenated polynomials
    auto targets = proving_key->get_concatenated_constraints();

    // A function that produces 1 concatenated polynomial
    // TODO(#756): This can be rewritten to use more cores. Currently uses at maximum the number of concatenated
    // polynomials (4 in Goblin Translator)
    auto ordering_function = [&](size_t i) {
        auto my_group = concatenation_groups[i];
        auto& current_target = targets[i];

        // For each polynomial in group
        for (size_t j = 0; j < my_group.size(); j++) {
            auto starting_write_offset = current_target.begin();
            auto finishing_read_offset = my_group[j].begin();
            std::advance(starting_write_offset, j * Flavor::MINI_CIRCUIT_SIZE);
            std::advance(finishing_read_offset, Flavor::MINI_CIRCUIT_SIZE);
            // Copy into appropriate position in the concatenated polynomial
            std::copy(my_group[j].begin(), finishing_read_offset, starting_write_offset);
        }
    };
    parallel_for(concatenation_groups.size(), ordering_function);
}

/**
 * @brief Compute denominator polynomials for Goblin Translator's range constraint permutation
 *
 * @details  We need to prove that all the range constraint wires indeed have values within the given range (unless
 * changed ∈  [0 , 2¹⁴ - 1]. To do this, we use several virtual concatenated wires, each of which represents a subset
 * or original wires (concatenated_range_constraints_<i>). We also generate several new polynomials of the same length
 * as concatenated ones. These polynomials have values within range, but they are also constrained by the
 * GoblinTranslator's GenPermSort relation, which ensures that sequential values differ by not more than 3, the last
 * value is the maximum and the first value is zero (zero at the start allows us not to dance around shifts).
 *
 * Ideally, we could simply rearrange the values in concatenated_.._0 ,..., concatenated_.._3 and get denominator
 * polynomials (ordered_constraints), but we could get the worst case scenario: each value in the polynomials is
 * maximum value. What can we do in that case? We still have to add (max_range/3)+1 values  to each of the ordered
 * wires for the sort constraint to hold.  So we also need a and extra denominator to store k ⋅ ( max_range / 3 + 1 )
 * values that couldn't go in + ( max_range / 3 +  1 ) connecting values. To counteract the extra ( k + 1 ) ⋅
 * ⋅ (max_range / 3 + 1 ) values needed for denominator sort constraints we need a polynomial in the numerator. So we
 * can construct a proof when ( k + 1 ) ⋅ ( max_range/ 3 + 1 ) < concatenated size
 *
 * @tparam Flavor
 * @tparam StorageHandle
 * @param proving_key
 */
template <typename Flavor, typename StorageHandle>
void compute_goblin_translator_range_constraint_ordered_polynomials(StorageHandle* proving_key)
{

    using FF = typename Flavor::FF;

    // Get constants
    constexpr auto sort_step = Flavor::SORT_STEP;
    constexpr auto num_concatenated_wires = Flavor::NUM_CONCATENATED_WIRES;
    constexpr auto full_circuit_size = Flavor::FULL_CIRCUIT_SIZE;
    constexpr auto mini_circuit_size = Flavor::MINI_CIRCUIT_SIZE;

    // The value we have to end polynomials with
    constexpr uint32_t max_value = (1 << Flavor::MICRO_LIMB_BITS) - 1;

    // Number of elements needed to go from 0 to MAX_VALUE with our step
    constexpr size_t sorted_elements_count = (max_value / sort_step) + 1 + (max_value % sort_step == 0 ? 0 : 1);

    // Check if we can construct these polynomials
    static_assert((num_concatenated_wires + 1) * sorted_elements_count < full_circuit_size);

    // First use integers (easier to sort)
    std::vector<size_t> sorted_elements(sorted_elements_count);

    // Fill with necessary steps
    sorted_elements[0] = max_value;
    for (size_t i = 1; i < sorted_elements_count; i++) {
        sorted_elements[i] = (sorted_elements_count - 1 - i) * sort_step;
    }

    std::vector<std::vector<uint32_t>> ordered_vectors_uint(num_concatenated_wires);
    auto ordered_constraint_polynomials = std::vector{ &proving_key->ordered_range_constraints_0,
                                                       &proving_key->ordered_range_constraints_1,
                                                       &proving_key->ordered_range_constraints_2,
                                                       &proving_key->ordered_range_constraints_3 };
    std::vector<size_t> extra_denominator_uint(full_circuit_size);

    // Get information which polynomials need to be concatenated
    auto concatenation_groups = proving_key->get_concatenation_groups();

    // A function that transfers elements from each of the polynomials in the chosen concatenation group in the uint
    // ordered polynomials
    auto ordering_function = [&](size_t i) {
        // Get the group and the main target vector
        auto my_group = concatenation_groups[i];
        auto& current_vector = ordered_vectors_uint[i];
        current_vector.resize(Flavor::FULL_CIRCUIT_SIZE);

        // Calculate how much space there is for values from the original polynomials
        auto free_space_before_runway = full_circuit_size - sorted_elements_count;

        // Calculate the offset of this group's overflowing elements in the extra denominator polynomial
        size_t extra_denominator_offset = i * sorted_elements_count;

        // Go through each polynomial in the concatenation group
        for (size_t j = 0; j < Flavor::CONCATENATION_INDEX; j++) {

            // Calculate the offset in the target vector
            auto current_offset = j * mini_circuit_size;
            // For each element in the polynomial
            for (size_t k = 0; k < mini_circuit_size; k++) {

                // Put it it the target polynomial
                if ((current_offset + k) < free_space_before_runway) {
                    current_vector[current_offset + k] = static_cast<uint32_t>(uint256_t(my_group[j][k]).data[0]);

                    // Or in the extra one if there is no space left
                } else {
                    extra_denominator_uint[extra_denominator_offset] =
                        static_cast<uint32_t>(uint256_t(my_group[j][k]).data[0]);
                    extra_denominator_offset++;
                }
            }
        }
        // Copy the steps into the target polynomial
        auto starting_write_offset = current_vector.begin();
        std::advance(starting_write_offset, free_space_before_runway);
        std::copy(sorted_elements.cbegin(), sorted_elements.cend(), starting_write_offset);

        // Sort the polynomial in nondescending order. We sort using vector with size_t elements for 2 reasons:
        // 1. It is faster to sort size_t
        // 2. Comparison operators for finite fields are operating on internal form, so we'd have to convert them from
        // Montgomery
        std::sort(current_vector.begin(), current_vector.end());

        // Copy the values into the actual polynomial
        std::transform(current_vector.cbegin(),
                       current_vector.cend(),
                       (*ordered_constraint_polynomials[i]).begin(),
                       [](uint32_t in) { return FF(in); });
    };

    // Construct the first 4 polynomials
    parallel_for(num_concatenated_wires, ordering_function);
    ordered_vectors_uint.clear();

    auto sorted_element_insertion_offset = extra_denominator_uint.begin();
    std::advance(sorted_element_insertion_offset, num_concatenated_wires * sorted_elements_count);

    // Add steps to the extra denominator polynomial
    std::copy(sorted_elements.cbegin(), sorted_elements.cend(), sorted_element_insertion_offset);

    // Sort it
#ifdef NO_TBB
    std::sort(extra_denominator_uint.begin(), extra_denominator_uint.end());
#else
    std::sort(std::execution::par_unseq, extra_denominator_uint.begin(), extra_denominator.end());
#endif

    // And copy it to the actual polynomial
    std::transform(extra_denominator_uint.cbegin(),
                   extra_denominator_uint.cend(),
                   proving_key->ordered_range_constraints_4.begin(),
                   [](uint32_t in) { return FF(in); });
}

/**
 * @brief Compute the extra numerator for Goblin range constraint argument
 *
 * @details Goblin proves that several polynomials contain only values in a certain range through 2 relations:
 * 1) A grand product which ignores positions of elements (GoblinTranslatorPermutationRelation)
 * 2) A relation enforcing a certain ordering on the elements of the given polynomial
 * (GoblinTranslatorGenPermSortRelation)
 *
 * We take the values from 4 polynomials, and spread them into 5 polynomials + add all the steps from MAX_VALUE to 0. We
 * order these polynomials and use them in the denominator of the grand product, at the same time checking that they go
 * from MAX_VALUE to 0. To counteract the added steps we also generate an extra range constraint numerator, which
 * contains 5 MAX_VALUE, 5 (MAX_VALUE-STEP),... values
 *
 * @param key Proving key where we will save the polynomials
 */
template <typename Flavor> inline void compute_extra_range_constraint_numerator(auto proving_key)
{

    // Get the full goblin circuits size (this is the length of concatenated range constraint polynomials)
    auto full_circuit_size = Flavor::FULL_CIRCUIT_SIZE;
    auto sort_step = Flavor::SORT_STEP;
    auto num_concatenated_wires = Flavor::NUM_CONCATENATED_WIRES;

    auto& extra_range_constraint_numerator = proving_key->ordered_extra_range_constraints_numerator;

    uint32_t MAX_VALUE = (1 << Flavor::MICRO_LIMB_BITS) - 1;

    // Calculate how many elements there are in the sequence MAX_VALUE, MAX_VALUE - 3,...,0
    size_t sorted_elements_count = (MAX_VALUE / sort_step) + 1 + (MAX_VALUE % sort_step == 0 ? 0 : 1);

    // Check that we can fit every element in the polynomial
    ASSERT((num_concatenated_wires + 1) * sorted_elements_count < full_circuit_size);

    std::vector<size_t> sorted_elements(sorted_elements_count);

    // Calculate the sequence in integers
    sorted_elements[0] = MAX_VALUE;
    for (size_t i = 1; i < sorted_elements_count; i++) {
        sorted_elements[i] = (sorted_elements_count - 1 - i) * sort_step;
    }

    // TODO(#756): can be parallelized further. This will use at most 5 threads
    auto fill_with_shift = [&](size_t shift) {
        for (size_t i = 0; i < sorted_elements_count; i++) {
            extra_range_constraint_numerator[shift + i * (num_concatenated_wires + 1)] = sorted_elements[i];
        }
    };
    // Fill polynomials with a sequence, where each element is repeated num_concatenated_wires+1 times
    parallel_for(num_concatenated_wires + 1, fill_with_shift);
}

/**
 * @brief Compute odd and even largrange polynomials (up to mini_circuit length) and put them in the polynomial cache
 *
 * @param key Proving key where we will save the polynomials
 */
template <typename Flavor> inline void compute_lagrange_polynomials_for_goblin_translator(auto proving_key)

{
    const size_t n = proving_key->circuit_size;
    typename Flavor::Polynomial lagrange_polynomial_odd_in_minicircuit(n);
    typename Flavor::Polynomial lagrange_polynomial_even_in_minicircut(n);
    typename Flavor::Polynomial lagrange_polynomial_second(n);
    typename Flavor::Polynomial lagrange_polynomial_second_to_last_in_minicircuit(n);

    for (size_t i = 1; i < Flavor::MINI_CIRCUIT_SIZE - 1; i += 2) {
        lagrange_polynomial_odd_in_minicircuit[i] = 1;
        lagrange_polynomial_even_in_minicircut[i + 1] = 1;
    }
    proving_key->lagrange_odd_in_minicircuit = lagrange_polynomial_odd_in_minicircuit.share();

    proving_key->lagrange_even_in_minicircuit = lagrange_polynomial_even_in_minicircut.share();
    lagrange_polynomial_second[1] = 1;
    lagrange_polynomial_second_to_last_in_minicircuit[Flavor::MINI_CIRCUIT_SIZE - 2] = 1;
    proving_key->lagrange_second_to_last_in_minicircuit = lagrange_polynomial_second_to_last_in_minicircuit.share();
    proving_key->lagrange_second = lagrange_polynomial_second.share();
}

} // namespace bb::honk::permutation_library