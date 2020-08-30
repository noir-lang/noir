#pragma once
#include <iomanip>
#include <ostream>
#include <vector>

namespace std {

inline std::ostream& operator<<(std::ostream& os, std::vector<uint8_t> const& arr)
{
    std::ios_base::fmtflags f(os.flags());
    os << "[" << std::hex << std::setfill('0');
    for (auto byte : arr) {
        os << ' ' << std::setw(2) << +(unsigned char)byte;
    }
    os << " ]";
    os.flags(f);
    return os;
}

template <typename T> inline std::ostream& operator<<(std::ostream& os, std::vector<T> const& arr)
{
    os << "[\n";
    for (auto element : arr) {
        os << ' ' << element << '\n';
    }
    os << "]\n";
    return os;
}

template <size_t S> inline std::ostream& operator<<(std::ostream& os, std::array<uint8_t, S> const& arr)
{
    std::ios_base::fmtflags f(os.flags());
    os << "[" << std::hex << std::setfill('0');
    for (auto byte : arr) {
        os << ' ' << std::setw(2) << +(unsigned char)byte;
    }
    os << " ]";
    os.flags(f);
    return os;
}

} // namespace std