#pragma once
#include "barretenberg/polynomials/polynomial.hpp"

namespace proof_system::honk {
/**
 * @brief Get a ProverPolynomials instance initialized to sequential values starting at 0.
 * @details Values are assigned according to the order specified in the underlying array of the flavor class. The
 * function returns an array of data pointed to by the ProverPolynomials.
 */
template <typename Flavor>
std::pair<std::array<barretenberg::Polynomial<typename Flavor::FF>, Flavor::NUM_ALL_ENTITIES>,
          typename Flavor::ProverPolynomials>
get_sequential_prover_polynomials(const size_t log_circuit_size, const size_t starting_value)
{
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Polynomial = typename Flavor::Polynomial;

    std::array<barretenberg::Polynomial<typename Flavor::FF>, Flavor::NUM_ALL_ENTITIES> storage;
    size_t circuit_size = 1 << log_circuit_size;
    size_t value_idx = starting_value;
    for (auto& polynomial : storage) {
        polynomial = Polynomial(circuit_size);
        for (auto& value : polynomial) {
            value = FF(value_idx++);
        }
    }

    ProverPolynomials prover_polynomials;
    auto prover_polynomials_pointers = prover_polynomials.pointer_view();
    size_t poly_idx = 0;
    for (auto& polynomial : storage) {
        *prover_polynomials_pointers[poly_idx] = polynomial;
        poly_idx++;
    }

    return std::pair(std::move(storage), prover_polynomials);
}

template <typename Flavor>
std::pair<std::array<barretenberg::Polynomial<typename Flavor::FF>, Flavor::NUM_ALL_ENTITIES>,
          typename Flavor::ProverPolynomials>
get_zero_prover_polynomials(const size_t log_circuit_size)
{
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Polynomial = typename Flavor::Polynomial;

    std::array<barretenberg::Polynomial<typename Flavor::FF>, Flavor::NUM_ALL_ENTITIES> storage;
    size_t circuit_size = 1 << log_circuit_size;
    for (auto& polynomial : storage) {
        polynomial = Polynomial(circuit_size);
        for (auto& value : polynomial) {
            value = FF(0);
        }
    }

    ProverPolynomials prover_polynomials;
    size_t poly_idx = 0;
    auto prover_polynomial_pointers = prover_polynomials.pointer_view();
    for (auto& polynomial : storage) {
        *prover_polynomial_pointers[poly_idx] = polynomial;
        poly_idx++;
    }

    return std::pair(std::move(storage), prover_polynomials);
}

} // namespace proof_system::honk