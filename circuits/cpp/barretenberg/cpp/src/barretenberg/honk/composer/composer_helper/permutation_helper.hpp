/**
 * @file permutation_helper.hpp
 * @brief Contains various functions that help construct Honk and Plonk Sigma and Id polynomials
 *
 * @details It is structured to reuse similar components in Honk and Plonk
 *
 */

#pragma once

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/proving_key/proving_key.hpp"

#include <algorithm>
#include <cstdint>
#include <initializer_list>
#include <cstdint>
#include <cstddef>
#include <utility>
#include <vector>

namespace bonk {

/**
 * @brief cycle_node represents the index of a value of the circuit.
 * It will belong to a CyclicPermutation, such that all nodes in a CyclicPermutation
 * must have the value.
 * The total number of constraints is always <2^32 since that is the type used to represent variables, so we can save
 * space by using a type smaller than size_t.
 */
struct cycle_node {
    uint32_t wire_index;
    uint32_t gate_index;
};

/**
 * @brief Permutations subgroup element structure is used to hold data necessary to construct permutation polynomials.
 *
 * @details All parameters define the evaluation of an id or sigma polynomial.
 *
 */
struct permutation_subgroup_element {
    uint32_t row_index = 0;
    uint8_t column_index = 0;
    bool is_public_input = false;
    bool is_tag = false;
};
using CyclicPermutation = std::vector<cycle_node>;

namespace {
/**
 * Compute all CyclicPermutations of the circuit. Each CyclicPermutation represents the indices of the values in the
 * witness wires that must have the same value.
 *
 * @tparam program_width Program width
 * */
template <size_t program_width, typename CircuitConstructor>
std::vector<CyclicPermutation> compute_wire_copy_cycles(const CircuitConstructor& circuit_constructor)
{
    // Reference circuit constructor members
    const size_t num_gates = circuit_constructor.num_gates;
    std::span<const uint32_t> public_inputs = circuit_constructor.public_inputs;
    const size_t num_public_inputs = public_inputs.size();

    // Get references to the wires containing the index of the value inside constructor.variables
    // These wires only contain the "real" gate constraints, and are not padded.
    std::array<std::span<const uint32_t>, program_width> wire_indices;
    wire_indices[0] = circuit_constructor.w_l;
    wire_indices[1] = circuit_constructor.w_r;
    wire_indices[2] = circuit_constructor.w_o;
    if constexpr (program_width > 3) {
        wire_indices[3] = circuit_constructor.w_4;
    }

    // Each variable represents one cycle
    const size_t number_of_cycles = circuit_constructor.variables.size();
    std::vector<CyclicPermutation> copy_cycles(number_of_cycles);

    // Represents the index of a variable in circuit_constructor.variables
    std::span<const uint32_t> real_variable_index = circuit_constructor.real_variable_index;

    // We use the permutation argument to enforce the public input variables to be equal to values provided by the
    // verifier. The convension we use is to place the public input values as the first rows of witness vectors.
    // More specifically, we set the LEFT and RIGHT wires to be the public inputs and set the other elements of the row
    // to 0. All selectors are zero at these rows, so they are fully unconstrained. The "real" gates that follow can use
    // references to these variables.
    //
    // The copy cycle for the i-th public variable looks like
    //   (i) -> (n+i) -> (i') -> ... -> (i'')
    // (Using the convention that W^L_i = W_i and W^R_i = W_{n+i}, W^O_i = W_{2n+i})
    //
    // This loop initializes the i-th cycle with (i) -> (n+i), meaning that we always expect W^L_i = W^R_i,
    // for all i s.t. row i defines a public input.
    for (size_t i = 0; i < num_public_inputs; ++i) {
        const uint32_t public_input_index = real_variable_index[public_inputs[i]];
        const auto gate_index = static_cast<uint32_t>(i);
        // These two nodes must be in adjacent locations in the cycle for correct handling of public inputs
        copy_cycles[public_input_index].emplace_back(cycle_node{ 0, gate_index });
        copy_cycles[public_input_index].emplace_back(cycle_node{ 1, gate_index });
    }

    // Iterate over all variables of the "real" gates, and add a corresponding node to the cycle for that variable
    for (size_t j = 0; j < program_width; ++j) {
        for (size_t i = 0; i < num_gates; ++i) {
            // We are looking at the j-th wire in the i-th row.
            // The value in this position should be equal to the value of the element at index `var_index`
            // of the `constructor.variables` vector.
            // Therefore, we add (i,j) to the cycle at index `var_index` to indicate that w^j_i should have the values
            // constructor.variables[var_index].
            const uint32_t var_index = circuit_constructor.real_variable_index[wire_indices[j][i]];
            const auto wire_index = static_cast<uint32_t>(j);
            const auto gate_index = static_cast<uint32_t>(i + num_public_inputs);
            copy_cycles[var_index].emplace_back(cycle_node{ wire_index, gate_index });
        }
    }
    return copy_cycles;
}

/**
 * @brief Compute the permutation mapping for the basic no-tags case
 *
 * @details This function computes the permutation information in a commonf format that can then be used to generate
 * either Plonk-style FFT-ready sigma polynomials or Honk-style indexed vectors
 *
 * @tparam program_width The number of wires
 * @tparam CircuitConstructor The class that holds basic circuitl ogic
 * @param circuit_constructor Circuit-containing object
 * @param key Pointer to the proving key
 */
template <size_t program_width, typename CircuitConstructor>
std::array<std::vector<permutation_subgroup_element>, program_width> compute_basic_bonk_sigma_permutations(
    const CircuitConstructor& circuit_constructor, bonk::proving_key* key)
{
    // Compute wire copy cycles (cycles of permutations)
    auto wire_copy_cycles = compute_wire_copy_cycles<program_width>(circuit_constructor);

    // Initialize the table of permutations so that every element points to itself
    std::array<std::vector<permutation_subgroup_element>, program_width> sigma_mappings;
    for (size_t i = 0; i < program_width; ++i) {
        sigma_mappings[i].reserve(key->circuit_size);

        for (size_t j = 0; j < key->circuit_size; ++j) {
            sigma_mappings[i].emplace_back(permutation_subgroup_element{
                .row_index = (uint32_t)j, .column_index = (uint8_t)i, .is_public_input = false, .is_tag = false });
        }
    }

    // Go through each cycle
    for (size_t i = 0; i < wire_copy_cycles.size(); ++i) {
        for (size_t j = 0; j < wire_copy_cycles[i].size(); ++j) {
            // Get the indices of the current node and next node int he cycle
            cycle_node current_cycle_node = wire_copy_cycles[i][j];
            // If current node is the last one in the cycle, then the next one is the first one
            size_t next_cycle_node_index = j == wire_copy_cycles[i].size() - 1 ? 0 : j + 1;
            cycle_node next_cycle_node = wire_copy_cycles[i][next_cycle_node_index];
            const auto current_row = current_cycle_node.gate_index;
            const auto next_row = next_cycle_node.gate_index;

            const uint32_t current_column = current_cycle_node.wire_index;
            const uint32_t next_column = next_cycle_node.wire_index;
            // Point current node to the next node
            sigma_mappings[current_column][current_row] = {
                .row_index = next_row, .column_index = (uint8_t)next_column, .is_public_input = false, .is_tag = false
            };
        }
    }

    // Add informationa about public inputs to the computation
    const uint32_t num_public_inputs = static_cast<uint32_t>(circuit_constructor.public_inputs.size());

    for (size_t i = 0; i < num_public_inputs; ++i) {
        sigma_mappings[0][i].row_index = static_cast<uint32_t>(i);
        sigma_mappings[0][i].column_index = 0;
        sigma_mappings[0][i].is_public_input = true;
    }
    return sigma_mappings;
}

/**
 * @brief Compute Sigma polynomials for Honk from a mapping and put into polynomial cache
 *
 * @details Given a mapping (effectively at table pointing witnesses to other witnesses) compute Sigma polynomials in
 * lagrange form and put them into the cache. This version distinguishes betweenr regular elements and public inputs,
 * but ignores tags
 *
 * @tparam program_width The number of wires
 * @param sigma_mappings A table with information about permuting each element
 * @param key Pointer to the proving key
 */
template <size_t program_width>
void compute_honk_style_sigma_lagrange_polynomials_from_mapping(
    std::array<std::vector<permutation_subgroup_element>, program_width>& sigma_mappings, bonk::proving_key* key)
{
    const size_t num_gates = key->circuit_size;

    std::array<barretenberg::polynomial, program_width> sigma;

    for (size_t wire_index = 0; wire_index < program_width; wire_index++) {
        sigma[wire_index] = barretenberg::polynomial(num_gates);
        auto& current_sigma_polynomial = sigma[wire_index];
        ITERATE_OVER_DOMAIN_START(key->small_domain)
        const auto& current_mapping = sigma_mappings[wire_index][i];
        if (current_mapping.is_public_input) {
            // We intentionally want to break the cycles of the public input variables.
            // During the witness generation, the left and right wire polynomials at index i contain the i-th public
            // input. The CyclicPermutation created for these variables always start with (i) -> (n+i), followed by
            // the indices of the variables in the "real" gates. We make i point to -(i+1), so that the only way of
            // repairing the cycle is add the mapping
            //  -(i+1) -> (n+i)
            // These indices are chosen so they can easily be computed by the verifier. They can expect the running
            // product to be equal to the "public input delta" that is computed in <honk/utils/public_inputs.hpp>
            current_sigma_polynomial[i] =
                -barretenberg::fr(current_mapping.row_index + 1 + num_gates * current_mapping.column_index);
        } else {
            ASSERT(!current_mapping.is_tag);
            // For the regular permutation we simply point to the next location by setting the evaluation to its
            // index
            current_sigma_polynomial[i] =
                barretenberg::fr(current_mapping.row_index + num_gates * current_mapping.column_index);
        }
        ITERATE_OVER_DOMAIN_END;
    }
    // Save to polynomial cache
    for (size_t j = 0; j < program_width; j++) {
        std::string index = std::to_string(j + 1);
        key->polynomial_store.put("sigma_" + index + "_lagrange", std::move(sigma[j]));
    }

} // namespace honk

/**
 * Compute sigma permutation polynomial in lagrange base
 *
 * @param output Output polynomial.
 * @param permuataion Input permutation.
 * @param small_domain The domain we base our polynomial in.
 *
 * */
inline void compute_standard_plonk_lagrange_polynomial(barretenberg::polynomial& output,
                                                       const std::vector<permutation_subgroup_element>& permutation,
                                                       const barretenberg::evaluation_domain& small_domain)
{
    if (output.size() < permutation.size()) {
        throw_or_abort("Permutation polynomial size is insufficient to store permutations.");
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

    // `permutation[i]` will specify the 'index' that this wire value will map to.
    // Here, 'index' refers to an element of our subgroup H.
    // We can almost use `permutation[i]` to directly index our `roots` array, which contains our subgroup elements.
    // We first have to accomodate for the fact that `roots` only contains *half* of our subgroup elements. This is
    // because ω^{n/2} = -ω and we don't want to perform redundant work computing roots of unity.

    size_t raw_idx = permutation[i].row_index;

    // Step 1: is `raw_idx` >= (n / 2)? if so, we will need to index `-roots[raw_idx - subgroup_size / 2]` instead
    // of `roots[raw_idx]`
    const bool negative_idx = raw_idx >= root_size;

    // Step 2: compute the index of the subgroup element we'll be accessing.
    // To avoid a conditional branch, we can subtract `negative_idx << log2_root_size` from `raw_idx`.
    // Here, `log2_root_size = numeric::get_msb(subgroup_size / 2)` (we know our subgroup size will be a power of 2,
    // so we lose no precision here)
    const size_t idx = raw_idx - (static_cast<size_t>(negative_idx) << log2_root_size);

    // Call `conditionally_subtract_double_modulus`, using `negative_idx` as our predicate.
    // Our roots of unity table is partially 'overloaded' - we either store the root `w`, or `modulus + w`
    // So to ensure we correctly compute `modulus - w`, we need to compute `2 * modulus - w`
    // The output will similarly be overloaded (containing either 2 * modulus - w, or modulus - w)
    output[i] = roots[idx].conditionally_subtract_from_double_modulus(static_cast<uint64_t>(negative_idx));

    // Finally, if our permutation maps to an index in either the right wire vector, or the output wire vector, we
    // need to multiply our result by one of two quadratic non-residues. (This ensures that mapping into the left
    // wires gives unique values that are not repeated in the right or output wire permutations) (ditto for right
    // wire and output wire mappings)

    if (permutation[i].is_public_input) {
        // As per the paper which modifies plonk to include the public inputs in a permutation argument, the permutation
        // `σ` is modified to `σ'`, where `σ'` maps all public inputs to a set of l distinct ζ elements which are
        // disjoint from H ∪ k1·H ∪ k2·H.
        output[i] *= barretenberg::fr::external_coset_generator();
    } else if (permutation[i].is_tag) {
        output[i] *= barretenberg::fr::tag_coset_generator();
    } else {
        {
            const uint32_t column_index = permutation[i].column_index;
            if (column_index > 0) {
                output[i] *= barretenberg::fr::coset_generator(column_index - 1);
            }
        }
    }
    ITERATE_OVER_DOMAIN_END;
}

/**
 * @brief Given a permutation mapping, compute sigma permutations polynomials in lagrange form and put them in the
 * polynomial cache
 *
 * @details Iterate over the mapping of each wire and calla  function, translating this mapping into lagrange
 * evaluations.
 *
 * @tparam program_width The number of wires
 * @param sigma_mappings At table with permutation information
 * @param key Pointer to the proving key
 */
template <size_t program_width>
void compute_standard_plonk_sigma_lagrange_polynomials_from_mapping(
    std::array<std::vector<permutation_subgroup_element>, program_width>& sigma_mappings, bonk::proving_key* key)
{
    for (size_t i = 0; i < program_width; i++) {
        std::string index = std::to_string(i + 1);
        barretenberg::polynomial sigma_polynomial_lagrange(key->circuit_size);
        compute_standard_plonk_lagrange_polynomial(sigma_polynomial_lagrange, sigma_mappings[i], key->small_domain);
        key->polynomial_store.put("sigma_" + index + "_lagrange", std::move(sigma_polynomial_lagrange));
    }
}

/**
 * @brief Compute the monomial and coset version of each sigma polynomial
 *
 * @details For Plonk we need the monomial and coset form of the polynomials, so we retrieve the lagrange form from
 * polynomial cache, compute FFT versions and put them in the cache
 *
 * @tparam program_width Number of wires
 * @param key Pointer to the proving key
 */
template <size_t program_width> void compute_sigma_polynomials_monomial_and_coset_fft(bonk::proving_key* key)
{
    for (size_t i = 0; i < program_width; ++i) {

        // Construct permutation polynomials in lagrange base
        std::string index = std::to_string(i + 1);

        barretenberg::polynomial sigma_polynomial_lagrange = key->polynomial_store.get("sigma_" + index + "_lagrange");
        // Compute permutation polynomial monomial form
        barretenberg::polynomial sigma_polynomial(key->circuit_size);
        barretenberg::polynomial_arithmetic::ifft(
            &sigma_polynomial_lagrange[0], &sigma_polynomial[0], key->small_domain);

        // Compute permutation polynomial coset FFT form
        barretenberg::polynomial sigma_fft(sigma_polynomial, key->large_domain.size);
        sigma_fft.coset_fft(key->large_domain);

        key->polynomial_store.put("sigma_" + index, std::move(sigma_polynomial));
        key->polynomial_store.put("sigma_" + index + "_fft", std::move(sigma_fft));
    }
}

} // namespace

/**
 * @brief Compute standard honk id polynomials and put them into cache
 *
 * @details Honk permutations involve using id and sigma polynomials to generate variable cycles. This function
 * generates the id polynomials and puts them into polynomial cache, so that they can be used by the prover.
 *
 * @tparam program_width The number of witness polynomials
 * @param key Proving key where we will save the polynomials
 */
template <size_t program_width>
void compute_standard_honk_id_polynomials(auto key) // proving_key* and shared_ptr<proving_key>
{
    const size_t n = key->circuit_size;
    // Fill id polynomials with default values
    for (size_t j = 0; j < program_width; ++j) {
        // Construct permutation polynomials in lagrange base
        barretenberg::polynomial id_j(n);
        for (size_t i = 0; i < key->circuit_size; ++i) {
            id_j[i] = (j * n + i);
        }
        std::string index = std::to_string(j + 1);
        key->polynomial_store.put("id_" + index + "_lagrange", std::move(id_j));
    }
}

/**
 * @brief Compute sigma permutations for standard honk and put them into polynomial cache
 *
 * @details These permutations don't involve sets. We only care about equating one witness value to another. The
 * sequences don't use cosets unlike FFT-based Plonk, because there is no need for them. We simply use indices based on
 * the witness vector and index within the vector. These values are permuted to account for wire copy cycles
 *
 * @tparam program_width
 * @tparam CircuitConstructor
 * @param circuit_constructor
 * @param key
 */
// TODO(#293): Update this (and all similar functions) to take a smart pointer.
template <size_t program_width, typename CircuitConstructor>
void compute_standard_honk_sigma_permutations(CircuitConstructor& circuit_constructor, bonk::proving_key* key)
{
    // Compute the permutation table specifying which element becomes which
    auto sigma_mappings = compute_basic_bonk_sigma_permutations<program_width>(circuit_constructor, key);
    // Compute Honk-style sigma polynomial fromt the permutation table
    compute_honk_style_sigma_lagrange_polynomials_from_mapping(sigma_mappings, key);
}

/**
 * @brief Compute sigma permutation polynomials for standard plonk and put them in the polynomial cache
 *
 * @tparam program_width Number of wires
 * @tparam CircuitConstructor Class holding the circuit
 * @param circuit_constructor An object holdingt he circuit
 * @param key Pointer to a proving key
 */
template <size_t program_width, typename CircuitConstructor>
void compute_standard_plonk_sigma_permutations(CircuitConstructor& circuit_constructor, bonk::proving_key* key)
{
    // Compute the permutation table specifying which element becomes which
    auto sigma_mappings = compute_basic_bonk_sigma_permutations<program_width>(circuit_constructor, key);
    // Compute Plonk-style sigma polynomials from the mapping
    compute_standard_plonk_sigma_lagrange_polynomials_from_mapping(sigma_mappings, key);
    // Compute their monomial and coset versions
    compute_sigma_polynomials_monomial_and_coset_fft<program_width>(key);
}

/**
 * @brief Compute Lagrange Polynomials L_0 and L_{n-1} and put them in the polynomial cache
 *
 * @param key Proving key where we will save the polynomials
 */
inline void compute_first_and_last_lagrange_polynomials(auto key) // proving_key* and share_ptr<proving_key>
{
    const size_t n = key->circuit_size;
    // info("Computing Lagrange basis polys, the  value of n is: ",/s n);
    barretenberg::polynomial lagrange_polynomial_0(n);
    barretenberg::polynomial lagrange_polynomial_n_min_1(n);
    lagrange_polynomial_0[0] = 1;
    lagrange_polynomial_n_min_1[n - 1] = 1;
    key->polynomial_store.put("L_first_lagrange", std::move(lagrange_polynomial_0));
    key->polynomial_store.put("L_last_lagrange", std::move(lagrange_polynomial_n_min_1));
}

} // namespace bonk
