#include "engine.hpp"
#include <array>
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
    return static_cast<uint8_t>(dist(engine));
}

uint32_t Engine::get_random_uint32()
{
    return static_cast<uint32_t>(dist(engine));
}

uint64_t Engine::get_random_uint64()
{
    return dist(engine);
}

uint128_t Engine::get_random_uint128()
{
    uint128_t hi = dist(engine);
    uint128_t lo = dist(engine);
    return (hi << 64) | lo;
}

uint256_t Engine::get_random_uint256()
{
    return uint256_t(dist(engine), dist(engine), dist(engine), dist(engine));
}

uint512_t Engine::get_random_uint512()
{
    return uint512_t(get_random_uint256(), get_random_uint256());
}

uint1024_t Engine::get_random_uint1024()
{
    return uint1024_t(get_random_uint512(), get_random_uint512());
}

Engine& get_debug_engine(bool reset)
{
    static Engine debug_engine;
    if (reset) {
        debug_engine = Engine();
    }
    return debug_engine;
}

Engine& get_engine()
{
    static auto random_data = generate_random_data();
    static std::seed_seq random_seed(random_data.begin(), random_data.end());
    static Engine engine(random_seed);
    return engine;
}

} // namespace random
} // namespace numeric