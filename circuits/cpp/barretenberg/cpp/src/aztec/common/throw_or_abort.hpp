#pragma once
#include <string>
#include "log.hpp"

inline void throw_or_abort [[noreturn]] (std::string const& err)
{
#ifndef __wasm__
    throw std::runtime_error(err);
#else
    info(err);
    std::abort();
#endif
}
