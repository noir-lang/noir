/// All AVM opcodes
/// Keep updated with TS and yellow paper!
#[derive(Copy, Clone)]
pub enum AvmOpcode {
    // Compute
    // Compute - Arithmetic
    ADD,
    SUB,
    MUL,
    DIV,
    // Compute - Comparators
    EQ,
    LT,
    LTE,
    // Compute - Bitwise
    AND,
    OR,
    XOR,
    NOT,
    SHL,
    SHR,
    // Compute - Type Conversions
    CAST,

    // Execution Environment
    ADDRESS,
    STORAGEADDRESS,
    ORIGIN,
    SENDER,
    PORTAL,
    FEEPERL1GAS,
    FEEPERL2GAS,
    FEEPERDAGAS,
    CONTRACTCALLDEPTH,
    // Execution Environment - Globals
    CHAINID,
    VERSION,
    BLOCKNUMBER,
    TIMESTAMP,
    COINBASE,
    BLOCKL1GASLIMIT,
    BLOCKL2GASLIMIT,
    BLOCKDAGASLIMIT,
    // Execution Environment - Calldata
    CALLDATACOPY,

    // Machine State
    // Machine State - Gas
    L1GASLEFT,
    L2GASLEFT,
    DAGASLEFT,
    // Machine State - Internal Control Flow
    JUMP,
    JUMPI,
    INTERNALCALL,
    INTERNALRETURN,
    // Machine State - Memory
    SET,
    MOV,
    CMOV,

    // World State
    BLOCKHEADERBYNUMBER,
    SLOAD,         // Public Storage
    SSTORE,        // Public Storage
    READL1TOL2MSG, // Messages
    SENDL2TOL1MSG, // Messages
    EMITNOTEHASH,  // Notes & Nullifiers
    EMITNULLIFIER, // Notes & Nullifiers

    // Accrued Substate
    EMITUNENCRYPTEDLOG,

    // Control Flow - Contract Calls
    CALL,
    STATICCALL,
    RETURN,
    REVERT,

    // Gadgets
    KECCAK,
    POSEIDON,
}

impl AvmOpcode {
    pub fn name(&self) -> &'static str {
        match self {
            // Compute
            // Compute - Arithmetic
            AvmOpcode::ADD => "ADD",
            AvmOpcode::SUB => "SUB",
            AvmOpcode::MUL => "MUL",
            AvmOpcode::DIV => "DIV",
            // Compute - Comparators
            AvmOpcode::EQ => "EQ",
            AvmOpcode::LT => "LT",
            AvmOpcode::LTE => "LTE",
            // Compute - Bitwise
            AvmOpcode::AND => "AND",
            AvmOpcode::OR => "OR",
            AvmOpcode::XOR => "XOR",
            AvmOpcode::NOT => "NOT",
            AvmOpcode::SHL => "SHL",
            AvmOpcode::SHR => "SHR",
            // Compute - Type Conversions
            AvmOpcode::CAST => "CAST",

            // Execution Environment
            AvmOpcode::ADDRESS => "ADDRESS",
            AvmOpcode::STORAGEADDRESS => "STORAGEADDRESS",
            AvmOpcode::ORIGIN => "ORIGIN",
            AvmOpcode::SENDER => "SENDER",
            AvmOpcode::PORTAL => "PORTAL",
            AvmOpcode::FEEPERL1GAS => "FEEPERL1GAS",
            AvmOpcode::FEEPERL2GAS => "FEEPERL2GAS",
            AvmOpcode::FEEPERDAGAS => "FEEPERDAGAS",
            AvmOpcode::CONTRACTCALLDEPTH => "CONTRACTCALLDEPTH",
            // Execution Environment - Globals
            AvmOpcode::CHAINID => "CHAINID",
            AvmOpcode::VERSION => "VERSION",
            AvmOpcode::BLOCKNUMBER => "BLOCKNUMBER",
            AvmOpcode::TIMESTAMP => "TIMESTAMP",
            AvmOpcode::COINBASE => "COINBASE",
            AvmOpcode::BLOCKL1GASLIMIT => "BLOCKL1GASLIMIT",
            AvmOpcode::BLOCKL2GASLIMIT => "BLOCKL2GASLIMIT",
            AvmOpcode::BLOCKDAGASLIMIT => "BLOCKDAGASLIMIT",
            // Execution Environment - Calldata
            AvmOpcode::CALLDATACOPY => "CALLDATACOPY",

            // Machine State
            // Machine State - Gas
            AvmOpcode::L1GASLEFT => "L1GASLEFT",
            AvmOpcode::L2GASLEFT => "L2GASLEFT",
            AvmOpcode::DAGASLEFT => "DAGASLEFT",
            // Machine State - Internal Control Flow
            AvmOpcode::JUMP => "JUMP",
            AvmOpcode::JUMPI => "JUMPI",
            AvmOpcode::INTERNALCALL => "INTERNALCALL",
            AvmOpcode::INTERNALRETURN => "INTERNALRETURN",
            // Machine State - Memory
            AvmOpcode::SET => "SET",
            AvmOpcode::MOV => "MOV",
            AvmOpcode::CMOV => "CMOV",

            // World State
            AvmOpcode::BLOCKHEADERBYNUMBER => "BLOCKHEADERBYNUMBER",
            AvmOpcode::SLOAD => "SLOAD",   // Public Storage
            AvmOpcode::SSTORE => "SSTORE", // Public Storage
            AvmOpcode::READL1TOL2MSG => "READL1TOL2MSG", // Messages
            AvmOpcode::SENDL2TOL1MSG => "SENDL2TOL1MSG", // Messages
            AvmOpcode::EMITNOTEHASH => "EMITNOTEHASH", // Notes & Nullifiers
            AvmOpcode::EMITNULLIFIER => "EMITNULLIFIER", // Notes & Nullifiers

            // Accrued Substate
            AvmOpcode::EMITUNENCRYPTEDLOG => "EMITUNENCRYPTEDLOG",

            // Control Flow - Contract Calls
            AvmOpcode::CALL => "CALL",
            AvmOpcode::STATICCALL => "STATICCALL",
            AvmOpcode::RETURN => "RETURN",
            AvmOpcode::REVERT => "REVERT",

            // Gadgets
            AvmOpcode::KECCAK => "KECCAK",
            AvmOpcode::POSEIDON => "POSEIDON",
        }
    }
}
