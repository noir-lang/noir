import { AVM_OPCODE_BYTE_LENGTH, AVM_OPERAND_BYTE_LENGTH, Instruction } from './instruction.js';
import { INSTRUCTION_SET } from './instruction_set.js';
import { Opcode } from './opcodes.js';

/**
 * Convert a buffer of bytecode into an array of instructions
 * @param bytecode - Buffer of bytecode
 * @returns Bytecode decoded into an ordered array of Instructions
 */
export function decodeBytecode(bytecode: Buffer): Instruction[] {
  let bytePtr = 0;
  const bytecodeLength = bytecode.length;

  const instructions: Instruction[] = [];

  while (bytePtr < bytecodeLength) {
    const opcodeByte = bytecode[bytePtr];
    bytePtr += AVM_OPCODE_BYTE_LENGTH;
    if (!(opcodeByte in Opcode)) {
      throw new Error(`Opcode 0x${opcodeByte.toString(16)} not implemented`);
    }
    const opcode = opcodeByte as Opcode;

    const instructionType = INSTRUCTION_SET.get(opcode);
    if (instructionType === undefined) {
      throw new Error(`Opcode 0x${opcode.toString(16)} not implemented`);
    }
    const numberOfOperands = instructionType.numberOfOperands;
    const operands: number[] = [];
    for (let i = 0; i < numberOfOperands; i++) {
      // TODO: support constants which might not be u32s
      const operand = bytecode.readUInt32BE(bytePtr);
      bytePtr += AVM_OPERAND_BYTE_LENGTH;
      operands.push(operand);
    }

    instructions.push(new instructionType(...operands));
  }

  return instructions;
}
