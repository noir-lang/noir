#pragma once
#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/common/zip_view.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include <typeinfo>

namespace bb {

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
 * @param proving_key Can be a proving_key or an AllEntities object
 */
template <typename Flavor> void compute_concatenated_polynomials(typename Flavor::ProverPolynomials& polynomials)
{
    // Concatenation groups are vectors of polynomials that are concatenated together
    auto concatenation_groups = polynomials.get_concatenation_groups();

    // Resulting concatenated polynomials
    auto targets = polynomials.get_concatenated_constraints();

    // Targets have to be full-sized polynomials. We can compute the mini circuit size from them by dividing by
    // concatenation index
    const size_t MINI_CIRCUIT_SIZE = targets[0].size() / Flavor::CONCATENATION_GROUP_SIZE;
    ASSERT(MINI_CIRCUIT_SIZE * Flavor::CONCATENATION_GROUP_SIZE == targets[0].size());
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
            std::advance(starting_write_offset, j * MINI_CIRCUIT_SIZE);
            std::advance(finishing_read_offset, MINI_CIRCUIT_SIZE);
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
 * GoblinTranslatorFlavor's DeltaRangeConstraint relation, which ensures that sequential values differ by not more than
 * 3, the last value is the maximum and the first value is zero (zero at the start allows us not to dance around
 * shifts).
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
 * @param proving_key
 */
template <typename Flavor>
void compute_goblin_translator_range_constraint_ordered_polynomials(typename Flavor::ProverPolynomials& polynomials,
                                                                    size_t mini_circuit_dyadic_size)
{

    using FF = typename Flavor::FF;

    // Get constants
    constexpr auto sort_step = Flavor::SORT_STEP;
    constexpr auto num_concatenated_wires = Flavor::NUM_CONCATENATED_WIRES;
    const auto mini_circuit_size = mini_circuit_dyadic_size;
    const auto full_circuit_size = mini_circuit_dyadic_size * Flavor::CONCATENATION_GROUP_SIZE;

    // The value we have to end polynomials with
    constexpr uint32_t max_value = (1 << Flavor::MICRO_LIMB_BITS) - 1;

    // Number of elements needed to go from 0 to MAX_VALUE with our step
    constexpr size_t sorted_elements_count = (max_value / sort_step) + 1 + (max_value % sort_step == 0 ? 0 : 1);

    // Check if we can construct these polynomials
    ASSERT((num_concatenated_wires + 1) * sorted_elements_count < full_circuit_size);

    // First use integers (easier to sort)
    std::vector<size_t> sorted_elements(sorted_elements_count);

    // Fill with necessary steps
    sorted_elements[0] = max_value;
    for (size_t i = 1; i < sorted_elements_count; i++) {
        sorted_elements[i] = (sorted_elements_count - 1 - i) * sort_step;
    }

    std::vector<std::vector<uint32_t>> ordered_vectors_uint(num_concatenated_wires);
    auto ordered_constraint_polynomials = std::vector{ &polynomials.ordered_range_constraints_0,
                                                       &polynomials.ordered_range_constraints_1,
                                                       &polynomials.ordered_range_constraints_2,
                                                       &polynomials.ordered_range_constraints_3 };
    std::vector<size_t> extra_denominator_uint(full_circuit_size);

    // Get information which polynomials need to be concatenated
    auto concatenation_groups = polynomials.get_concatenation_groups();

    // A function that transfers elements from each of the polynomials in the chosen concatenation group in the uint
    // ordered polynomials
    auto ordering_function = [&](size_t i) {
        // Get the group and the main target vector
        auto my_group = concatenation_groups[i];
        auto& current_vector = ordered_vectors_uint[i];
        current_vector.resize(full_circuit_size);

        // Calculate how much space there is for values from the original polynomials
        auto free_space_before_runway = full_circuit_size - sorted_elements_count;

        // Calculate the offset of this group's overflowing elements in the extra denominator polynomial
        size_t extra_denominator_offset = i * sorted_elements_count;

        // Go through each polynomial in the concatenation group
        for (size_t j = 0; j < Flavor::CONCATENATION_GROUP_SIZE; j++) {

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
                   polynomials.ordered_range_constraints_4.begin(),
                   [](uint32_t in) { return FF(in); });
}

} // namespace bb