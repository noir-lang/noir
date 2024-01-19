#include "./polynomial_store_cache.hpp"

namespace bb {

PolynomialStoreCache::PolynomialStoreCache()
    : max_cache_size_(40)
{}

PolynomialStoreCache::PolynomialStoreCache(size_t max_cache_size)
    : max_cache_size_(max_cache_size)
{}

void PolynomialStoreCache::put(std::string const& key, Polynomial&& value)
{
    // info("cache put ", key);
    auto it = cache_.find(key);
    if (it != cache_.end()) {
        it->second = std::move(value);
        return;
    }

    purge_until_free();

    auto size = value.size();
    auto [cache_it, _] = cache_.insert({ key, std::move(value) });
    size_map_.insert({ size, cache_it });
};

PolynomialStoreCache::Polynomial PolynomialStoreCache::get(std::string const& key)
{
    auto it = cache_.find(key);
    if (it != cache_.end()) {
        // info("cache get hit ", key);
        return it->second.share();
    }

    // info("cache get miss ", key);
    return external_store.get(key);
};

void PolynomialStoreCache::purge_until_free()
{
    while (cache_.size() >= max_cache_size_) {
        auto size_it = size_map_.begin();
        auto [size, cache_it] = *size_it;
        auto key = cache_it->first;
        auto p = std::move(cache_it->second);
        size_map_.erase(size_it);
        cache_.erase(cache_it);
        // info("cache purging ", key, " size ", size);
        external_store.put(key, std::move(p));
    }
}

} // namespace bb