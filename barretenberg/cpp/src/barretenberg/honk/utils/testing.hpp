#pragma once
#include "barretenberg/common/zip_view.hpp"
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
    for (auto [prover_poly, storage_poly] : zip_view(prover_polynomials.get_all(), storage)) {
        prover_poly = storage_poly;
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
    for (auto [prover_poly, storage_poly] : zip_view(prover_polynomials.get_all(), storage)) {
        prover_poly = storage_poly;
    }

    return std::pair(std::move(storage), prover_polynomials);
}

} // namespace proof_system::honk