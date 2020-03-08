#include "engine.hpp"

namespace barretenberg {
namespace random {

namespace {
std::seed_seq debug_seed({1, 2, 3, 4, 5, 6, 7, 8});
Engine debug_engine(debug_seed);
}

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

uint32_t Engine::get_random_uint32() {
    return static_cast<uint32_t>(dist(engine));
}

uint64_t Engine::get_random_uint64()
{
    return dist(engine);
}

uint256_t Engine::get_random_uint256() {
    return uint256_t(dist(engine), dist(engine), dist(engine), dist(engine));
}

uint512_t Engine::get_random_uint512() {
    return uint512_t(get_random_uint256(), get_random_uint256());
}

Engine& get_debug_engine() {
    return debug_engine;
}

}
}