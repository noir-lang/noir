#pragma once
#include <boost/format.hpp>
#include <iostream>

namespace noir {
namespace code_gen {

namespace {
inline void format_chain(boost::format&) {}

template <typename T> void format_chain(boost::format& fmt, T const& first)
{
    fmt % first;
}

template <typename T, typename... Args> void format_chain(boost::format& fmt, T const& first, Args const&... args)
{
    fmt % first;
    format_chain(fmt, args...);
}
} // namespace

template <typename... Args> std::string format(std::string const& str, Args... args)
{
    auto fmt = boost::format(str);
    format_chain(fmt, args...);
    return fmt.str();
}

} // namespace code_gen
} // namespace noir