#pragma once
#include "../uint128/uint128.hpp"
#include "../uint256/uint256.hpp"
#include "../uintx/uintx.hpp"
#include "stdint.h"
#include "unistd.h"
#include <random>

namespace numeric {
namespace random {

class Engine {
  public:
    Engine(std::seed_seq& seed);

    Engine(const Engine& other);

    Engine(Engine&& other);

    Engine& operator=(const Engine& other);

    Engine& operator=(Engine&& other);

    uint8_t get_random_uint8();

    uint32_t get_random_uint32();

    uint64_t get_random_uint64();

    uint128_t get_random_uint128();

    uint256_t get_random_uint256();

    uint512_t get_random_uint512();

    uint1024_t get_random_uint1024();

    std::mt19937_64 engine;
    std::uniform_int_distribution<uint64_t> dist = std::uniform_int_distribution<uint64_t>{ 0ULL, UINT64_MAX };
};

Engine& get_debug_engine();
Engine& get_engine();

} // namespace random
} // namespace numeric