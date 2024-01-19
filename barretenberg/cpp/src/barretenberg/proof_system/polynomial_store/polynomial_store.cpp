#include "polynomial_store.hpp"
#include "barretenberg/common/assert.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include <cstddef>
#include <map>
#include <string>
#include <unordered_map>

namespace bb {

template <typename Fr> void PolynomialStore<Fr>::put(std::string const& key, Polynomial&& value)
{
    // info("put ", key, ": ", value.hash());
    polynomial_map[key] = std::move(value);
    // info("poly store put: ", key, " ", get_size_in_bytes() / (1024 * 1024), "MB");
};

/**
 * @brief Get a reference to a polynomial in the PolynomialStore; will throw exception if the
 * key does not exist in the map
 *
 * @param key string ID of the polynomial
 * @return Polynomial&; a reference to the polynomial associated with the given key
 */
template <typename Fr> bb::Polynomial<Fr> PolynomialStore<Fr>::get(std::string const& key)
{
    // info("poly store get: ", key);
    // Take a shallow copy of the polynomial. Compiler will move the shallow copy to call site.
    auto p = polynomial_map.at(key).share();
    // info("got ", key, ": ", p.hash());
    return p;
};

/**
 * @brief Erase the polynomial with the given key from the map if it exists. (ASSERT that it does)
 *
 * @param key
 */
template <typename Fr> void PolynomialStore<Fr>::remove(std::string const& key)
{
    ASSERT(polynomial_map.contains(key));
    polynomial_map.erase(key);
};

/**
 * @brief Get the current size (bytes) of all polynomials in the PolynomialStore
 *
 * @return size_t
 */
template <typename Fr> size_t PolynomialStore<Fr>::get_size_in_bytes() const
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
template <typename Fr> void PolynomialStore<Fr>::print()
{
    double size_in_mb = static_cast<double>(get_size_in_bytes()) / 1e6;
    info("\n PolynomialStore contents (total size ", size_in_mb, " MB):");
    for (auto& entry : polynomial_map) {
        size_t entry_bytes = entry.second.size() * sizeof(Fr);
        info(entry.first, " (", entry_bytes, " bytes): \t", entry.second);
    }
    info();
}

template class PolynomialStore<bb::fr>;

} // namespace bb