#include <polynomials/polynomial.hpp>
#include <common/max_threads.hpp>
#include <span>

#ifndef NO_MULTITHREADING
#include "omp.h"
#endif
namespace honk {
namespace power_polynomial {
/**
 * @brief Generate the power polynomial vector
 *
 * @details Generates a vector, where v[i]=ζⁱ
 *
 * @param zeta
 * @param vector_size
 * @return barretenberg::polynomial
 */
template <typename Fr> barretenberg::Polynomial<Fr> generate_vector(Fr zeta, size_t vector_size)
{
    // We know the size from the start, so we can allocate exactly the right amount of memory
    barretenberg::Polynomial<Fr> pow_vector(vector_size);

    constexpr size_t usefulness_margin = 4;
    size_t num_threads = max_threads::compute_num_threads();
    if (vector_size < (usefulness_margin * num_threads)) {
        num_threads = 1;
    }
    // Prepare for a random number of threads. We need to handle the last thread separately
    size_t thread_size = vector_size / num_threads;
    size_t last_thread_size = thread_size;
    // Check if the vector size is divided into threads cleanly
    if ((vector_size % thread_size) != 0) {
        thread_size += 1;
        last_thread_size = vector_size % thread_size;
    }
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < num_threads; i++) {
        // Exponentiate ζ to the starting power of the chunk
        Fr starting_power = zeta.pow(i * thread_size);
        // Set the chunk size depending ontwhether this is the last thread
        size_t chunk_size = i != (num_threads - 1) ? thread_size : last_thread_size;
        size_t j = 0;
        // Go through elements and compute ζ powers
        for (; j < chunk_size - 1; j++) {
            pow_vector[i * thread_size + j] = starting_power;
            starting_power *= zeta;
        }
        pow_vector[i * thread_size + j] = starting_power;
    }
    return pow_vector;
}

/**
 * @brief Evaluate the power polynomial on {x_0,..,x_d}
 *
 * @details The power polynomial can be efficiently evaluated as ∏( ( b^{2^i} - 1 ) * x_i + 1)
 *
 * @param zeta ζ
 * @param variables
 * @return barretenberg::fr
 */
template <typename Fr> Fr evaluate(Fr zeta, const std::span<Fr>& variables)
{
    Fr evaluation = Fr::one();
    for (size_t i = 0; i < variables.size(); i++) {
        // evaulutaion *= b^{2^i} - 1) * x_i + 1
        evaluation *= (zeta - 1) * variables[i] + 1;
        zeta *= zeta;
    }
    return evaluation;
}
} // namespace power_polynomial
} // namespace honk
