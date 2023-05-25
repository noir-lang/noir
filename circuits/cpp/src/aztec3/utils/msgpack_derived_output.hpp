#pragma once

#include <barretenberg/serialize/msgpack.hpp>

#include <iostream>
#include <string>

namespace aztec3::utils {

template <typename T> std::ostream& msgpack_derived_output(std::ostream& os, const T& value);

inline void _msgpack_derived_output_helper(std::ostream& os)
{
    // base case
    (void)os;  // unused
}
inline void _msgpack_derived_output_helper(std::ostream& os,
                                           const std::string& key,
                                           const auto& value,
                                           const auto&... rest)
{
    os << key << ": ";
    msgpack_derived_output(os, value);
    os << '\n';
    _msgpack_derived_output_helper(os, rest...);  // NOLINT
}
// Specialization if we have msgpack
template <msgpack_concepts::HasMsgPack T> void msgpack_derived_output(std::ostream& os, const T& value)
{
    const_cast<T&>(value).msgpack([&](auto&... args) { _msgpack_derived_output_helper(os, args...); });  // NOLINT
}

// Otherwise
template <typename T> std::ostream& msgpack_derived_output(std::ostream& os, const T& value)
{
    return os << value;
}
}  // namespace aztec3::utils
