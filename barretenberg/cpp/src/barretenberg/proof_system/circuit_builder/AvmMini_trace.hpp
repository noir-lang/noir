#pragma once

#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/proof_system/circuit_builder/circuit_builder_base.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/AvmMini_circuit_builder.hpp"

#include "barretenberg/flavor/generated/AvmMini_flavor.hpp"

#include "barretenberg/relations/generated/AvmMini/avm_mini.hpp"

using Flavor = proof_system::honk::flavor::AvmMiniFlavor;
using FF = Flavor::FF;
using Row = proof_system::AvmMiniFullRow<FF>;

namespace proof_system {

// This is the internal context that we keep along the lifecycle of bytecode execution
// to iteratively build the whole trace. This is effectively performing witness generation.
// At the end of circuit building, mainTrace can be moved to AvmMiniCircuitBuilder by calling
// AvmMiniCircuitBuilder::set_trace(rows).
class AvmMiniTraceBuilder {

  public:
    // Number of rows
    static const size_t N = 256;
    static const size_t MEM_SIZE = 1024;

    static const uint32_t SUB_CLK_LOAD_A = 0;
    static const uint32_t SUB_CLK_LOAD_B = 1;
    static const uint32_t SUB_CLK_LOAD_C = 2;
    static const uint32_t SUB_CLK_STORE_A = 3;
    static const uint32_t SUB_CLK_STORE_B = 4;
    static const uint32_t SUB_CLK_STORE_C = 5;

    AvmMiniTraceBuilder();

    // Temporary helper to initialize memory.
    void setFFMem(size_t idx, FF el);

    std::vector<Row> finalize();
    void reset();

    // Addition over finite field with direct memory access.
    void add(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset);

    // Subtraction over finite field with direct memory access.
    void sub(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset);

    // Multiplication over finite field with direct memory access.
    void mul(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset);

    // Division over finite field with direct memory access.
    void div(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset);

    // CALLDATACOPY opcode with direct memory access, i.e.,
    // M[dstOffset:dstOffset+copySize] = calldata[cdOffset:cdOffset+copySize]
    void callDataCopy(uint32_t cdOffset, uint32_t copySize, uint32_t dstOffset, std::vector<FF> const& callDataMem);

    // RETURN opcode with direct memory access, i.e.,
    // return(M[retOffset:retOffset+retSize])
    std::vector<FF> returnOP(uint32_t retOffset, uint32_t retSize);

  private:
    struct MemoryTraceEntry {
        uint32_t m_clk;
        uint32_t m_sub_clk;
        uint32_t m_addr;
        FF m_val;
        bool m_rw;
    };

    std::vector<Row> mainTrace;
    std::vector<MemoryTraceEntry> memTrace; // Entries will be sorted by m_clk, m_sub_clk after finalize().
    std::array<FF, MEM_SIZE> ffMemory{};    // Memory table for finite field elements
    // Used for simulation of memory table

    static bool compareMemEntries(const MemoryTraceEntry& left, const MemoryTraceEntry& right);
    void insertInMemTrace(uint32_t m_clk, uint32_t m_sub_clk, uint32_t m_addr, FF m_val, bool m_rw);
    void loadAInMemTrace(uint32_t addr, FF val);
    void loadBInMemTrace(uint32_t addr, FF val);
    void loadCInMemTrace(uint32_t addr, FF val);
    void storeAInMemTrace(uint32_t addr, FF val);
    void storeBInMemTrace(uint32_t addr, FF val);
    void storeCInMemTrace(uint32_t addr, FF val);
};
} // namespace proof_system
