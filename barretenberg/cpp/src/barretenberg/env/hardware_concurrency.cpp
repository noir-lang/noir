#include "hardware_concurrency.hpp"
#include <barretenberg/common/throw_or_abort.hpp>
#include <cstdlib>
#include <iostream>
#include <stdexcept>
#include <string>

#ifndef NO_MULTITHREADING
#include <thread>
#endif

extern "C" {

#ifdef NO_MULTITHREADING
uint32_t env_hardware_concurrency()
{
    return 1;
}
#else
uint32_t env_hardware_concurrency()
{
#ifndef __wasm__
    try {
#endif
        static auto val = std::getenv("HARDWARE_CONCURRENCY");
        static const uint32_t cores = val ? (uint32_t)std::stoul(val) : std::thread::hardware_concurrency();
        return cores;
#ifndef __wasm__
    } catch (std::exception const&) {
        throw std::runtime_error("HARDWARE_CONCURRENCY invalid.");
    }
#endif
}
#endif
}