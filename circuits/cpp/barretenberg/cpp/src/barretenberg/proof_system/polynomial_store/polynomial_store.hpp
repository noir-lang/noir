#pragma once

#include "barretenberg/common/assert.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include <cstddef>
#include <map>
#include <string>
#include <unordered_map>

namespace proof_system {
/**
 * @brief Basic storage class for Polynomials
 *
 * @tparam Fr
 */
// TODO(Cody): Move into plonk namespace.
template <typename Fr> class PolynomialStore {

    using Polynomial = barretenberg::Polynomial<Fr>;

  private:
    std::unordered_map<std::string, Polynomial> polynomial_map;

  public:
    PolynomialStore() = default;
    PolynomialStore(PolynomialStore& other) noexcept = default;
    PolynomialStore(PolynomialStore&& other) noexcept = default;
    PolynomialStore& operator=(const PolynomialStore& other) = default;
    PolynomialStore& operator=(PolynomialStore&& other) noexcept = default;
    ~PolynomialStore() = default;

    /**
     * @brief Transfer ownership of a polynomial to the PolynomialStore
     *
     * @param key string ID of the polynomial
     * @param value a Polynomial
     */
    inline void put(std::string const& key, Polynomial&& value) { polynomial_map[key] = std::move(value); };

    /**
     * @brief Get a reference to a polynomial in the PolynomialStore; will throw exception if the
     * key does not exist in the map
     *
     * @param key string ID of the polynomial
     * @return Polynomial&; a reference to the polynomial associated with the given key
     */
    inline Polynomial& get(std::string const& key) { return polynomial_map.at(key); };

    /**
     * @brief Erase the polynomial with the given key from the map if it exists. (ASSERT that it does)
     *
     * @param key
     */
    inline void remove(std::string const& key)
    {
        ASSERT(polynomial_map.contains(key));
        polynomial_map.erase(key);
    };

    /**
     * @brief Get the current size (bytes) of all polynomials in the PolynomialStore
     *
     * @return size_t
     */
    inline size_t get_size_in_bytes() const
    {
        size_t size_in_bytes = 0;
        for (auto& entry : polynomial_map) {
            size_in_bytes += sizeof(Fr) * entry.second.size();
        }
        return size_in_bytes;
    };

    /**
     * @brief Print a summary of the PolynomialStore contents
     *
     */
    inline void print()
    {
        double size_in_mb = static_cast<double>(get_size_in_bytes()) / 1e6;
        info("\n PolynomialStore contents (total size ", size_in_mb, " MB):");
        for (auto& entry : polynomial_map) {
            size_t entry_bytes = entry.second.size() * sizeof(Fr);
            info(entry.first, " (", entry_bytes, " bytes): \t", entry.second);
        }
        info();
    }

    // Basic map methods
    bool contains(std::string const& key) { return polynomial_map.contains(key); };
    size_t size() { return polynomial_map.size(); };

    // Allow for const range based for loop
    typename std::unordered_map<std::string, Polynomial>::const_iterator begin() const
    {
        return polynomial_map.begin();
    }
    typename std::unordered_map<std::string, Polynomial>::const_iterator end() const { return polynomial_map.end(); }
};

} // namespace proof_system