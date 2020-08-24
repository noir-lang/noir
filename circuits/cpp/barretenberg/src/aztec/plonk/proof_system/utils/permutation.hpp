#pragma once
#include <numeric/bitop/get_msb.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <polynomials/polynomial.hpp>

namespace waffle {

struct permutation_subgroup_element {
    uint32_t subgroup_index = 0;
    uint8_t column_index = 0;
    bool is_public_input = false;
    bool is_tag = false;
};

/**
 * TODO: Remove need for this. We have a lot of old tests that use a vector of uint32s
 * instead of permutation_subgroup_element
 **/
template <typename program_settings>
inline void compute_permutation_lagrange_base_single(barretenberg::polynomial& output,
                                                     const std::vector<uint32_t>& permutation,
                                                     const barretenberg::evaluation_domain& small_domain)
{
    std::vector<permutation_subgroup_element> subgroup_elements;
    for (const auto& permutation_element : permutation) {
        uint32_t index = permutation_element & (uint32_t)0xffffffU;
        uint32_t column = permutation_element >> 30U;
        subgroup_elements.emplace_back(permutation_subgroup_element{ index, (uint8_t)column, false, false });
    }
    compute_permutation_lagrange_base_single<program_settings>(output, subgroup_elements, small_domain);
}

template <typename program_settings>
inline void compute_permutation_lagrange_base_single(barretenberg::polynomial& output,
                                                     const std::vector<permutation_subgroup_element>& permutation,
                                                     const barretenberg::evaluation_domain& small_domain)
{
    if (output.get_size() < permutation.size()) {
        output.resize_unsafe(permutation.size());
    }
    // permutation encoding:
    // low 28 bits defines the location in witness polynomial
    // upper 2 bits defines the witness polynomial:
    // 0 = left
    // 1 = right
    // 2 = output
    ASSERT(small_domain.log2_size > 1);
    const barretenberg::fr* roots = small_domain.get_round_roots()[small_domain.log2_size - 2];
    const size_t root_size = small_domain.size >> 1UL;
    const size_t log2_root_size = static_cast<size_t>(numeric::get_msb(root_size));

    ITERATE_OVER_DOMAIN_START(small_domain);
    // permutation[i] will specify the 'index' that this wire value will map to
    // here, 'index' refers to an element of our subgroup H
    // we can almost use permutation[i] to directly index our `roots` array, which contains our subgroup elements
    // we first have to mask off the 2 high bits, which describe which wire polynomial our permutation maps to (we'll
    // deal with that in a bit) we then have to accomodate for the fact that, `roots` only contains *half* of our
    // subgroup elements. this is because w^{n/2} = -w and we don't want to perform redundant work computing roots of
    // unity

    // Step 1: mask the high bits and get the permutation index
    size_t raw_idx = permutation[i].subgroup_index;
    bool is_public_input = permutation[i].is_public_input;
    bool is_tag = permutation[i].is_tag;

    // Step 2: is `raw_idx` >= (n / 2)? if so, we will need to index `-roots[raw_idx - subgroup_size / 2]` instead
    // of `roots[raw_idx]`
    const bool negative_idx = raw_idx >= root_size;

    // Step 3: compute the index of the subgroup element we'll be accessing.
    // To avoid a conditional branch, we can subtract `negative_idx << log2_root_size` from `raw_idx`
    // here, `log2_root_size = numeric::get_msb(subgroup_size / 2)` (we know our subgroup size will be a power of 2,
    // so we lose no precision here)
    const size_t idx = raw_idx - (static_cast<size_t>(negative_idx) << log2_root_size);

    // call `conditionally_subtract_double_modulus`, using `negative_idx` as our predicate.
    // Our roots of unity table is partially 'overloaded' - we either store the root `w`, or `modulus + w`
    // So to ensure we correctly compute `modulus - w`, we need to compute `2 * modulus - w`
    // The output will similarly be overloaded (containing either 2 * modulus - w, or modulus - w)
    output[i] = roots[idx].conditionally_subtract_from_double_modulus(static_cast<uint64_t>(negative_idx));

    // finally, if our permutation maps to an index in either the right wire vector, or the output wire vector, we
    // need to multiply our result by one of two quadratic non-residues. (this ensure that mapping into the left
    // wires gives unique values that are not repeated in the right or output wire permutations) (ditto for right
    // wire and output wire mappings)

    if (is_public_input) {
        output[i] *= barretenberg::fr::external_coset_generator();
    } else if (is_tag) {
        output[i] *= barretenberg::fr::tag_coset_generator();
    } else {
        {
            // isolate the highest 2 bits of `permutation[i]` and shunt them down into the 2 least significant bits
            const uint32_t column_index = permutation[i].column_index;
            // ((permutation[i] & program_settings::permutation_mask) >> program_settings::permutation_shift);
            if (column_index > 0) {
                output[i] *= barretenberg::fr::coset_generator(column_index - 1);
            }
        }
    }
    ITERATE_OVER_DOMAIN_END;
}
} // namespace waffle
