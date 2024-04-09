import { type AvmContext } from '../avm_context.js';
import { type MemoryValue } from '../avm_memory_types.js';
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

export abstract class GetterInstruction extends Instruction {
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [OperandType.UINT8, OperandType.UINT8, OperandType.UINT32];

  constructor(protected indirect: number, protected dstOffset: number) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    memory.set(this.dstOffset, this.getValue(context));

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }

  protected abstract getValue(env: AvmContext): MemoryValue;
}
