#include "polynomial_store.hpp"

namespace waffle {

using namespace barretenberg;

// Set to 1 to enable logging.
#if 0
template <typename... Args> inline void debug(Args... args)
{
    info("PolynomialStoreMem: ", args...);
}
#else
template <typename... Args> inline void debug(Args...) {}
#endif

void PolynomialStoreMem::put(std::string const& key, polynomial const& value)
{
    debug("put: taking copy of polynomial: ", key);
    map_[key] = value;
    auto& stats = stats_[key];
    stats.first++;
}

polynomial PolynomialStoreMem::get(std::string const& key) const
{
    auto map_it = map_.find(key);
    auto& stats = stats_[key];
    stats.second++;
    if (map_it != map_.end()) {
        debug("get: returning polynomial: ", key);
        return map_it->second;
    }

    return polynomial();
}

} // namespace waffle