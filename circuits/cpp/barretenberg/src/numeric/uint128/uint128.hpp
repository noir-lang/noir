#pragma once
#include <ostream>
#include <iomanip>

__extension__ using uint128_t = unsigned __int128;

inline std::ostream& operator<<(std::ostream& os, uint128_t const& a)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << "0x" << std::setfill('0') << std::setw(16) << (uint64_t)(a >> 64) << std::setw(16) << (uint64_t)a;
    os.flags(f);
    return os;
}