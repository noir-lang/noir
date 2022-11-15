#pragma once

#include <unordered_map>
#include <polynomials/polynomial.hpp>

namespace waffle {

using namespace barretenberg;

class PolynomialStore {
  public:
    virtual ~PolynomialStore() {}

    /**
     * @brief Put a polynomial into the store.
     * Ownership of the polynomial is retained by the caller.
     */
    virtual void put(std::string const& key, polynomial const& value) = 0;

    /**
     * @brief Get a polynomial from the store.
     * If the polynomial doesn't exist, we return a 0 sized polynomial.
     */
    virtual polynomial get(std::string const& key) const = 0;
};

/**
 * @brief Local memory store;
 * for testing purposes, does not reduce local mem consumption
 */
class PolynomialStoreMem : public PolynomialStore {
  public:
    void put(std::string const& key, polynomial const& value);

    polynomial get(std::string const& key) const;

    auto& get_stats() const { return stats_; }

  private:
    std::unordered_map<std::string, polynomial> map_;
    mutable std::map<std::string, std::pair<size_t, size_t>> stats_;
};

/**
 * @brief WASM local memory store;
 * for testing purposes, does not reduce local mem consumption
 *
 */
class PolynomialStoreWasm : public PolynomialStore {
  public:
    void put(std::string const& key, polynomial const& value);

    polynomial get(std::string const& key) const;
};

} // namespace waffle