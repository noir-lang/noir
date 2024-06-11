/**
 * This is a non-msgpack flat buffer serialization library.
 * It is currently used alongside msgpack, with hope to eventually move to msgpack.
 * It enables the reading and writing of big-endian formatted integers and various standard library types
 * to and from the following supported types:
 *  - uint8_t*
 *  - std::vector<uint8_t>
 *  - std::ostream / std::istream
 *
 * To support custom types, free functions taking the following form should be defined alongside the custom type:
 *  - template <typename B> inline void read(B& it, my_custom_type& value)
 *  - template <typename B> inline void write(B& it, my_custom_type const& value)
 * They should be implemented in terms of lower level read/write functions.
 * Be aware that if B is a uint8_t*, it will be advanced appropriately during reads and writes.
 *
 * For understanding, given integers do not belong in any namespace, they have been defined inside the serialize
 * namespace. It may sometimes be necessary to specify a `using serialize::read` or `using serialize::write` to
 * find them. This is prefereable to polluting the global namespace which comes with its own issues.
 *
 * Standard library types are defined inside the `std` namespace, so they can be discovered using argument dependent
 * lookup. Placing them inside the serialize namespace was an option, but would mean integers and std types would need
 * to have the serialize namespace specified, but custom types would not. By leveraging ADL we can avoid needing
 * to specify the serialize namespace in almost all cases.
 *
 * A few helpers are defined at global namespace:
 *  - from_buffer<T>
 *  - many_from_buffer<T>
 *  - to_buffer
 */
#pragma once
#include "barretenberg/common/log.hpp"
#include "barretenberg/common/net.hpp"
#include "barretenberg/serialize/msgpack_apply.hpp"
#include <array>
#include <cassert>
#include <iostream>
#include <map>
#include <memory>
#include <optional>
#include <type_traits>
#include <vector>

#ifndef __i386__
__extension__ using uint128_t = unsigned __int128;
#endif

// clang-format off
// disabling the following style guides:
// cert-dcl58-cpp , restricts modifying the standard library. We need to do this for portable serialization methods
// cppcoreguidelines-pro-type-reinterpret-cast, we heavily use reinterpret-cast and would be difficult to refactor this out
// NOLINTBEGIN(cppcoreguidelines-pro-type-reinterpret-cast, cert-dcl58-cpp)
// clang-format on

template <typename T>
concept IntegralOrEnum = std::integral<T> || std::is_enum_v<T>;

namespace serialize {
// Forward declare derived msgpack methods
void read(auto& it, msgpack_concepts::HasMsgPack auto& obj);
void write(auto& buf, const msgpack_concepts::HasMsgPack auto& obj);

// Basic integer read / write, to / from raw buffers.
// Pointers to buffers are advanced by length of type.
inline void read(uint8_t const*& it, uint8_t& value)
{
    value = *it;
    it += 1;
}

inline void write(uint8_t*& it, uint8_t value)
{
    *it = value;
    it += 1;
}

inline void read(uint8_t const*& it, bool& value)
{
    value = (*it != 0U);
    it += 1;
}

inline void write(uint8_t*& it, bool value)
{
    *it = static_cast<uint8_t>(value);
    it += 1;
}

inline void read(uint8_t const*& it, uint16_t& value)
{
    value = ntohs(*reinterpret_cast<uint16_t const*>(it)); // NOLINT
    it += 2;
}

inline void write(uint8_t*& it, uint16_t value)
{
    *reinterpret_cast<uint16_t*>(it) = htons(value); // NOLINT
    it += 2;
}

inline void read(uint8_t const*& it, uint32_t& value)
{
    value = ntohl(*reinterpret_cast<uint32_t const*>(it)); // NOLINT
    it += 4;
}

inline void write(uint8_t*& it, uint32_t value)
{
    *reinterpret_cast<uint32_t*>(it) = htonl(value);
    it += 4;
}

inline void read(uint8_t const*& it, uint64_t& value)
{
    value = ntohll(*reinterpret_cast<uint64_t const*>(it));
    it += 8;
}

inline void write(uint8_t*& it, uint64_t value)
{
    *reinterpret_cast<uint64_t*>(it) = htonll(value);
    it += 8;
}

#ifdef __APPLE__
inline void read(uint8_t const*& it, unsigned long& value)
{
    value = ntohll(*reinterpret_cast<unsigned long const*>(it));
    it += 8;
}

inline void write(uint8_t*& it, unsigned long value)
{
    *reinterpret_cast<unsigned long*>(it) = htonll(value);
    it += 8;
}
#endif

#ifndef __i386__
inline void read(uint8_t const*& it, uint128_t& value)
{
    uint64_t hi, lo; // NOLINT
    read(it, hi);
    read(it, lo);
    value = (static_cast<uint128_t>(hi) << 64) | lo;
}
inline void write(uint8_t*& it, uint128_t value)
{
    auto hi = static_cast<uint64_t>(value >> 64);
    auto lo = static_cast<uint64_t>(value);
    write(it, hi);
    write(it, lo);
}
#endif

// Reading / writing integer types to / from vectors.
void read(std::vector<uint8_t> const& buf, std::integral auto& value)
{
    const auto* ptr = &buf[0];
    read(ptr, value);
}

void write(std::vector<uint8_t>& buf, const std::integral auto& value)
{
    buf.resize(buf.size() + sizeof(value));
    uint8_t* ptr = &*buf.end() - sizeof(value);
    write(ptr, value);
}

// Reading writing integer types to / from streams.
void read(std::istream& is, std::integral auto& value)
{
    std::array<uint8_t, sizeof(value)> buf;
    is.read(reinterpret_cast<char*>(buf.data()), sizeof(value));
    uint8_t const* ptr = &buf[0];
    read(ptr, value);
}

void write(std::ostream& os, const std::integral auto& value)
{
    std::array<uint8_t, sizeof(value)> buf;
    uint8_t* ptr = &buf[0];
    write(ptr, value);
    os.write(reinterpret_cast<char*>(buf.data()), sizeof(value));
}
} // namespace serialize

