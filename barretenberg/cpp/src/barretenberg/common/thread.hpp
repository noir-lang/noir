#pragma once
#include <atomic>
#include <barretenberg/env/hardware_concurrency.hpp>
#include <barretenberg/numeric/bitop/get_msb.hpp>
#include <functional>
#include <iostream>
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
void run_loop_in_parallel(size_t num_points,
                          const std::function<void(size_t, size_t)>& func,
                          size_t no_multhreading_if_less_or_equal = 0);
void run_loop_in_parallel_if_effective(size_t num_points,
                                       const std::function<void(size_t, size_t)>& func,
                                       size_t finite_field_additions_per_iteration = 0,
                                       size_t finite_field_multiplications_per_iteration = 0,
                                       size_t finite_field_inversions_per_iteration = 0,
                                       size_t group_element_additions_per_iteration = 0,
                                       size_t group_element_doublings_per_iteration = 0,
                                       size_t scalar_multiplications_per_iteration = 0,
                                       size_t sequential_copy_ops_per_iteration = 0);