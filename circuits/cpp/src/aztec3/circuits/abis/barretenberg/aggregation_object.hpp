#pragma once
#include "barretenberg/common/serialize.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace serialize {

inline void read(uint8_t const*& it, aztec3::utils::types::NativeTypes::AggregationObject& obj)
{
    using serialize::read;

    read(it, obj.P0);
    read(it, obj.P1);
    read(it, obj.public_inputs);
    read(it, obj.proof_witness_indices);
    read(it, obj.has_data);
};

} // namespace serialize

namespace std {

inline std::ostream& operator<<(std::ostream& os, stdlib::recursion::native_recursion_output const& obj)
{
    return os << "P0: " << obj.P0 << "\n"
              << "P1: " << obj.P1 << "\n"
              << "public_inputs:\n"
              << obj.public_inputs << "\n"
              << "proof_witness_indices:\n"
              << obj.proof_witness_indices << "\n"
              << "has_data: " << obj.has_data << "\n";
};

} // namespace std