import { strict as assert } from 'assert';

import { BufferCursor } from './buffer_cursor.js';

/**
 * All AVM opcodes. (Keep in sync with cpp counterpart code avm_opcode.hpp).
 * Source: https://yp-aztec.netlify.app/docs/public-vm/instruction-set
 */
export enum Opcode {
  // Compute
  ADD,
  SUB,
  MUL,
  DIV,
  FDIV,
  EQ,
  LT,
  LTE,
  AND,
  OR,
  XOR,
  NOT,
  SHL,
  SHR,
  CAST,
  // Execution environment
  ADDRESS,
  STORAGEADDRESS,
  ORIGIN,
  SENDER,
  PORTAL,
  FEEPERL1GAS,
  FEEPERL2GAS,
  FEEPERDAGAS,
  CONTRACTCALLDEPTH,
  CHAINID,
  VERSION,
  BLOCKNUMBER,
  TIMESTAMP,
  COINBASE,
  BLOCKL1GASLIMIT,
  BLOCKL2GASLIMIT,
  BLOCKDAGASLIMIT,
  CALLDATACOPY,
  // Gas
  L1GASLEFT,
  L2GASLEFT,
  DAGASLEFT,
  // Control flow
  JUMP,
  JUMPI,
  INTERNALCALL,
  INTERNALRETURN,
  // Memory
  SET,
  MOV,
  CMOV,
  // World state
  SLOAD,
  SSTORE,
  NOTEHASHEXISTS,
  EMITNOTEHASH,
  NULLIFIEREXISTS,
  EMITNULLIFIER,
  L1TOL2MSGEXISTS,
  HEADERMEMBER,
  GETCONTRACTINSTANCE,
  EMITUNENCRYPTEDLOG,
  SENDL2TOL1MSG,
  // External calls
  CALL,
  STATICCALL,
  DELEGATECALL,
  RETURN,
  REVERT,
  // Gadgets
  KECCAK,
  POSEIDON,
  SHA256, // temp - may be removed, but alot of contracts rely on it
  PEDERSEN, // temp - may be removed, but alot of contracts rely on it
}

// Possible types for an instruction's operand in its wire format. (Keep in sync with CPP code.
// See vm/avm_trace/avm_deserialization.cpp)
// Note that cpp code introduced an additional enum value TAG to express the instruction tag. In TS,
// this one is parsed as UINT8.
export enum OperandType {
  UINT8,
  UINT16,
  UINT32,
  UINT64,
  UINT128,
}

type OperandNativeType = number | bigint;
type OperandWriter = (value: any) => void;

// Specifies how to read and write each operand type.
const OPERAND_SPEC = new Map<OperandType, [number, () => OperandNativeType, OperandWriter]>([
  [OperandType.UINT8, [1, Buffer.prototype.readUint8, Buffer.prototype.writeUint8]],
  [OperandType.UINT16, [2, Buffer.prototype.readUint16BE, Buffer.prototype.writeUint16BE]],
  [OperandType.UINT32, [4, Buffer.prototype.readUint32BE, Buffer.prototype.writeUint32BE]],
  [OperandType.UINT64, [8, Buffer.prototype.readBigInt64BE, Buffer.prototype.writeBigInt64BE]],
  [OperandType.UINT128, [16, readBigInt128BE, writeBigInt128BE]],
]);

function readBigInt128BE(this: Buffer): bigint {
  const totalBytes = 16;
  let ret: bigint = 0n;
  for (let i = 0; i < totalBytes; ++i) {
    ret <<= 8n;
    ret |= BigInt(this.readUint8(i));
  }
  return ret;
}

function writeBigInt128BE(this: Buffer, value: bigint): void {
  const totalBytes = 16;
  for (let offset = totalBytes - 1; offset >= 0; --offset) {
    this.writeUint8(Number(value & 0xffn), offset);
    value >>= 8n;
  }
}

/**
 * Reads an array of operands from a buffer.
 * @param cursor Buffer to read from. Might be longer than needed.
 * @param operands Specification of the operand types.
 * @returns An array as big as {@code operands}, with the converted TS values.
 */
export function deserialize(cursor: BufferCursor | Buffer, operands: OperandType[]): (number | bigint)[] {
  const argValues = [];
  if (cursor instanceof Buffer) {
    cursor = new BufferCursor(cursor);
  }

  for (const op of operands) {
    const opType = op;
    const [sizeBytes, reader, _writer] = OPERAND_SPEC.get(opType)!;
    argValues.push(reader.call(cursor.bufferAtPosition()));
    cursor.advance(sizeBytes);
  }

  return argValues;
}

/**
 * Serializes a class using the specified operand types.
 * More specifically, this serializes {@code [cls.constructor.opcode, ...Object.values(cls)]}.
 * Observe in particular that:
 *   (1) the first operand type specified must correspond to the opcode;
 *   (2) the rest of the operand types must be specified in the order returned by {@code Object.values()}.
 * @param operands Type specification for the values to be serialized.
 * @param cls The class to be serialized.
 * @returns
 */
export function serialize(operands: OperandType[], cls: any): Buffer {
  const chunks: Buffer[] = [];

  // TODO: infer opcode not in this loop
  assert(cls.constructor.opcode !== undefined && cls.constructor.opcode !== null);
  const rawClassValues: any[] = [cls.constructor.opcode, ...Object.values(cls)];
  assert(
    rawClassValues.length === operands.length,
    `Got ${rawClassValues.length} values but only ${operands.length} serialization operands are specified!`,
  );
  const classValues = rawClassValues as OperandNativeType[];

  for (let i = 0; i < operands.length; i++) {
    const opType = operands[i];
    const [sizeBytes, _reader, writer] = OPERAND_SPEC.get(opType)!;
    const buf = Buffer.alloc(sizeBytes);
    writer.call(buf, classValues[i]);
    chunks.push(buf);
  }

  return Buffer.concat(chunks);
}
