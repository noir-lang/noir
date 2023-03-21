#pragma once

#include "aztec3/utils/types/native_types.hpp"

namespace serialize {

inline void read(uint8_t const*& it, aztec3::utils::types::NativeTypes::Proof& proof)
{
    using serialize::read;

    read(it, proof.proof_data);
};
} // namespace serialize

namespace std {
inline std::ostream& operator<<(std::ostream& os, aztec3::utils::types::NativeTypes::Proof const& data)
{
    return os << data.proof_data;
}
} // namespace std