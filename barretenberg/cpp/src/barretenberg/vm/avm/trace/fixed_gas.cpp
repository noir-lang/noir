#include "barretenberg/vm/avm/trace/fixed_gas.hpp"
#include "barretenberg/vm/avm/trace/opcode.hpp"
#include <unordered_map>

namespace bb::avm_trace {

namespace {

const auto DEFAULT_COST = FixedGasTable::GasRow{
    .base_l2_gas_fixed_table = 10,
    .base_da_gas_fixed_table = 0,
    .dyn_l2_gas_fixed_table = 0,
    .dyn_da_gas_fixed_table = 0,
};

const std::unordered_map<OpCode, FixedGasTable::GasRow> GAS_COST_TABLE = {
    { OpCode::ADD, DEFAULT_COST },
    { OpCode::SUB, DEFAULT_COST },
    { OpCode::MUL, DEFAULT_COST },
    { OpCode::DIV, DEFAULT_COST },
    { OpCode::FDIV, DEFAULT_COST },
    { OpCode::EQ, DEFAULT_COST },
    { OpCode::LT, DEFAULT_COST },
    { OpCode::LTE, DEFAULT_COST },
    { OpCode::AND, DEFAULT_COST },
    { OpCode::OR, DEFAULT_COST },
    { OpCode::XOR, DEFAULT_COST },
    { OpCode::NOT, DEFAULT_COST },
    { OpCode::SHL, DEFAULT_COST },
    { OpCode::SHR, DEFAULT_COST },
    { OpCode::CAST, DEFAULT_COST },
    { OpCode::ADDRESS, DEFAULT_COST },
    { OpCode::STORAGEADDRESS, DEFAULT_COST },
    { OpCode::SENDER, DEFAULT_COST },
    { OpCode::FUNCTIONSELECTOR, DEFAULT_COST },
    { OpCode::TRANSACTIONFEE, DEFAULT_COST },
    { OpCode::CHAINID, DEFAULT_COST },
    { OpCode::VERSION, DEFAULT_COST },
    { OpCode::BLOCKNUMBER, DEFAULT_COST },
    { OpCode::TIMESTAMP, DEFAULT_COST },
    { OpCode::COINBASE, DEFAULT_COST },
    { OpCode::FEEPERL2GAS, DEFAULT_COST },
    { OpCode::FEEPERDAGAS, DEFAULT_COST },
    { OpCode::BLOCKL2GASLIMIT, DEFAULT_COST },
    { OpCode::BLOCKDAGASLIMIT, DEFAULT_COST },
    { OpCode::CALLDATACOPY, DEFAULT_COST },
    { OpCode::L2GASLEFT, DEFAULT_COST },
    { OpCode::DAGASLEFT, DEFAULT_COST },
    { OpCode::JUMP, DEFAULT_COST },
    { OpCode::JUMPI, DEFAULT_COST },
    { OpCode::INTERNALCALL, DEFAULT_COST },
    { OpCode::INTERNALRETURN, DEFAULT_COST },
    { OpCode::SET, DEFAULT_COST },
    { OpCode::MOV, DEFAULT_COST },
    { OpCode::CMOV, DEFAULT_COST },
    { OpCode::SLOAD, DEFAULT_COST },
    { OpCode::SSTORE, DEFAULT_COST },
    { OpCode::NOTEHASHEXISTS, DEFAULT_COST },
    { OpCode::EMITNOTEHASH, DEFAULT_COST },
    { OpCode::NULLIFIEREXISTS, DEFAULT_COST },
    { OpCode::EMITNULLIFIER, DEFAULT_COST },
    { OpCode::L1TOL2MSGEXISTS, DEFAULT_COST },
    { OpCode::HEADERMEMBER, DEFAULT_COST },
    { OpCode::GETCONTRACTINSTANCE, DEFAULT_COST },
    { OpCode::EMITUNENCRYPTEDLOG, DEFAULT_COST },
    { OpCode::SENDL2TOL1MSG, DEFAULT_COST },
    { OpCode::CALL, DEFAULT_COST },
    { OpCode::STATICCALL, DEFAULT_COST },
    { OpCode::DELEGATECALL, DEFAULT_COST },
    { OpCode::RETURN, DEFAULT_COST },
    { OpCode::REVERT, DEFAULT_COST },
    { OpCode::DEBUGLOG, DEFAULT_COST },
    { OpCode::KECCAK, DEFAULT_COST },
    { OpCode::POSEIDON2, DEFAULT_COST },
    { OpCode::SHA256, DEFAULT_COST },
    { OpCode::PEDERSEN, DEFAULT_COST },
    { OpCode::ECADD, DEFAULT_COST },
    { OpCode::MSM, DEFAULT_COST },
    { OpCode::PEDERSENCOMMITMENT, DEFAULT_COST },
    { OpCode::TORADIXLE, DEFAULT_COST },
    { OpCode::SHA256COMPRESSION, DEFAULT_COST },
    { OpCode::KECCAKF1600, DEFAULT_COST },
};

} // namespace

size_t FixedGasTable::size() const
{
    return GAS_COST_TABLE.size();
}

const FixedGasTable::GasRow& FixedGasTable::at(OpCode o) const
{
    return GAS_COST_TABLE.at(o);
}

// Singleton.
const FixedGasTable& FixedGasTable::get()
{
    static FixedGasTable table;
    return table;
}

} // namespace bb::avm_trace