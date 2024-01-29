import { AVM_OPCODE_BYTE_LENGTH, AVM_OPERAND_BYTE_LENGTH } from './instruction.js';
import { INSTRUCTION_SET } from './instruction_set.js';
import { Opcode } from './opcodes.js';

/**
 * Encode an instruction (opcode & arguments) to bytecode.
 * @param opcode - the opcode to encode
 * @param args - the arguments to encode
 * @returns the bytecode for this one instruction
 */
export function encodeToBytecode(opcode: Opcode, args: number[]): Buffer {
  const instructionType = INSTRUCTION_SET.get(opcode);
  if (instructionType === undefined) {
    throw new Error(`Opcode 0x${opcode.toString(16)} not implemented`);
  }

  const numberOfOperands = instructionType.numberOfOperands;
  if (args.length !== numberOfOperands) {
    throw new Error(
      `Opcode 0x${opcode.toString(16)} expects ${numberOfOperands} arguments, but ${args.length} were provided`,
    );
  }

  const bytecode = Buffer.alloc(AVM_OPCODE_BYTE_LENGTH + numberOfOperands * AVM_OPERAND_BYTE_LENGTH);

  let bytePtr = 0;
  bytecode.writeUInt8(opcode as number, bytePtr);
  bytePtr += AVM_OPCODE_BYTE_LENGTH;
  for (let i = 0; i < args.length; i++) {
    bytecode.writeUInt32BE(args[i], bytePtr);
    bytePtr += AVM_OPERAND_BYTE_LENGTH;
  }
  return bytecode;
}
