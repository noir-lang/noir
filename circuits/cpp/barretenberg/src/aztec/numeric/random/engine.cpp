#include "engine.hpp"
#include <array>
#include <common/assert.hpp>
#include <functional>
#include <random>

namespace numeric {
namespace random {

namespace {
auto generate_random_data()
{
    std::array<unsigned int, 32> random_data;
    std::random_device source;
    std::generate(std::begin(random_data), std::end(random_data), std::ref(source));
    return random_data;
}
} // namespace

class RandomEngine : public Engine {
  public:
    uint8_t get_random_uint8()
    {
        auto buf = generate_random_data();
        uint32_t out = buf[0];
        return static_cast<uint8_t>(out);
    }

    uint32_t get_random_uint32()
    {
        auto buf = generate_random_data();
        uint32_t out = buf[0];
        return static_cast<uint32_t>(out);
    }

    uint64_t get_random_uint64()
    {
        auto buf = generate_random_data();
        uint64_t lo = static_cast<uint64_t>(buf[0]);
        uint64_t hi = static_cast<uint64_t>(buf[1]);
        return (lo + (hi << 32ULL));
    }

    uint128_t get_random_uint128()
    {
        auto big = get_random_uint256();
        uint128_t lo = static_cast<uint128_t>(big.data[0]);
        uint128_t hi = static_cast<uint128_t>(big.data[1]);
        return (lo + (hi << (uint128_t)(64ULL)));
    }

    uint256_t get_random_uint256()
    {
        const auto get64 = [](const std::array<uint32_t, 32>& buffer, const size_t offset) {
            uint64_t lo = static_cast<uint64_t>(buffer[0 + offset]);
            uint64_t hi = static_cast<uint64_t>(buffer[1 + offset]);
            return (lo + (hi << 32ULL));
        };
        auto buf = generate_random_data();
        uint64_t lolo = get64(buf, 0);
        uint64_t lohi = get64(buf, 2);
        uint64_t hilo = get64(buf, 4);
        uint64_t hihi = get64(buf, 6);
        return uint256_t(lolo, lohi, hilo, hihi);
    }
};

class DebugEngine : public Engine {
  public:
    DebugEngine()
        : engine(std::mt19937_64(12345))
    {}

    DebugEngine(std::seed_seq& seed)
        : engine(std::mt19937_64(seed))
    {}

    uint8_t get_random_uint8() { return static_cast<uint8_t>(dist(engine)); }

    uint32_t get_random_uint32() { return static_cast<uint32_t>(dist(engine)); }

    uint64_t get_random_uint64() { return dist(engine); }

    uint128_t get_random_uint128()
    {
        uint128_t hi = dist(engine);
        uint128_t lo = dist(engine);
        return (hi << 64) | lo;
    }

    uint256_t get_random_uint256()
    {
        // Do not inline in constructor call. Evaluation order is important for cross-compiler consistency.
        auto a = dist(engine);
        auto b = dist(engine);
        auto c = dist(engine);
        auto d = dist(engine);
        return uint256_t(a, b, c, d);
    }

    uint512_t get_random_uint512();

    uint1024_t get_random_uint1024();

  private:
    std::mt19937_64 engine;
    std::uniform_int_distribution<uint64_t> dist = std::uniform_int_distribution<uint64_t>{ 0ULL, UINT64_MAX };
};

Engine& get_debug_engine(bool reset)
{
    // static std::seed_seq seed({ 1, 2, 3, 4, 5 });
    static DebugEngine debug_engine;
    if (reset) {
        debug_engine = DebugEngine();
    }
    return debug_engine;
}

Engine& get_engine()
{
    static RandomEngine engine;
    return engine;
}

} // namespace random
} // namespace numeric