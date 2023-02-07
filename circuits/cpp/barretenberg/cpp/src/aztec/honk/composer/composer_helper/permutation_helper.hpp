#pragma once

#include <ecc/curves/bn254/fr.hpp>
#include <polynomials/polynomial.hpp>
#include <proof_system/proving_key/proving_key.hpp>

#include <algorithm>
#include <cstdint>
#include <initializer_list>
#include <cstdint>
#include <cstddef>
#include <utility>
#include <vector>

namespace honk {

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
using CyclicPermutation = std::vector<cycle_node>;

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
template <size_t program_width, typename CircuitConstructor>
void compute_standard_honk_sigma_permutations(CircuitConstructor& circuit_constructor, waffle::proving_key* key)
{
    // Compute wire copy cycles for public and private variables
    std::vector<CyclicPermutation> copy_cycles = compute_wire_copy_cycles<program_width>(circuit_constructor);
    const size_t n = key->circuit_size;

    // Initialize sigma[0], sigma[1], ..., as the identity permutation
    // at the end of the loop, sigma[j][i] = j*n + i
    std::array<barretenberg::polynomial, program_width> sigma;
    for (size_t j = 0; j < program_width; ++j) {
        sigma[j] = barretenberg::polynomial(n);
        for (size_t i = 0; i < n; i++) {
            sigma[j][i] = (j * n + i);
        }
    }

    // Each cycle is a partition of the indexes
    for (auto& single_copy_cycle : copy_cycles) {
        const size_t cycle_size = single_copy_cycle.size();

        // If we use assert equal, we lose a real variable index, which creates an empty cycle
        if (cycle_size == 0) {
            continue;
        }

        // next_index represents the index of the variable that the current node in the cycle should point to.
        // We iterate over the cycle in reverse order, so the index of the last node should map to the index of the
        // first one.
        const auto [first_col, first_idx] = single_copy_cycle.front();
        auto next_index = sigma[first_col][first_idx];

        // The index of variable reference by the j-th node should map to the index of the (j+1)-th node.
        // The last one points to the first, and the index of the latter is stored in `next_index`.
        // When we get to the second node in the list, we replace it with the index of the third node, and save the
        // index it currently points to into `next_index`
        for (size_t j = cycle_size - 1; j != 0; --j) {
            const auto [current_col, current_idx] = single_copy_cycle[j];
            next_index = std::exchange(sigma[current_col][current_idx], next_index);
        }
        // After the loop ends, we make the first node point to the index of the second node,
        // thereby completing the cycle.
        sigma[first_col][first_idx] = next_index;
    }

    // We intentionally want to break the cycles of the public input variables.
    // During the witness generation, the left and right wire polynomials at index i contain the i-th public input.
    // The CyclicPermutation created for these variables always start with (i) -> (n+i), followed by the indices of the
    // variables in the "real" gates.
    // We make i point to -(i+1), so that the only way of repairing the cycle is add the mapping
    //  -(i+1) -> (n+i)
    // These indices are chosen so they can easily be computed by the verifier. They can expect the running product
    // to be equal to the "public input delta" that is computed in <honk/utils/public_inputs.hpp>
    const auto num_public_inputs = static_cast<uint32_t>(circuit_constructor.public_inputs.size());
    for (size_t i = 0; i < num_public_inputs; ++i) {
        sigma[0][i] = -barretenberg::fr(i + 1);
    }

    // Save to polynomial cache
    for (size_t j = 0; j < program_width; j++) {
        std::string index = std::to_string(j + 1);
        key->polynomial_cache.put("sigma_" + index + "_lagrange", std::move(sigma[j]));
    }
}

/**
 * @brief Compute standard honk id polynomials and put them into cache
 *
 * @details Honk permutations involve using id and sigma polynomials to generate variable cycles. This function
 * generates the id polynomials and puts them into polynomial cache, so that they can be used by the prover.
 *
 * @tparam program_width The number of witness polynomials
 * @param key Proving key where we will save the polynomials
 */
// TODO(Cody): why not a shared pointer here?/s
template <size_t program_width>
void compute_standard_honk_id_polynomials(auto key) // proving_key* and share_ptr<proving_key>
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
        key->polynomial_cache.put("id_" + index + "_lagrange", std::move(id_j));
    }
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
    key->polynomial_cache.put("L_first_lagrange", std::move(lagrange_polynomial_0));
    key->polynomial_cache.put("L_last_lagrange", std::move(lagrange_polynomial_n_min_1));
}

} // namespace honk
