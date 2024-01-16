#pragma once

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "stdint.h"
#include <array>
#include <iomanip>
#include <ostream>
#include <vector>

namespace sha256 {

using hash = std::array<uint8_t, 32>;

hash sha256_block(const std::vector<uint8_t>& input);

template <typename T> hash sha256(const T& input);

inline bb::fr sha256_to_field(std::vector<uint8_t> const& input)
{
    auto result = sha256::sha256(input);
    return from_buffer<bb::fr>(&result[0]);
}

inline bool operator==(hash const& lhs, std::vector<uint8_t> const& rhs)
{
    return std::equal(lhs.begin(), lhs.end(), rhs.begin());
}

} // namespace sha256

namespace std {
inline bool operator==(std::vector<uint8_t> const& lhs, sha256::hash const& rhs)
{
    return std::equal(lhs.begin(), lhs.end(), rhs.begin());
}

inline std::ostream& operator<<(std::ostream& os, sha256::hash const& arr)
{
    std::ios_base::fmtflags f(os.flags());
    os << std::hex << std::setfill('0');
    for (auto byte : arr) {
        os << std::setw(2) << +(unsigned char)byte;
    }
    os.flags(f);
    return os;
}
} // namespace std
