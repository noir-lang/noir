#pragma once
#include <sstream>

extern "C" {
#ifdef __wasm__
void logstr(char const*);
#else
#include <iostream>
inline void logstr(char const* str)
{
    std::cout << str << std::endl;
}
#endif
}

namespace {
inline void format_chain(std::ostream&) {}

template <typename T> void format_chain(std::ostream& os, T const& first)
{
    os << first;
}

template <typename T, typename... Args> void format_chain(std::ostream& os, T const& first, Args const&... args)
{
    os << first;
    format_chain(os, args...);
}

template <typename... Args> std::string format(Args... args)
{
    std::ostringstream os;
    format_chain(os, args...);
    return os.str();
}
}

#if NDEBUG
template <typename... Args> inline void debug(Args... args)
{
    logstr(format(args...).c_str());
}
#else
template <typename... Args> inline void debug(Args...) {}
#endif

template <typename... Args> inline void info(Args... args)
{
    logstr(format(args...).c_str());
}