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

Engine::Engine(std::seed_seq& seed)
    : engine(std::mt19937_64(seed))
{}

Engine::Engine(const Engine& other)
    : engine(other.engine)
    , dist(other.dist)
{}

Engine::Engine(Engine&& other)
    : engine(other.engine)
    , dist(other.dist)
{}

Engine& Engine::operator=(const Engine& other)
{
    engine = other.engine;
    dist = other.dist;
    return *this;
}

Engine& Engine::operator=(Engine&& other)
{
    engine = other.engine;
    dist = other.dist;
    return *this;
}

uint32_t Engine::get_random_uint32()
{
    return static_cast<uint32_t>(dist(engine));
}

uint64_t Engine::get_random_uint64()
{
    return dist(engine);
}

uint256_t Engine::get_random_uint256()
{
    return uint256_t(dist(engine), dist(engine), dist(engine), dist(engine));
}

uint512_t Engine::get_random_uint512()
{
    return uint512_t(get_random_uint256(), get_random_uint256());
}

Engine& get_debug_engine()
{
    static std::seed_seq debug_seed({ 1, 2, 3, 4, 5, 6, 7, 8 });
    static Engine debug_engine(debug_seed);
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