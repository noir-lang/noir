#pragma once
#include "log.hpp"
#include <string>

inline void throw_or_abort [[noreturn]] (std::string const& err)
{
#ifndef __wasm__
    throw std::runtime_error(err);
#else
    info("abort: ", err);
    std::abort();
#endif
}