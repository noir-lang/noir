#pragma once
#include <vector>
#include <type_traits>
#include <common/net.hpp>

inline void read(uint8_t*& it, uint8_t& value) {
    value = *it;
    it += 1;
}

inline void write(std::vector<uint8_t>& buf, uint8_t value) {
    buf.resize(buf.size() + 1);
    *(buf.end() - 1) = value;
}

inline void read(uint8_t*& it, uint16_t& value) {
    value = ntohs(*reinterpret_cast<uint16_t*>(it));
    it += 2;
}

inline void write(std::vector<uint8_t>& buf, uint16_t value) {
    buf.resize(buf.size() + 2);
    *reinterpret_cast<uint16_t*>(&*buf.end() - 2) = htons(value);
}

inline void read(uint8_t*& it, uint32_t& value) {
    value = ntohl(*reinterpret_cast<uint32_t*>(it));
    it += 4;
}

inline void write(std::vector<uint8_t>& buf, uint32_t value) {
    buf.resize(buf.size() + 4);
    *reinterpret_cast<uint32_t*>(&*buf.end() - 4) = htonl(value);
}

inline void read(uint8_t*& it, uint64_t& value) {
    value = ntohll(*reinterpret_cast<uint64_t*>(it));
    it += 8;
}

inline void write(std::vector<uint8_t>& buf, uint64_t value) {
    buf.resize(buf.size() + 8);
    *reinterpret_cast<uint64_t*>(&*buf.end() - 8) = htonll(value);
}

namespace std {
template<size_t N>
inline void read(uint8_t*& it, std::array<uint8_t, N>& value) {
    std::copy(it, it+N, value.data());
    it += N;
}

template<size_t N>
inline void write(std::vector<uint8_t>& buf, std::array<uint8_t, N> const& value) {
    buf.resize(buf.size() + N);
    std::copy(value.begin(), value.end(), buf.end()-N);
}

template<typename T, size_t N>
typename std::enable_if_t<std::is_integral_v<T>> read(uint8_t*& it, std::array<T, N>& value) {
    for (size_t i=0; i<N; ++i) {
        ::read(it, value[i]);
    }
}

template<typename T, size_t N>
inline std::enable_if_t<std::is_integral_v<T>> write(std::vector<uint8_t>& buf, std::array<T, N> const& value) {
    for (size_t i=0; i<N; ++i) {
        ::write(buf, value[i]);
    }
}

template<typename T, size_t N>
inline std::enable_if_t<!std::is_integral_v<T>> read(uint8_t*& it, std::array<T, N>& value) {
    for (size_t i=0; i<N; ++i) {
        read(it, value[i]);
    }
}

template<typename T, size_t N>
inline std::enable_if_t<!std::is_integral_v<T>> write(std::vector<uint8_t>& buf, std::array<T, N> const& value) {
    for (size_t i=0; i<N; ++i) {
        write(buf, value[i]);
    }
}
}