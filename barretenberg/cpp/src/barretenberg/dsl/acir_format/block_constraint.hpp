#pragma once
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct MemOp {
    uint8_t access_type;
    bb::poly_triple index;
    bb::poly_triple value;
};

enum BlockType {
    ROM = 0,
    RAM = 1,
    CallData = 2,
    ReturnData = 3,
};

struct BlockConstraint {
    std::vector<bb::poly_triple> init;
    std::vector<MemOp> trace;
    BlockType type;
};

template <typename Builder>
void create_block_constraints(Builder& builder,
                              const BlockConstraint& constraint,
                              bool has_valid_witness_assignments = true);

template <typename Builder>
void process_ROM_operations(Builder& builder,
                            const BlockConstraint& constraint,
                            bool has_valid_witness_assignments,
                            std::vector<bb::stdlib::field_t<Builder>>& init);
template <typename Builder>
void process_RAM_operations(Builder& builder,
                            const BlockConstraint& constraint,
                            bool has_valid_witness_assignments,
                            std::vector<bb::stdlib::field_t<Builder>>& init);
template <typename Builder>
void process_call_data_operations(Builder& builder,
                                  const BlockConstraint& constraint,
                                  bool has_valid_witness_assignments,
                                  std::vector<bb::stdlib::field_t<Builder>>& init);
template <typename Builder>
void process_return_data_operations(const BlockConstraint& constraint, std::vector<bb::stdlib::field_t<Builder>>& init);

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
