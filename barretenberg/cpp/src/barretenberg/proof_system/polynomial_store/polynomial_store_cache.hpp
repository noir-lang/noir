#pragma once
#include "./polynomial_store_wasm.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include <map>
#include <string>

namespace bb {

/**
 * A cache that wraps an underlying external store. It favours holding the largest polynomials in it's cache up
 * to max_cache_size_ polynomials. This saves on many expensive copies of large amounts of memory to the external
 * store. Smaller polynomials get swapped out, but they're also much cheaper to read/write.
 * The default ctor sets the cache size to 70.
 * In combination with the slab allocator, this brings us to about 4GB mem usage for 512k circuits.
 * In tests using just the external store increased proof time from by about 50%.
 * This pretty much recoups all losses.
 */
class PolynomialStoreCache {
  private:
    using Polynomial = bb::Polynomial<bb::fr>;
    std::map<std::string, Polynomial> cache_;
    std::multimap<size_t, std::map<std::string, Polynomial>::iterator> size_map_;
    PolynomialStoreWasm<bb::fr> external_store;
    size_t max_cache_size_;

  public:
    PolynomialStoreCache();
    explicit PolynomialStoreCache(size_t max_cache_size_);

    void put(std::string const& key, Polynomial&& value);

    Polynomial get(std::string const& key);

  private:
    void purge_until_free();
};

} // namespace bb