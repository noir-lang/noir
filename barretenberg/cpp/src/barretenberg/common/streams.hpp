#pragma once
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/serialize/msgpack_apply.hpp"
#include <iomanip>
#include <map>
#include <memory>
#include <optional>
#include <ostream>
#include <vector>

// clang-format off
// disabling the following style guides:
// cert-dcl58-cpp , restricts modifying the standard library. We need to do this for portable serialization methods
// NOLINTBEGIN(cert-dcl58-cpp)
// clang-format on

namespace serialize {
/**
 * @brief Helper method for streaming msgpack values, specialized for shared_ptr
 */
template <typename T> void _msgpack_stream_write(std::ostream& os, const std::shared_ptr<T>& field)
{
    using namespace serialize;
    os << *field;
}
/**
 * @brief Helper method for streaming msgpack values, normal case
 */
inline void _msgpack_stream_write(std::ostream& os, const auto& field)
{
    using namespace serialize;
    os << field;
}
/**
 * @brief Recursive helper method for streaming msgpack key value pairs, base case
 */
inline void _msgpack_stream_write_key_value_pairs(std::ostream& os)
{
    // base case
    (void)os; // unused
}
/**
 * @brief Recursive helper method for streaming msgpack key value pairs, default arg case
 */
inline void _msgpack_stream_write_key_value_pairs(std::ostream& os,
                                                  const std::string& key,
                                                  const auto& value,
                                                  const auto&... rest)
{
    os << key << ": ";
    _msgpack_stream_write(os, value);
    os << '\n';
    _msgpack_stream_write_key_value_pairs(os, rest...); // NOLINT
}
/**
 * @brief Recursive helper method for streaming msgpack key value pairs, msgpack arg case
 * We add a new line as this was the previous output captured in snapshot tests.
 * TODO(AD): Ideally some tab indenting?
 */
inline void _msgpack_stream_write_key_value_pairs(std::ostream& os,
                                                  const std::string& key,
                                                  const msgpack_concepts::HasMsgPack auto& value,
                                                  const auto&... rest)
{
    os << key << ":\n";
    _msgpack_stream_write(os, value);
    os << '\n';
    _msgpack_stream_write_key_value_pairs(os, rest...); // NOLINT
}
} // namespace serialize

namespace std {
/**
 * @brief Automatically derived stream operator for any object that defines .msgpack() (implicitly defined by
 * MSGPACK_FIELDS). Note this is duplicated as it must be seen in both std and global namespaces.
 * @param os The stream to write to.
 * @param obj The object to write.
 */
template <msgpack_concepts::HasMsgPack T> std::ostream& operator<<(std::ostream& os, const T& obj)
{
    // We must use const_cast as our method is meant to be polymorphic over const, but there's no such concept in C++
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    const_cast<T&>(obj).msgpack([&](auto&... key_value_pairs) {
        // apply 'operator<<' to each object field
        serialize::_msgpack_stream_write_key_value_pairs(os, key_value_pairs...);
    });
    return os;
}

inline std::ostream& operator<<(std::ostream& os, std::vector<uint8_t> const& arr)
{
    std::ios_base::fmtflags f(os.flags());
    os << "[" << std::hex << std::setfill('0');
    for (auto byte : arr) {
        os << ' ' << std::setw(2) << +static_cast<unsigned char>(byte);
    }
    os << " ]";
    os.flags(f);
    return os;
}

template <std::integral T, typename A> inline std::ostream& operator<<(std::ostream& os, std::vector<T, A> const& arr)
{
    os << "[";
    for (auto element : arr) {
        os << ' ' << element;
    }
    os << " ]";
    return os;
}

template <typename T, typename A>
    requires(!std::integral<T>)
inline std::ostream& operator<<(std::ostream& os, std::vector<T, A> const& arr)
{
    os << "[\n";
    for (auto element : arr) {
        os << ' ' << element << '\n';
    }
    os << "]";
    return os;
}

template <size_t S> inline std::ostream& operator<<(std::ostream& os, std::array<uint8_t, S> const& arr)
{
    std::ios_base::fmtflags f(os.flags());
    os << "[" << std::hex << std::setfill('0');
    for (auto byte : arr) {
        os << ' ' << std::setw(2) << +static_cast<unsigned char>(byte);
    }
    os << " ]";
    os.flags(f);
    return os;
}

template <typename T, size_t S> inline std::ostream& operator<<(std::ostream& os, std::array<T, S> const& arr)
{
    std::ios_base::fmtflags f(os.flags());
    os << "[" << std::hex << std::setfill('0');
    for (auto element : arr) {
        os << ' ' << element;
    }
    os << " ]";
    os.flags(f);
    return os;
}

template <typename T, typename U> inline std::ostream& operator<<(std::ostream& os, std::pair<T, U> const& pair)
{
    os << "(" << pair.first << ", " << pair.second << ")";
    return os;
}

template <typename T> inline std::ostream& operator<<(std::ostream& os, std::optional<T> const& opt)
{
    return opt ? os << *opt : os << "std::nullopt";
}

template <typename T, typename U> inline std::ostream& operator<<(std::ostream& os, std::map<T, U> const& map)
{
    os << "[\n";
    for (const auto& elem : map) {
        os << " " << elem.first << ": " << elem.second << "\n";
    }
    os << "]";
    return os;
}
} // namespace std

// NOLINTEND(cert-dcl58-cpp)
