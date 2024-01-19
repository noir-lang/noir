#pragma once

#include "barretenberg/common/assert.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include <cstddef>
#include <map>
#include <string>
#include <unordered_map>

namespace bb {

template <typename Fr> class PolynomialStore {
  private:
    using Polynomial = bb::Polynomial<Fr>;
    std::unordered_map<std::string, Polynomial> polynomial_map;

  public:
    /**
     * Transfer ownership of a polynomial to the PolynomialStore.
     */
    void put(std::string const& key, Polynomial&& value);

    /**
     * Returns a polynomial by value.
     * A shallow copy of the polynomial is taken, which the compiler will move to the caller.
     */
    Polynomial get(std::string const& key);

    void remove(std::string const& key);

    size_t get_size_in_bytes() const;

    void print();

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

} // namespace bb