namespace std {
inline void read(auto& buf, std::integral auto& value)
{
    serialize::read(buf, value);
}

inline void write(auto& buf, std::integral auto value)
{
    serialize::write(buf, value);
}

// Optimized specialisation for reading arrays of bytes from a raw buffer.
template <size_t N> inline void read(uint8_t const*& it, std::array<uint8_t, N>& value)
{
    std::copy(it, it + N, value.data());
    it += N;
}

// Optimized specialisation for writing arrays of bytes to a raw buffer.
template <size_t N> inline void write(uint8_t*& buf, std::array<uint8_t, N> const& value)
{
    std::copy(value.begin(), value.end(), buf);
    buf += N;
}

// Optimized specialisation for reading vectors of bytes from a raw buffer.
inline void read(uint8_t const*& it, std::vector<uint8_t>& value)
{
    uint32_t size = 0;
    read(it, size);
    value.resize(size);
    std::copy(it, it + size, value.data());
    it += size;
}

// Optimized specialisation for writing vectors of bytes to a raw buffer.
inline void write(uint8_t*& buf, std::vector<uint8_t> const& value)
{
    write(buf, static_cast<uint32_t>(value.size()));
    std::copy(value.begin(), value.end(), buf);
    buf += value.size();
}

// Optimized specialisation for reading vectors of bytes from an input stream.
inline void read(std::istream& is, std::vector<uint8_t>& value)
{
    uint32_t size = 0;
    read(is, size);
    value.resize(size);
    is.read(reinterpret_cast<char*>(value.data()), static_cast<std::streamsize>(size));
}

// Optimized specialisation for writing vectors of bytes to an output stream.
inline void write(std::ostream& os, std::vector<uint8_t> const& value)
{
    write(os, static_cast<uint32_t>(value.size()));
    os.write(reinterpret_cast<const char*>(value.data()), static_cast<std::streamsize>(value.size()));
}

// Optimized specialisation for writing arrays of bytes to a vector.
template <size_t N> inline void write(std::vector<uint8_t>& buf, std::array<uint8_t, N> const& value)
{
    buf.resize(buf.size() + N);
    auto* ptr = &*buf.end() - N;
    write(ptr, value);
}

// Optimized specialisation for writing arrays of bytes to an output stream.
template <size_t N> inline void write(std::ostream& os, std::array<uint8_t, N> const& value)
{
    os.write(reinterpret_cast<char*>(value.data()), value.size());
}

// Generic read of array of types from supported buffer types.
template <typename B, typename T, size_t N> inline void read(B& it, std::array<T, N>& value)
{
    using serialize::read;
    for (size_t i = 0; i < N; ++i) {
        read(it, value[i]);
    }
}

// Generic write of array of types to supported buffer types.
template <typename B, typename T, size_t N> inline void write(B& buf, std::array<T, N> const& value)
{
    using serialize::write;
    for (size_t i = 0; i < N; ++i) {
        write(buf, value[i]);
    }
}

// Generic read of vector of types from supported buffer types.
template <typename B, typename T, typename A> inline void read(B& it, std::vector<T, A>& value)
{
    using serialize::read;
    uint32_t size = 0;
    read(it, size);
    value.resize(size);
    for (size_t i = 0; i < size; ++i) {
        read(it, value[i]);
    }
}

// Generic write of vector of types to supported buffer types.
template <typename B, typename T, typename A> inline void write(B& buf, std::vector<T, A> const& value)
{
    using serialize::write;
    write(buf, static_cast<uint32_t>(value.size()));
    for (size_t i = 0; i < value.size(); ++i) {
        write(buf, value[i]);
    }
}

// Read string from supported buffer types.
template <typename B> inline void read(B& it, std::string& value)
{
    using serialize::read;
    std::vector<uint8_t> buf;
    read(it, buf);
    value = std::string(buf.begin(), buf.end());
}

// Write of strings to supported buffer types.
template <typename B> inline void write(B& buf, std::string const& value)
{
    using serialize::write;
    write(buf, std::vector<uint8_t>(value.begin(), value.end()));
}

// Read std::pair.
template <typename B, typename T, typename U> inline void read(B& it, std::pair<T, U>& value)
{
    using serialize::read;
    read(it, value.first);
    read(it, value.second);
}

// Write std::pair.
template <typename B, typename T, typename U> inline void write(B& buf, std::pair<T, U> const& value)
{
    using serialize::write;
    write(buf, value.first);
    write(buf, value.second);
}

// Read std::shared_ptr.
template <typename B, typename T> inline void read(B& it, std::shared_ptr<T>& value_ptr)
{
    using serialize::read;
    T value;
    read(it, value);
    value_ptr = std::make_shared<T>(value);
}

// Write std::shared_ptr.
template <typename B, typename T> inline void write(B& buf, std::shared_ptr<T> const& value_ptr)
{
    using serialize::write;
    write(buf, *value_ptr);
}

// Read std::map
template <typename B, typename T, typename U> inline void read(B& it, std::map<T, U>& value)
{
    using serialize::read;
    value.clear();
    uint32_t size = 0;
    read(it, size);
    for (size_t i = 0; i < size; ++i) {
        std::pair<T, U> v;
        read(it, v);
        value.emplace(std::move(v));
    }
}

// Write std::map.
template <typename B, typename T, typename U> inline void write(B& buf, std::map<T, U> const& value)
{
    using serialize::write;
    write(buf, static_cast<uint32_t>(value.size()));
    for (auto const& kv : value) {
        write(buf, kv);
    }
}

// Read std::optional<T>.
template <typename B, typename T> inline void read(B& it, std::optional<T>& opt_value)
{
    using serialize::read;
    bool is_nullopt = false;
    read(it, is_nullopt);
    if (is_nullopt) {
        opt_value = std::nullopt;
        return;
    }
    T value;
    read(it, value);
    opt_value = T(value);
}

template <typename T>
concept HasGetAll = requires(T t) { t.get_all(); } && !msgpack_concepts::HasMsgPack<T>;

// Write out a struct that defines get_all()
template <typename B, HasGetAll T> inline void write(B& buf, T const& value)
{
    using serialize::write;
    for (auto& reference : value.get_all()) {
        write(buf, reference);
    }
}

// Write std::optional<T>.
// Note: It takes up a different amount of space, depending on whether it's std::nullopt or populated with an actual
// value.
template <typename B, typename T> inline void write(B& buf, std::optional<T> const& opt_value)
{
    using serialize::write;
    if (opt_value) {
        write(buf, false); // is not nullopt
        write(buf, *opt_value);
        return;
    }
    write(buf, true); // is nullopt
}

} // namespace std

