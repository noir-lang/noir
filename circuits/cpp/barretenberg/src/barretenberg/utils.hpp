#pragma once

#include "stdint.h"
#include "unistd.h"
#include <random>

#ifdef _WIN32
#define BBERG_INLINE __forceinline inline
#else
#define BBERG_INLINE __attribute__((always_inline)) inline
#endif

namespace unstd {
template <class Lambda, int = (Lambda{}(), 0)> constexpr bool is_constant_evaluated(Lambda)
{
    return true;
}
constexpr bool is_constant_evaluated(...)
{
    return false;
}
} // namespace unstd

namespace barretenberg {
namespace random {

class Engine {
  public:
    Engine(std::seed_seq& seed)
        : engine(std::mt19937_64(seed))
    {}

    Engine(const Engine& other)
        : engine(other.engine)
        , dist(other.dist)
    {}

    Engine(Engine&& other)
        : engine(other.engine)
        , dist(other.dist)
    {}

    Engine& operator=(const Engine& other)
    {
        engine = other.engine;
        dist = other.dist;
        return *this;
    }

    Engine& operator=(Engine&& other)
    {
        engine = other.engine;
        dist = other.dist;
        return *this;
    }

    uint64_t get_random_uint64()
    {
        uint64_t out = dist(engine);
        return out;
    }

  private:
    std::mt19937_64 engine;
    std::uniform_int_distribution<uint64_t> dist = std::uniform_int_distribution<uint64_t>{ 0ULL, UINT64_MAX };
};

static std::seed_seq debug_seed = std::seed_seq{ 1, 2, 3, 4, 5, 6, 7, 8 };
static std::uniform_int_distribution<uint64_t> dist = std::uniform_int_distribution<uint64_t>{ 0ULL, UINT64_MAX };
static std::mt19937_64 debug_engine = std::mt19937_64(debug_seed);

inline std::mt19937_64* get_debug_engine()
{
    return &debug_engine;
}

inline std::uniform_int_distribution<uint64_t>* get_distribution()
{
    return &dist;
}
} // namespace random
} // namespace barretenberg