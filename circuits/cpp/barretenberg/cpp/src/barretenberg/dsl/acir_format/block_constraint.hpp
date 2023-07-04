#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct MemOp {
    uint8_t access_type;
    poly_triple index;
    poly_triple value;
};

enum BlockType {
    ROM = 0,
    RAM = 1,
};

struct BlockConstraint {
    std::vector<poly_triple> init;
    std::vector<MemOp> trace;
    BlockType type;
};

void create_block_constraints(Builder& builder,
                              const BlockConstraint constraint,
                              bool has_valid_witness_assignments = true);

template <typename B> inline void read(B& buf, MemOp& mem_op)
{
    using serialize::read;
    read(buf, mem_op.access_type);
    read(buf, mem_op.index);
    read(buf, mem_op.value);
}

template <typename B> inline void write(B& buf, MemOp const& mem_op)
{
    using serialize::write;
    write(buf, mem_op.access_type);
    write(buf, mem_op.index);
    write(buf, mem_op.value);
}

template <typename B> inline void read(B& buf, BlockConstraint& constraint)
{
    using serialize::read;
    read(buf, constraint.init);
    read(buf, constraint.trace);
    uint8_t type;
    read(buf, type);
    constraint.type = static_cast<BlockType>(type);
}

template <typename B> inline void write(B& buf, BlockConstraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.init);
    write(buf, constraint.trace);
    write(buf, static_cast<uint8_t>(constraint.type));
}
} // namespace acir_format
