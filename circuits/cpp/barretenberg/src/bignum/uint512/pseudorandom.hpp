#pragma once
#include "../uint256/pseudorandom.hpp"
#include "uint512.hpp"

inline uint512_t get_pseudorandom_uint512()
{
    return { get_pseudorandom_uint256(), get_pseudorandom_uint256() };
}