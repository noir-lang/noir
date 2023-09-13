#pragma once
#include <atomic>
#include <barretenberg/env/hardware_concurrency.hpp>
#include <barretenberg/numeric/bitop/get_msb.hpp>
#include <functional>
#include <thread>
#include <vector>

inline size_t get_num_cpus()
{
#ifdef NO_MULTITHREADING
    return 1;
#else
    return env_hardware_concurrency();
#endif
}

// For algorithms that need to be divided amongst power of 2 threads.
inline size_t get_num_cpus_pow2()
{
    return static_cast<size_t>(1ULL << numeric::get_msb(get_num_cpus()));
}

void parallel_for(size_t num_iterations, const std::function<void(size_t)>& func);