// Helper functions that have return values.
template <typename T, typename B> T from_buffer(B const& buffer, size_t offset = 0)
{
    using serialize::read;
    T result;
    const auto* ptr = reinterpret_cast<uint8_t const*>(&buffer[offset]);
    read(ptr, result);
    return result;
}

template <typename T> std::vector<uint8_t> to_buffer(T const& value)
{
    using serialize::write;
    std::vector<uint8_t> buf;
    write(buf, value);
    return buf;
}

/**
 * Serializes the given value, such that it is byte length prefixed, for deserialization on the calling side.
 * This is used for variable length outputs, whereby the caller needs to discover the length so the memory can be
 * appropriately sliced.
 * It can result in the (expected) oddity of e.g. a vector<uint8_t> being "multiple prefixed":
 * e.g. [heap buffer length][value length][value bytes].
 */
template <typename T> uint8_t* to_heap_buffer(T const& value)
{
    using serialize::write;

    // Initial serialization of the value. Creates a vector of bytes.
    auto buf = to_buffer(value);

    // Serialize this byte vector, giving us a length prefixed buffer of bytes.
    auto heap_buf = to_buffer(buf);

    auto* ptr = (uint8_t*)aligned_alloc(64, heap_buf.size()); // NOLINT
    std::copy(heap_buf.begin(), heap_buf.end(), ptr);
    return ptr;
}

