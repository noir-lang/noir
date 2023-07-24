#pragma once
#include <barretenberg/common/log.hpp>

extern bool verbose;

template <typename... Args> inline void vinfo(Args... args)
{
    if (verbose) {
        info(args...);
    }
}