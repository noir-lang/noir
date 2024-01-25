import { Add, Div, Mul, Sub } from './arithmetic.js';
import { And, Not, Or, Shl, Shr, Xor } from './bitwise.js';
import { InternalCall, InternalReturn, Jump, JumpI, Return } from './control_flow.js';
// import { Call } from './external_calls.js';
import { Instruction } from './instruction.js';
import { CMov, CalldataCopy, Cast, Mov, Set } from './memory.js';
import { Opcode } from './opcodes.js';
//import { Eq, Lt, Lte } from './comparators.js';
import { SLoad, SStore } from './storage.js';

/** - */
type InstructionConstructor = new (...args: any[]) => Instruction;
/** - */
type InstructionConstructorAndMembers = InstructionConstructor & {
  /** - */
  numberOfOperands: number;
};

export const INSTRUCTION_SET: Map<Opcode, InstructionConstructorAndMembers> = new Map(
  new Array<[Opcode, InstructionConstructorAndMembers]>(
    // Compute
    // Compute - Arithmetic
    [Opcode.ADD, Add],
    [Opcode.SUB, Sub],
    [Opcode.MUL, Mul],
    [Opcode.DIV, Div],
    //// Compute - Comparators
    //[Opcode.EQ, Eq],
    //[Opcode.LT, Lt],
    //[Opcode.LTE, Lte],
    //// Compute - Bitwise
    [Opcode.AND, And],
    [Opcode.OR, Or],
    [Opcode.XOR, Xor],
    [Opcode.NOT, Not],
    [Opcode.SHL, Shl],
    [Opcode.SHR, Shr],
    //// Compute - Type Conversions
    [Opcode.CAST, Cast],

    //// Execution Environment
    //[Opcode.ADDRESS, Address],
    //[Opcode.STORAGEADDRESS, Storageaddress],
    //[Opcode.ORIGIN, Origin],
    //[Opcode.SENDER, Sender],
    //[Opcode.PORTAL, Portal],
    //[Opcode.FEEPERL1GAS, Feeperl1gas],
    //[Opcode.FEEPERL2GAS, Feeperl2gas],
    //[Opcode.FEEPERDAGAS, Feeperdagas],
    //[Opcode.CONTRACTCALLDEPTH, Contractcalldepth],
    //// Execution Environment - Globals
    //[Opcode.CHAINID, Chainid],
    //[Opcode.VERSION, Version],
    //[Opcode.BLOCKNUMBER, Blocknumber],
    //[Opcode.TIMESTAMP, Timestamp],
    //[Opcode.COINBASE, Coinbase],
    //[Opcode.BLOCKL1GASLIMIT, Blockl1gaslimit],
    //[Opcode.BLOCKL2GASLIMIT, Blockl2gaslimit],
    //[Opcode.BLOCKDAGASLIMIT, Blockdagaslimit],
    // Execution Environment - Calldata
    [Opcode.CALLDATACOPY, CalldataCopy],

    //// Machine State
    // Machine State - Gas
    //[Opcode.L1GASLEFT, L1gasleft],
    //[Opcode.L2GASLEFT, L2gasleft],
    //[Opcode.DAGASLEFT, Dagasleft],
    //// Machine State - Internal Control Flow
    [Opcode.JUMP, Jump],
    [Opcode.JUMPI, JumpI],
    [Opcode.INTERNALCALL, InternalCall],
    [Opcode.INTERNALRETURN, InternalReturn],
    //// Machine State - Memory
    [Opcode.SET, Set],
    [Opcode.MOV, Mov],
    [Opcode.CMOV, CMov],

    //// World State
    //[Opcode.BLOCKHEADERBYNUMBER, Blockheaderbynumber],
    [Opcode.SLOAD, SLoad], // Public Storage
    [Opcode.SSTORE, SStore], // Public Storage
    //[Opcode.READL1TOL2MSG, Readl1tol2msg], // Messages
    //[Opcode.SENDL2TOL1MSG, Sendl2tol1msg], // Messages
    //[Opcode.EMITNOTEHASH, Emitnotehash], // Notes & Nullifiers
    //[Opcode.EMITNULLIFIER, Emitnullifier], // Notes & Nullifiers

    //// Accrued Substate
    //[Opcode.EMITUNENCRYPTEDLOG, Emitunencryptedlog],

    //// Control Flow - Contract Calls
    // [Opcode.CALL, Call],
    //[Opcode.STATICCALL, Staticcall],
    [Opcode.RETURN, Return],
    //[Opcode.REVERT, Revert],

    //// Gadgets
    //[Opcode.KECCAK, Keccak],
    //[Opcode.POSEIDON, Poseidon],
  ),
);
