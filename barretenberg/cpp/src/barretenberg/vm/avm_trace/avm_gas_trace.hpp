#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include <cstdint>

namespace bb::avm_trace {

struct GasTableEntry {
    uint32_t l2_fixed_gas_cost = 0;
    uint32_t da_fixed_gas_cost = 0;
};

// Temporary values until the definitive gas cost values are settled.
// See TS counterpart constant TemporaryDefaultGasCost in avm_gas.ts
static const inline GasTableEntry temp_default_gas_entry{ .l2_fixed_gas_cost = 10, .da_fixed_gas_cost = 2 };

static const inline std::unordered_map<OpCode, GasTableEntry> GAS_COST_TABLE = {
    // Compute
    // Compute - Arithmetic
    { OpCode::ADD, temp_default_gas_entry },
    { OpCode::SUB, temp_default_gas_entry },
    { OpCode::MUL, temp_default_gas_entry },
    { OpCode::DIV, temp_default_gas_entry },
    { OpCode::FDIV, temp_default_gas_entry },
    // Compute - Comparators
    { OpCode::EQ, temp_default_gas_entry },
    { OpCode::LT, temp_default_gas_entry },
    { OpCode::LTE, temp_default_gas_entry },
    // Compute - Bitwise
    { OpCode::AND, temp_default_gas_entry },
    { OpCode::OR, temp_default_gas_entry },
    { OpCode::XOR, temp_default_gas_entry },
    { OpCode::NOT, temp_default_gas_entry },
    { OpCode::SHL, temp_default_gas_entry },
    { OpCode::SHR, temp_default_gas_entry },
    // Compute - Type Conversions
    { OpCode::CAST, temp_default_gas_entry },

    // Execution Environment
    { OpCode::ADDRESS, temp_default_gas_entry },
    { OpCode::STORAGEADDRESS, temp_default_gas_entry },
    { OpCode::SENDER, temp_default_gas_entry },
    { OpCode::FEEPERL2GAS, temp_default_gas_entry },
    { OpCode::FEEPERDAGAS, temp_default_gas_entry },
    { OpCode::TRANSACTIONFEE, temp_default_gas_entry },
    { OpCode::CONTRACTCALLDEPTH, temp_default_gas_entry },
    // Execution Environment - Globals
    { OpCode::CHAINID, temp_default_gas_entry },
    { OpCode::VERSION, temp_default_gas_entry },
    { OpCode::BLOCKNUMBER, temp_default_gas_entry },
    { OpCode::TIMESTAMP, temp_default_gas_entry },
    { OpCode::COINBASE, temp_default_gas_entry },
    { OpCode::BLOCKL2GASLIMIT, temp_default_gas_entry },
    { OpCode::BLOCKDAGASLIMIT, temp_default_gas_entry },
    // Execution Environment - Calldata
    { OpCode::CALLDATACOPY, temp_default_gas_entry },

    // Machine State
    // Machine State - Gas
    { OpCode::L2GASLEFT, temp_default_gas_entry },
    { OpCode::DAGASLEFT, temp_default_gas_entry },
    // Machine State - Internal Control Flow
    { OpCode::JUMP, temp_default_gas_entry },
    { OpCode::JUMPI, temp_default_gas_entry },
    { OpCode::INTERNALCALL, temp_default_gas_entry },
    { OpCode::INTERNALRETURN, temp_default_gas_entry },
    // Machine State - Memory
    { OpCode::SET, temp_default_gas_entry },
    { OpCode::MOV, temp_default_gas_entry },
    { OpCode::CMOV, temp_default_gas_entry },

    // World State
    { OpCode::SLOAD, temp_default_gas_entry },
    { OpCode::SSTORE, temp_default_gas_entry },
    { OpCode::NOTEHASHEXISTS, temp_default_gas_entry },
    { OpCode::EMITNOTEHASH, temp_default_gas_entry },
    { OpCode::NULLIFIEREXISTS, temp_default_gas_entry },
    { OpCode::EMITNULLIFIER, temp_default_gas_entry },
    { OpCode::L1TOL2MSGEXISTS, temp_default_gas_entry },
    { OpCode::HEADERMEMBER, temp_default_gas_entry },
    { OpCode::GETCONTRACTINSTANCE, temp_default_gas_entry },

    // Accrued Substate
    { OpCode::EMITUNENCRYPTEDLOG, temp_default_gas_entry },
    { OpCode::SENDL2TOL1MSG, temp_default_gas_entry },

    // Control Flow - Contract Calls
    { OpCode::CALL, temp_default_gas_entry },
    { OpCode::STATICCALL, temp_default_gas_entry },
    { OpCode::DELEGATECALL, temp_default_gas_entry },
    { OpCode::RETURN, temp_default_gas_entry },
    { OpCode::REVERT, temp_default_gas_entry },

    // Misc
    { OpCode::DEBUGLOG, temp_default_gas_entry },

    // Gadgets
    { OpCode::KECCAK, temp_default_gas_entry },
    { OpCode::POSEIDON2, temp_default_gas_entry },
    { OpCode::SHA256, temp_default_gas_entry },
    { OpCode::PEDERSEN, temp_default_gas_entry },
    { OpCode::ECADD, temp_default_gas_entry },

    // Conversions
    { OpCode::TORADIXLE, temp_default_gas_entry },

    // Future Gadgets -- pending changes in noir
    { OpCode::SHA256COMPRESSION, temp_default_gas_entry },
    { OpCode::KECCAKF1600, temp_default_gas_entry }, // Here for when we eventually support this
    // Sentinel
    // LAST_OPCODE_SENTINEL,
};

class AvmGasTraceBuilder {
  public:
    struct GasTraceEntry {
        uint32_t clk = 0;
        OpCode opcode = OpCode::ADD; // 0
        uint32_t l2_gas_cost = 0;
        uint32_t da_gas_cost = 0;
        uint32_t remaining_l2_gas = 0;
        uint32_t remaining_da_gas = 0;
    };

    // Counts each time an opcode is read
    // opcode -> count
    std::unordered_map<OpCode, uint32_t> gas_opcode_lookup_counter;

    AvmGasTraceBuilder() = default;

    void reset();
    std::vector<GasTraceEntry> finalize();

    void constrain_gas_lookup(uint32_t clk, OpCode opcode);
    void constrain_gas_for_external_call(uint32_t clk, uint32_t nested_l2_gas_cost, uint32_t nested_da_gas_cost);
    void set_initial_gas(uint32_t l2_gas, uint32_t da_gas);

    uint32_t get_l2_gas_left();
    uint32_t get_da_gas_left();

    std::vector<GasTraceEntry> gas_trace;

    uint32_t initial_l2_gas = 0;
    uint32_t initial_da_gas = 0;

  private:
    uint32_t remaining_l2_gas = 0;
    uint32_t remaining_da_gas = 0;
};

} // namespace bb::avm_trace
