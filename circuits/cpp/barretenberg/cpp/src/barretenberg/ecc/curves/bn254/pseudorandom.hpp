#pragma once
#include "fq.hpp"

inline barretenberg::fq get_pseudorandom_fq()
{
    static std::seed_seq seq{ 1, 2, 3, 4, 5, 6, 7, 8 };
    static std::mt19937_64 engine = std::mt19937_64(seq);
    static std::uniform_int_distribution<uint64_t> dist{ 0ULL, UINT64_MAX };

    barretenberg::fq out{
        (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine)
    };
    out.self_reduce_once();
    out.self_reduce_once();
    out.self_reduce_once();
    out.self_reduce_once();
    return out;
}