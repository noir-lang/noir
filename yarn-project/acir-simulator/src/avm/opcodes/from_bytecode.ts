import { Instruction } from './instruction.js';
import { INSTRUCTION_SET } from './instruction_set.js';
import { Opcode } from './opcodes.js';

export const OPERAND_BIT_LENGTH = 32;
export const OPERAND_BYTE_LENGTH = 4;
export const OPCODE_BIT_LENGTH = 8;
export const OPCODE_BYTE_LENGTH = 1;

/**
 * Convert a buffer of bytecode into an array of instructions
 * @param bytecode - Buffer of bytecode
 * @returns Bytecode interpreted into an ordered array of Instructions
 */
export function interpretBytecode(bytecode: Buffer): Instruction[] {
  let readPtr = 0;
  const bytecodeLength = bytecode.length;

  const instructions: Instruction[] = [];

  while (readPtr < bytecodeLength) {
    const opcodeByte = bytecode[readPtr];
    readPtr += 1;
    if (!(opcodeByte in Opcode)) {
      throw new Error(`Opcode ${opcodeByte} not implemented`);
    }
    const opcode = opcodeByte as Opcode;

    const instructionType = INSTRUCTION_SET.get(opcode);
    if (instructionType === undefined) {
      throw new Error(`Opcode ${opcode} not implemented`);
    }
    const numberOfOperands = instructionType.numberOfOperands;
    const operands: number[] = [];
    for (let i = 0; i < numberOfOperands; i++) {
      const operand = bytecode.readUInt32BE(readPtr);
      readPtr += OPERAND_BYTE_LENGTH;
      operands.push(operand);
    }

    instructions.push(new instructionType(...operands));
  }

  return instructions;
}
