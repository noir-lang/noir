#include "engine.hpp"
#include "barretenberg/common/assert.hpp"
#include <array>
#include <functional>
#include <random>

namespace bb::numeric {

namespace {
auto generate_random_data()
{
    std::array<unsigned int, 32> random_data;
    std::random_device source;
    std::generate(std::begin(random_data), std::end(random_data), std::ref(source));
    return random_data;
}
} // namespace

class RandomEngine : public RNG {
  public:
    uint8_t get_random_uint8() override
    {
        auto buf = generate_random_data();
        uint32_t out = buf[0];
        return static_cast<uint8_t>(out);
    }

    uint16_t get_random_uint16() override
    {
        auto buf = generate_random_data();
        uint32_t out = buf[0];
        return static_cast<uint16_t>(out);
    }

    uint32_t get_random_uint32() override
    {
        auto buf = generate_random_data();
        uint32_t out = buf[0];
        return static_cast<uint32_t>(out);
    }

    uint64_t get_random_uint64() override
    {
        auto buf = generate_random_data();
        auto lo = static_cast<uint64_t>(buf[0]);
        auto hi = static_cast<uint64_t>(buf[1]);
        return (lo + (hi << 32ULL));
    }

    uint128_t get_random_uint128() override
    {
        auto big = get_random_uint256();
        auto lo = static_cast<uint128_t>(big.data[0]);
        auto hi = static_cast<uint128_t>(big.data[1]);
        return (lo + (hi << static_cast<uint128_t>(64ULL)));
    }

    uint256_t get_random_uint256() override
    {
        const auto get64 = [](const std::array<uint32_t, 32>& buffer, const size_t offset) {
            auto lo = static_cast<uint64_t>(buffer[0 + offset]);
            auto hi = static_cast<uint64_t>(buffer[1 + offset]);
            return (lo + (hi << 32ULL));
        };
        auto buf = generate_random_data();
        uint64_t lolo = get64(buf, 0);
        uint64_t lohi = get64(buf, 2);
        uint64_t hilo = get64(buf, 4);
        uint64_t hihi = get64(buf, 6);
        return { lolo, lohi, hilo, hihi };
    }
};

class DebugEngine : public RNG {
  public:
    DebugEngine()
        // disable linting for this line: we want the DEBUG engine to produce predictable pseudorandom numbers!
        // NOLINTNEXTLINE(cert-msc32-c, cert-msc51-cpp)
        : engine(std::mt19937_64(12345))
    {}

    DebugEngine(std::seed_seq& seed)
        : engine(std::mt19937_64(seed))
    {}

    uint8_t get_random_uint8() override { return static_cast<uint8_t>(dist(engine)); }

    uint16_t get_random_uint16() override { return static_cast<uint16_t>(dist(engine)); }

    uint32_t get_random_uint32() override { return static_cast<uint32_t>(dist(engine)); }

    uint64_t get_random_uint64() override { return dist(engine); }

    uint128_t get_random_uint128() override
    {
        uint128_t hi = dist(engine);
        uint128_t lo = dist(engine);
        return (hi << 64) | lo;
    }

    uint256_t get_random_uint256() override
    {
        // Do not inline in constructor call. Evaluation order is important for cross-compiler consistency.
        auto a = dist(engine);
        auto b = dist(engine);
        auto c = dist(engine);
        auto d = dist(engine);
        return { a, b, c, d };
    }

  private:
    std::mt19937_64 engine;
    std::uniform_int_distribution<uint64_t> dist = std::uniform_int_distribution<uint64_t>{ 0ULL, UINT64_MAX };
};

/**
 * Used by tests to ensure consistent behavior.
 */
RNG& get_debug_randomness(bool reset)
{
    // static std::seed_seq seed({ 1, 2, 3, 4, 5 });
    static DebugEngine debug_engine;
    if (reset) {
        debug_engine = DebugEngine();
    }
    return debug_engine;
}

/**
 * Default engine. If wanting consistent proof construction, uncomment the line to return the debug engine.
 */
RNG& get_randomness()
{
    // return get_debug_randomness();
    static RandomEngine engine;
    return engine;
}

} // namespace bb::numeric
