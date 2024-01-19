#pragma once
#include "barretenberg/common/zip_view.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

namespace bb::honk {
/**
 * @brief Get a ProverPolynomials instance initialized to sequential values starting at 0.
 * @details Values are assigned according to the order specified in the underlying array of the flavor class. The
 * function returns an array of data pointed to by the ProverPolynomials.
 */
template <typename Flavor>
typename Flavor::ProverPolynomials get_sequential_prover_polynomials(const size_t log_circuit_size,
                                                                     const size_t starting_value)
{
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

    typename Flavor::ProverPolynomials prover_polynomials;
    size_t circuit_size = 1 << log_circuit_size;
    size_t value_idx = starting_value;
    for (auto& polynomial : prover_polynomials.get_all()) {
        polynomial = Polynomial(circuit_size);
        for (auto& value : polynomial) {
            value = FF(value_idx++);
        }
    }
    return prover_polynomials;
}

template <typename Flavor> typename Flavor::ProverPolynomials get_zero_prover_polynomials(const size_t log_circuit_size)
{
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

    typename Flavor::ProverPolynomials prover_polynomials;
    size_t circuit_size = 1 << log_circuit_size;
    for (auto& polynomial : prover_polynomials.get_all()) {
        polynomial = Polynomial(circuit_size);
        for (auto& value : polynomial) {
            value = FF(0);
        }
    }
    return prover_polynomials;
}

} // namespace bb::honk