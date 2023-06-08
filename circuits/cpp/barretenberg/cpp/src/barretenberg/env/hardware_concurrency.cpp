#include "hardware_concurrency.hpp"
#include <thread>

extern "C" {

uint32_t env_hardware_concurrency()
{
    return std::thread::hardware_concurrency();
}
}