#pragma once
#include "format.hpp"
#include "throw_or_abort.hpp"

namespace noir {

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

template <typename... Args> inline void abort [[noreturn]] (std::string const& str, Args... args)
{
    throw_or_abort(format(str, args...));
}

} // namespace noir