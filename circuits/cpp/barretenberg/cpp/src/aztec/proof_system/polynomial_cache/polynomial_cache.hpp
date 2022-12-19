#pragma once

#include <unordered_map>
#include <list>
#include "polynomial_store.hpp"
#include "../../plonk/proof_system/types/polynomial_manifest.hpp"

namespace waffle {

using namespace barretenberg;

/**
 * Implements a memory cache on top of an underlying PolynomialStore.
 */
class PolynomialCache {
  public:
    /**
     * @brief If no store is given, the cache acts as an unbounded memory store.
     */
    PolynomialCache(PolynomialStore* underlying = nullptr, size_t capacity_bytes = 0);

    PolynomialCache(PolynomialCache& other, PolynomialStore* underlying = nullptr, size_t capacity_bytes = 0);

    /**
     * @brief Ownership of the polynomial is taken by the cache.
     * flush() is called after adding the polynomial to bring usage below capacity_bytes.
     */
    void put(std::string const& key, polynomial&& value);

    /**
     * @brief A reference to the polynomial is returned if it exists, ownership is retained by the store.
     * If the polynomial doesn't exist and we have size we `put` a new poly and return, else we throw_or_abort.
     */
    polynomial& get(std::string const& key, size_t size = 0);

    /**
     * @brief Get the current volume (bytes) of all polynomials in the cache
     */
    size_t get_volume() const;

    std::vector<std::string> const get_lru() const { return std::vector(lru_.begin(), lru_.end()); }

  private:
    size_t capacity_bytes_;                           // max allowed memory (bytes) consumed by the polys in the cache
    PolynomialStore* store_;                          // underlying store that handles writing polys to file if enabled
    std::unordered_map<std::string, polynomial> map_; // actual container for all polynomials in the cache
    std::list<std::string> lru_; // imposes ordering of cache according to 'Least Recently Used' rule

    /**
     * @brief Move polynomial id to the front of lru_
     */
    void move_to_front(std::string const& key);

    /**
     * @brief Purges polynomials from map_ starting at the end lru_ until volume is below target_bytes.
     * Each purged polynomial is `put` onto the underlying store.
     */
    void flush(size_t target_bytes);
};

size_t get_cache_capacity(size_t num_gates, waffle::ComposerType composer_type);

} // namespace waffle