#pragma once
#include "uint256.hpp"
#include <random>

inline uint256_t get_pseudorandom_uint256()
{
    static std::seed_seq seq{ 1, 2, 3, 4, 5, 6, 7, 8 };
    static std::mt19937_64 engine = std::mt19937_64(seq);
    static std::uniform_int_distribution<uint64_t> dist{ 0ULL, UINT64_MAX };
    return { (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine) };
}