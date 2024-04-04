import { OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';

/** Wire format that informs deserialization for instructions with two operands. */
export const TwoOperandWireFormat = [
  OperandType.UINT8,
  OperandType.UINT8,
  OperandType.UINT8,
  OperandType.UINT32,
  OperandType.UINT32,
];

/** Wire format that informs deserialization for instructions with three operands. */
export const ThreeOperandWireFormat = [
  OperandType.UINT8,
  OperandType.UINT8,
  OperandType.UINT8,
  OperandType.UINT32,
  OperandType.UINT32,
  OperandType.UINT32,
];

/**
 * Covers (de)serialization for an instruction with:
 * indirect, inTag, and two UINT32s.
 */
export abstract class TwoOperandInstruction extends Instruction {
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = TwoOperandWireFormat;

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
  static readonly wireFormat: OperandType[] = ThreeOperandWireFormat;

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
