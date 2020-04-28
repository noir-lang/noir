#pragma once
#include <vector>
#include <type_traits>
#include <common/net.hpp>

inline void read(uint8_t const*& it, uint8_t& value) {
    value = *it;
    it += 1;
}

inline void write(uint8_t*& it, uint8_t value) {
    *it = value;
    it += 1;
}

inline void read(uint8_t const*& it, uint16_t& value) {
    value = ntohs(*reinterpret_cast<uint16_t const*>(it));
    it += 2;
}

inline void write(uint8_t*& it, uint16_t value) {
    *reinterpret_cast<uint16_t*>(it) = htons(value);
    it += 2;
}

inline void read(uint8_t const*& it, uint32_t& value) {
    value = ntohl(*reinterpret_cast<uint32_t const*>(it));
    it += 4;
}

inline void write(uint8_t*& it, uint32_t value) {
    *reinterpret_cast<uint32_t*>(it) = htonl(value);
    it += 4;
}

inline void read(uint8_t const*& it, uint64_t& value) {
    value = ntohll(*reinterpret_cast<uint64_t const*>(it));
    it += 8;
}

inline void write(uint8_t*& it, uint64_t value) {
    *reinterpret_cast<uint64_t*>(it) = htonll(value);
    it += 8;
}

template<typename T>
inline std::enable_if_t<std::is_integral_v<T>> read(std::vector<uint8_t> const& buf, T& value) {
    auto ptr = &buf[0];
    ::read(ptr, value);
}

template<typename T>
inline std::enable_if_t<std::is_integral_v<T>> write(std::vector<uint8_t>& buf, T value) {
    buf.resize(buf.size() + sizeof(T));
    uint8_t* ptr = &*buf.end() - sizeof(T);
    ::write(ptr, value);
}

template<typename T>
inline std::enable_if_t<std::is_integral_v<T>> read(std::istream& is, T& value) {
    std::array<uint8_t, sizeof(T)> buf;
    is.read((char*)buf.data(), sizeof(T));
    ::read(buf.data(), value);
}

template<typename T>
inline std::enable_if_t<std::is_integral_v<T>> write(std::ostream& os, T value) {
    std::array<uint8_t, sizeof(T)> buf;
    ::write(buf.data(), value);
    os.write(buf.data(), sizeof(T));
}

namespace std {
template<size_t N>
inline void read(uint8_t const*& it, std::array<uint8_t, N>& value) {
    std::copy(it, it+N, value.data());
    it += N;
}

template<size_t N>
inline void write(uint8_t*& buf, std::array<uint8_t, N> const& value) {
    std::copy(value.begin(), value.end(), buf);
    buf += N;
}

template<size_t N>
inline void write(std::vector<uint8_t>& buf, std::array<uint8_t, N> const& value) {
    buf.resize(buf.size() + N);
    auto ptr = &*buf.end() - N;
    write(ptr, value);
}

template<typename T, size_t N>
typename std::enable_if_t<std::is_integral_v<T>> read(uint8_t const*& it, std::array<T, N>& value) {
    for (size_t i=0; i<N; ++i) {
        ::read(it, value[i]);
    }
}

template<typename B, typename T, size_t N>
inline std::enable_if_t<std::is_integral_v<T>> write(B& buf, std::array<T, N> const& value) {
    for (size_t i=0; i<N; ++i) {
        ::write(buf, value[i]);
    }
}

template<typename T, size_t N>
inline std::enable_if_t<!std::is_integral_v<T>> read(uint8_t const*& it, std::array<T, N>& value) {
    for (size_t i=0; i<N; ++i) {
        read(it, value[i]);
    }
}

template<typename B, typename T, size_t N>
inline std::enable_if_t<!std::is_integral_v<T>> write(B& buf, std::array<T, N> const& value) {
    for (size_t i=0; i<N; ++i) {
        write(buf, value[i]);
    }
}
}