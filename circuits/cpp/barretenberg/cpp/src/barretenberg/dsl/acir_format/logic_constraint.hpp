#pragma once
#include <cstdint>
#include "barretenberg/dsl/types.hpp"

namespace acir_format {

struct LogicConstraint {
    uint32_t a;
    uint32_t b;
    uint32_t result;
    uint32_t num_bits;
    uint32_t is_xor_gate;

    friend bool operator==(LogicConstraint const& lhs, LogicConstraint const& rhs) = default;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(a, b, result, num_bits, is_xor_gate);
};

void create_logic_gate(Composer& composer, uint32_t a, uint32_t b, uint32_t result, size_t num_bits, bool is_xor_gate);

void xor_gate(Composer& composer, uint32_t a, uint32_t b, uint32_t result);

void and_gate(Composer& composer, uint32_t a, uint32_t b, uint32_t result);

template <typename B> inline void read(B& buf, LogicConstraint& constraint)
{
    using serialize::read;
    read(buf, constraint.a);
    read(buf, constraint.b);
    read(buf, constraint.result);
    read(buf, constraint.num_bits);
    read(buf, constraint.is_xor_gate);
}

template <typename B> inline void write(B& buf, LogicConstraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.a);
    write(buf, constraint.b);
    write(buf, constraint.result);
    write(buf, constraint.num_bits);
    write(buf, constraint.is_xor_gate);
}
} // namespace acir_format
