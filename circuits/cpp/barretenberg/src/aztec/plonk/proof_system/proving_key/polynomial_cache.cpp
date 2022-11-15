#include "polynomial_cache.hpp"
#include <common/streams.hpp>

namespace waffle {

using namespace barretenberg;

// Set to 1 to enable logging.
#if 0
template <typename... Args> inline void debug(Args... args)
{
    info("PolynomialCache: ", args...);
}
#else
template <typename... Args> inline void debug(Args...) {}
#endif

PolynomialCache::PolynomialCache(PolynomialStore* store, size_t capacity_bytes)
    : capacity_bytes_(capacity_bytes)
    , store_(store)
{}

PolynomialCache::PolynomialCache(PolynomialCache& other, PolynomialStore* store, size_t capacity_bytes)
    : capacity_bytes_(capacity_bytes)
    , store_(store)
    , map_(other.map_)
    , lru_(other.lru_)
{}

void PolynomialCache::put(std::string const& key, polynomial&& value)
{
    map_[key] = std::move(value);
    move_to_front(key);
    flush(capacity_bytes_);
    debug("put: ", key, " moved to cache. volume_bytes: ", get_volume());
}

polynomial& PolynomialCache::get(std::string const& key, size_t size)
{
    auto map_it = map_.find(key);
    if (map_it != map_.end()) {
        debug("get: ", key, " found in cache.");
        move_to_front(key);
        return map_it->second;
    }

    polynomial poly;
    if (store_) {
        debug("get: ", key, " not in cache, checking store.");
        poly = store_->get(key);
    } else {
        debug("get: ", key, " not in cache.");
    }

    if (poly.get_size() == 0) {
        if (!size) {
            throw_or_abort(format("PolynomialCache: get: ", key, " not found and no size given."));
        }
        debug("get: ", key, " will be adding as zero poly of size ", size);
        poly.resize(size);
    }

    map_[key] = std::move(poly);
    lru_.push_front(key);
    debug("get: ", key, " added to cache.");
    flush(capacity_bytes_);
    return map_[key];
}

size_t PolynomialCache::get_volume() const
{
    return std::accumulate(map_.begin(), map_.end(), size_t(0), [](size_t acc, auto& e) {
        return acc + e.second.get_size() * sizeof(fr);
    });
}

void PolynomialCache::move_to_front(std::string const& key)
{
    auto it = std::find(lru_.begin(), lru_.end(), key);
    if (it != lru_.end()) {
        lru_.splice(lru_.begin(), lru_, it);
    } else {
        lru_.push_front(key);
    }
}

void PolynomialCache::flush(size_t target_bytes)
{
    if (!store_) {
        return;
    }
    while (lru_.size() > 0 && get_volume() > target_bytes) {
        auto key = lru_.back();
        lru_.pop_back();
        auto it = map_.find(key);
        store_->put(key, it->second);
        map_.erase(it);
        debug("flush: ", key, " purged.");
    }
    debug("flush: volume_bytes: ", get_volume());
}

size_t get_cache_capacity(size_t num_gates, waffle::ComposerType composer_type)
{
    // Set capacity of polynomial store cache based on composer type: The idea
    // here is that we can limit the capacity of the polynomial store cache based
    // on the most memory demanding component of the prover algorithm, which
    // is quotient construction (where we need to use coset FFTs). In particular,
    // we set the cache capacity based on the greatest number of coset FFTs that
    // are needed simultaneously by any one widget.
    size_t MAX_NUM_SIMULTANEOUS_COSET_FFTS;
    // Set max number of coset FFTs needed at any one point in the prover algorithm
    switch (composer_type) {
    case ComposerType::STANDARD: {
        // Standard arithmetic widget requires 8 coset FFTs
        MAX_NUM_SIMULTANEOUS_COSET_FFTS = 8;
        break;
    };
    case ComposerType::TURBO: {
        // Turbo arithmetic widget requires 12 coset FFTs
        MAX_NUM_SIMULTANEOUS_COSET_FFTS = 12;
        break;
    };
    case ComposerType::PLOOKUP: {
        // This number is TBD for UltraPlonk/Plookup
        MAX_NUM_SIMULTANEOUS_COSET_FFTS = 20; // TBD
        break;
    };
    default: {
        throw_or_abort("Received invalid composer type");
    }
    };

    // The max capacity in bytes that the polynomial store cache should require is equal to
    // (size of one coset FFT poly) * (size of field element fr) * (max simultaneous coset FFTs).
    // The max number of simultaneous coset FFTs is determined by the widget requiring the most
    // coset FFT polynomials.
    // Note: that the size of some coset FFTs is 4n (selectors) and others are 4n+4 (witnesses)
    // so this is a slight overestimate of the required capacity
    size_t size_of_coset_fft = 4 * num_gates + 4;
    size_t fr_bytes = sizeof(barretenberg::fr);
    size_t min_required_capacity = size_of_coset_fft * fr_bytes * MAX_NUM_SIMULTANEOUS_COSET_FFTS;

    return min_required_capacity;
}
} // namespace waffle