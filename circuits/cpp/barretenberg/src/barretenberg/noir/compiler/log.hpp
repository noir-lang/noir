#pragma once
#include "format.hpp"

namespace noir {
namespace code_gen {

#if 0
template <typename... Args> inline void debug(std::string const& str, Args... args)
{
    std::cout << format(str, args...) << std::endl;
}
#else
template <typename... Args> inline void debug(std::string const&, Args...) {}
#endif

template <typename... Args> inline void info(std::string const& str, Args... args)
{
    std::cout << format(str, args...) << std::endl;
}

template <typename... Args> inline void abort[[noreturn]](std::string const& str, Args... args)
{
    std::cout << format(str, args...) << std::endl;
    std::abort();
}

} // namespace code_gen
} // namespace noir