template <typename T> std::vector<T> many_from_buffer(std::vector<uint8_t> const& buffer)
{
    const size_t num_elements = buffer.size() / sizeof(T);
    std::vector<T> elements;
    for (size_t i = 0; i < num_elements; ++i) {
        elements.push_back(from_buffer<T>(buffer, i * sizeof(T)));
    }
    return elements;
}

// By default, if calling to_buffer on a vector of types, we don't prefix the vector size.
template <bool include_size = false, typename T> std::vector<uint8_t> to_buffer(std::vector<T> const& value)
{
    using serialize::write;
    std::vector<uint8_t> buf;
    if (include_size) {
        write(buf, value);
    } else {
        for (auto e : value) {
            write(buf, e);
        }
    }
    return buf;
}

// Some types to describe fixed size buffers for c_bind arguments.
using in_buf32 = uint8_t const*;
using out_buf32 = uint8_t*;
using in_buf64 = uint8_t const*;
using out_buf64 = uint8_t*;
using in_buf128 = uint8_t const*;
using out_buf128 = uint8_t*;

// Variable length string buffers. Prefixed with length.
using in_str_buf = uint8_t const*;
using out_str_buf = uint8_t**;

// Use these to pass a raw memory pointer.
using in_ptr = void* const*;
using out_ptr = void**;

namespace serialize {

/**
 * @brief Helper method for better error reporting. Clang does not give the best errors for "auto..."
 * arguments.
 */
inline void _read_msgpack_field(auto& it, auto& field)
{
    using namespace serialize;
    read(it, field);
}

/**
 * @brief Automatically derived read for any object that defines .msgpack() (implicitly defined by MSGPACK_FIELDS).
 * @param it The iterator to read from.
 * @param func The function to call with each field as an argument.
 */
inline void read(auto& it, msgpack_concepts::HasMsgPack auto& obj)
{
    msgpack::msgpack_apply(obj, [&](auto&... obj_fields) {
        // apply 'read' to each object field
        (_read_msgpack_field(it, obj_fields), ...);
    });
};

/**
 * @brief Helper method for better error reporting. Clang does not give the best errors for "auto..."
 * arguments.
 */
inline void _write_msgpack_field(auto& it, const auto& field)
{
    using namespace serialize;
    write(it, field);
}
/**
 * @brief Automatically derived write for any object that defines .msgpack() (implicitly defined by MSGPACK_FIELDS).
 * @param buf The buffer to write to.
 * @param obj The object to write.
 */
inline void write(auto& buf, const msgpack_concepts::HasMsgPack auto& obj)
{
    msgpack::msgpack_apply(obj, [&](auto&... obj_fields) {
        // apply 'write' to each object field
        (_write_msgpack_field(buf, obj_fields), ...);
    });
}
} // namespace serialize
// clang-format off
// NOLINTEND(cppcoreguidelines-pro-type-reinterpret-cast, cert-dcl58-cpp)
// clang-format on
