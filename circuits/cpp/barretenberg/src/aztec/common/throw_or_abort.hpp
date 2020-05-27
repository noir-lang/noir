#pragma once
#include <string>

inline void throw_or_abort [[noreturn]] (std::string const& err)
{
#ifndef __wasm__
    throw std::runtime_error(err);
#else
    std::cout << err << std::endl;
    std::abort();
#endif
}
