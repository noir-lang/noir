#pragma once
#include <array>
#include <common/net.hpp>
#include <type_traits>
#include <vector>
#include <map>
#include <iostream>

__extension__ using uint128_t = unsigned __int128;

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
    value = true;
    it += 1;
}

inline void write(uint8_t*& it, bool value)
{
    *it = static_cast<uint8_t>(value);
    it += 1;
}

inline void read(uint8_t const*& it, uint16_t& value)
{
    value = ntohs(*reinterpret_cast<uint16_t const*>(it));
    it += 2;
}

inline void write(uint8_t*& it, uint16_t value)
{
    *reinterpret_cast<uint16_t*>(it) = htons(value);
    it += 2;
}

inline void read(uint8_t const*& it, uint32_t& value)
{
    value = ntohl(*reinterpret_cast<uint32_t const*>(it));
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

inline void read(uint8_t const*& it, uint128_t& value)
{
    uint64_t hi, lo;
    read(it, hi);
    read(it, lo);
    value = (static_cast<uint128_t>(hi) << 64) | lo;
}

inline void write(uint8_t*& it, uint128_t value)
{
    uint64_t hi = static_cast<uint64_t>(value >> 64);
    uint64_t lo = static_cast<uint64_t>(value);
    write(it, hi);
    write(it, lo);
}

// Reading / writing integer types to / from vectors.
template <typename T> inline std::enable_if_t<std::is_integral_v<T>> read(std::vector<uint8_t> const& buf, T& value)
{
    auto ptr = &buf[0];
    ::read(ptr, value);
}

template <typename T> inline std::enable_if_t<std::is_integral_v<T>> write(std::vector<uint8_t>& buf, T value)
{
    buf.resize(buf.size() + sizeof(T));
    uint8_t* ptr = &*buf.end() - sizeof(T);
    ::write(ptr, value);
}

// Reading writing integer types to / from streams.
template <typename T> inline std::enable_if_t<std::is_integral_v<T>> read(std::istream& is, T& value)
{
    std::array<uint8_t, sizeof(T)> buf;
    is.read((char*)buf.data(), sizeof(T));
    uint8_t const* ptr = &buf[0];
    ::read(ptr, value);
}

template <typename T> inline std::enable_if_t<std::is_integral_v<T>> write(std::ostream& os, T value)
{
    std::array<uint8_t, sizeof(T)> buf;
    uint8_t* ptr = &buf[0];
    ::write(ptr, value);
    os.write((char*)buf.data(), sizeof(T));
}

namespace std {

// Forwarding functions from std to global namespace for integers.
template <typename B, typename T> inline std::enable_if_t<std::is_integral_v<T>> read(B& buf, T& value)
{
    ::read(buf, value);
}

template <typename B, typename T> inline std::enable_if_t<std::is_integral_v<T>> write(B& buf, T value)
{
    ::write(buf, value);
}

// Optimised specialisation for reading arrays of bytes from a raw buffer.
template <size_t N> inline void read(uint8_t const*& it, std::array<uint8_t, N>& value)
{
    std::copy(it, it + N, value.data());
    it += N;
}

// Optimised specialisation for writing arrays of bytes to a raw buffer.
template <size_t N> inline void write(uint8_t*& buf, std::array<uint8_t, N> const& value)
{
    std::copy(value.begin(), value.end(), buf);
    buf += N;
}

// Optimised specialisation for reading vectors of bytes from a raw buffer.
inline void read(uint8_t const*& it, std::vector<uint8_t>& value)
{
    uint32_t size;
    read(it, size);
    value.resize(size);
    std::copy(it, it + size, value.data());
    it += size;
}

// Optimised specialisation for writing vectors of bytes to a raw buffer.
inline void write(uint8_t*& buf, std::vector<uint8_t> const& value)
{
    write(buf, static_cast<uint32_t>(value.size()));
    std::copy(value.begin(), value.end(), buf);
    buf += value.size();
}

// Optimised specialisation for reading vectors of bytes from an input stream.
inline void read(std::istream& is, std::vector<uint8_t>& value)
{
    uint32_t size;
    read(is, size);
    value.resize(size);
    is.read((char*)value.data(), (std::streamsize)size);
}

// Optimised specialisation for writing vectors of bytes to an output stream.
inline void write(std::ostream& os, std::vector<uint8_t> const& value)
{
    write(os, static_cast<uint32_t>(value.size()));
    os.write((char*)value.data(), (std::streamsize)value.size());
}

// Optimised specialisation for writing arrays of bytes to a vector.
template <size_t N> inline void write(std::vector<uint8_t>& buf, std::array<uint8_t, N> const& value)
{
    buf.resize(buf.size() + N);
    auto ptr = &*buf.end() - N;
    write(ptr, value);
}

// Optimised specialisation for writing arrays of bytes to an output stream.
template <size_t N> inline void write(std::ostream& os, std::array<uint8_t, N> const& value)
{
    os.write((char*)value.data(), value.size());
}

// Generic read of array of types from supported buffer types.
template <typename B, typename T, size_t N> inline void read(B& it, std::array<T, N>& value)
{
    for (size_t i = 0; i < N; ++i) {
        read(it, value[i]);
    }
}

// Generic write of array of types to supported buffer types.
template <typename B, typename T, size_t N> inline void write(B& buf, std::array<T, N> const& value)
{
    for (size_t i = 0; i < N; ++i) {
        write(buf, value[i]);
    }
}

// Generic read of vector of types from supported buffer types.
template <typename B, typename T> inline void read(B& it, std::vector<T>& value)
{
    uint32_t size;
    read(it, size);
    value.resize(size);
    for (size_t i = 0; i < size; ++i) {
        read(it, value[i]);
    }
}

// Generic write of vector of types to supported buffer types.
template <typename B, typename T> inline void write(B& buf, std::vector<T> const& value)
{
    write(buf, static_cast<uint32_t>(value.size()));
    for (size_t i = 0; i < value.size(); ++i) {
        write(buf, value[i]);
    }
}

// Read string from supported buffer types.
template <typename B> inline void read(B& it, std::string& value)
{
    std::vector<uint8_t> buf;
    read(it, buf);
    value = std::string(buf.begin(), buf.end());
}

// Write of strings to supported buffer types.
template <typename B> inline void write(B& buf, std::string const& value)
{
    write(buf, std::vector<uint8_t>(value.begin(), value.end()));
}

// Read std::pair.
template <typename B, typename T, typename U> inline void read(B& it, std::pair<T, U>& value)
{
    read(it, value.first);
    read(it, value.second);
}

// Write std::pair.
template <typename B, typename T, typename U> inline void write(B& buf, std::pair<T, U> const& value)
{
    write(buf, value.first);
    write(buf, value.second);
}

// Read std::map
template <typename B, typename T, typename U> inline void read(B& it, std::map<T, U>& value)
{
    value.clear();
    uint32_t size;
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
    write(buf, static_cast<uint32_t>(value.size()));
    for (auto const& kv : value) {
        write(buf, kv);
    }
}

} // namespace std

// Helper functions that have return values.
template <typename T, typename B> T from_buffer(B const& buffer, size_t offset = 0)
{
    T result;
    auto ptr = (uint8_t const*)&buffer[offset];
    read(ptr, result);
    return result;
}

template <typename T> std::vector<uint8_t> to_buffer(T const& value)
{
    std::vector<uint8_t> buf;
    write(buf, value);
    return buf;
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
    std::vector<uint8_t> buf;
    for (auto e : value) {
        write(buf, e);
    }
    return buf;
}
