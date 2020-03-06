#pragma once

#include "stddef.h"
#include "stdint.h"
#include "stdlib.h"

#ifdef _WIN32
#include "./portability/win32.hpp"
#endif

#ifdef __linux__
#include "./portability/linux.hpp"
#endif

#ifdef __wasm__
#define aligned_free free
#endif

#ifdef __APPLE__
#include "./portability/apple.hpp"
#endif

#ifdef __wasm__
#define aligned_free free
#endif

#ifndef BARRETENBERG_SRS_PATH
#define BARRETENBERG_SRS_PATH "../srs_db"
#endif

#if 0
#define NO_MULTITHREADING 1
#endif

#if 0
#define USE_AVX 1
#endif

#ifndef PIPPENGER_BLOCK_SIZE
#define PIPPENGER_BLOCK_SIZE 20
#endif
// void foo(const evaluation_domain& domain)
// {
//     std::vector<std::thread> threads(domain.thread_size);
//     auto parallel_loop = [&](int start, int end)
//     {
//         for (size_t i = start; i < end; ++i)
//         {
//             // do work
//         }
//     }
//     for (size_t i = 0; i < num_threads; ++i)
//     {
//         const size_t start = i * domain.thread_size;
//         const size_t end = i * domain.thread_size;
//         threads[i] = std::thread(parallel_loop, start, end);
//     }
//     for (size_t i = 0; i < num_threads; ++i)
//     {
//         threads[i].join();
//     }
// }
#if 0
#include <thread>
#define ITERATE_OVER_DOMAIN_START(domain)                                                                              \
    {                                                                                                                  \
        const size_t __num_threads = domain.num_threads;                                                               \
        const size_t __thread_size = domain.thread_size;                                                               \
        std::vector<std::thread> threads(__num_threads);                                                               \
        auto parallel_loop = [&](size_t __start, size_t __end) { \
        for (size_t i = __start; i < __end; ++i) \
        {

#define ITERATE_OVER_DOMAIN_END                                                                                        \
    }                                                                                                                  \
    }                                                                                                                  \
    ;                                                                                                                  \
    for (size_t j = 0; j < __num_threads; ++j) {                                                                       \
        const size_t _start = j * __thread_size;                                                                       \
        const size_t _end = (j + 1) * __thread_size;                                                                   \
        threads[j] = std::thread(parallel_loop, _start, _end);                                                         \
    }                                                                                                                  \
    for (size_t j = 0; j < __num_threads; ++j) {                                                                       \
        threads[j].join();                                                                                             \
    }                                                                                                                  \
    }
#endif

// TODO: PUT SOMEWHERE NICE
// Some hacky macros that allow us to parallelize iterating over a polynomial's point-evaluations
#if 1
#ifndef NO_MULTITHREADING
#define ITERATE_OVER_DOMAIN_START(domain)                                                                              \
    _Pragma("omp parallel for") for (size_t j = 0; j < domain.num_threads; ++j)                                        \
    {                                                                                                                  \
        const size_t internal_bound_start = j * domain.thread_size;                                                    \
        const size_t internal_bound_end = (j + 1) * domain.thread_size;                                                \
        for (size_t i = internal_bound_start; i < internal_bound_end; ++i) {

#define ITERATE_OVER_DOMAIN_END                                                                                        \
    }                                                                                                                  \
    }
#else
#define ITERATE_OVER_DOMAIN_START(domain) for (size_t i = 0; i < domain.size; ++i) {

#define ITERATE_OVER_DOMAIN_END }
#endif
#endif