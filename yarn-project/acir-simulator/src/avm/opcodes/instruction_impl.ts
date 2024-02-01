import { OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';

/**
 * Covers (de)serialization for an instruction with:
 * indirect, inTag, and two UINT32s.
 */
export abstract class TwoOperandInstruction extends Instruction {
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    protected indirect: number,
    protected inTag: number,
    protected aOffset: number,
    protected dstOffset: number,
  ) {
    super();
  }
}

/**
 * Covers (de)serialization for an instruction with:
 * indirect, inTag, and three UINT32s.
 */
export abstract class ThreeOperandInstruction extends Instruction {
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    protected indirect: number,
    protected inTag: number,
    protected aOffset: number,
    protected bOffset: number,
    protected dstOffset: number,
  ) {
    super();
  }
}
