#pragma once
#include <common/serialize.hpp>
#include <iomanip>
#include <ostream>

__extension__ using uint128_t = unsigned __int128;

/*
inline void read(uint8_t const*& it, uint128_t& value) {
    uint64_t hi, lo;
    ::read(it, hi);
    ::read(it, lo);
    value = (static_cast<uint128_t>(hi) << 64) | lo;
}

inline void write(uint8_t*& it, uint128_t value) {
    uint64_t hi = value >> 64;
    uint64_t lo = static_cast<uint64_t>(value);
    ::write(it, hi);
    ::write(it, lo);
}

inline void write(std::vector<uint8_t>& buf, uint128_t value) {
    buf.resize(buf.size() + sizeof(uint128_t));
    auto ptr = &*buf.end() - sizeof(uint128_t);
    ::write(ptr, value);
}
*/
namespace std {
inline std::ostream& operator<<(std::ostream& os, uint128_t const& a)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << "0x" << std::setfill('0') << std::setw(16) << (uint64_t)(a >> 64) << std::setw(16) << (uint64_t)a;
    os.flags(f);
    return os;
}
} // namespace std