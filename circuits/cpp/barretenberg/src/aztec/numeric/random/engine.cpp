#include "engine.hpp"
#include <array>
#include <common/assert.hpp>
#include <functional>

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

Engine::Engine() {}

Engine::Engine(std::seed_seq& seed)
    : engine(std::mt19937_64(seed))
{}

Engine::Engine(const Engine& other)
    : engine(other.engine)
    , dist(other.dist)
{}

Engine::Engine(Engine&& other)
    : engine(std::move(other.engine))
    , dist(std::move(other.dist))
{}

Engine& Engine::operator=(const Engine& other)
{
    engine = other.engine;
    dist = other.dist;
    return *this;
}

Engine& Engine::operator=(Engine&& other)
{
    engine = std::move(other.engine);
    dist = std::move(other.dist);
    return *this;
}

uint8_t Engine::get_random_uint8()
{
    if (is_debug) {
        return static_cast<uint8_t>(dist(engine));
    }
    auto buf = generate_random_data();
    uint32_t out = buf[0];
    return static_cast<uint8_t>(out);
}

uint32_t Engine::get_random_uint32()
{
    if (is_debug) {
        return static_cast<uint32_t>(dist(engine));
    }
    auto buf = generate_random_data();
    uint32_t out = buf[0];
    return static_cast<uint32_t>(out);
}

uint64_t Engine::get_random_uint64()
{
    if (is_debug) {
        return dist(engine);
    }
    auto buf = generate_random_data();
    uint64_t lo = static_cast<uint64_t>(buf[0]);
    uint64_t hi = static_cast<uint64_t>(buf[1]);
    return (lo + (hi << 32ULL));
}

uint128_t Engine::get_random_uint128()
{
    if (is_debug) {
        uint128_t hi = dist(engine);
        uint128_t lo = dist(engine);
        return (hi << 64) | lo;
    }
    auto big = get_random_uint256();
    uint128_t lo = static_cast<uint128_t>(big.data[0]);
    uint128_t hi = static_cast<uint128_t>(big.data[1]);
    return (lo + (hi << (uint128_t)(64ULL)));
}

uint256_t Engine::get_random_uint256()
{
    if (is_debug) {
        return uint256_t(dist(engine), dist(engine), dist(engine), dist(engine));
    }
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

uint512_t Engine::get_random_uint512()
{
    return uint512_t(get_random_uint256(), get_random_uint256());
}

uint1024_t Engine::get_random_uint1024()
{
    return uint1024_t(get_random_uint512(), get_random_uint512());
}

static bool init = false;
Engine& get_debug_engine(bool reset)
{
    static Engine debug_engine;
    if (!init) {
        init = true;
    } else {
        ASSERT(debug_engine.is_debug = true);
    }
    if (reset) {
        debug_engine = Engine();
    }
    debug_engine.is_debug = true;
    return debug_engine;
}

Engine& get_engine()
{
    static auto random_data = generate_random_data();
    static std::seed_seq random_seed(random_data.begin(), random_data.end());
    static Engine engine(random_seed);
    if (!init) {
        init = true;
    } else {
        ASSERT(engine.is_debug = false);
    }
    engine.is_debug = false;
    return engine;
}

} // namespace random
} // namespace numeric