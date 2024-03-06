import { Fr } from '@aztec/foundation/fields';

import type { AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

abstract class BaseStorageInstruction extends Instruction {
  // Informs (de)serialization. See Instruction.deserialize.
  public static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    protected indirect: number,
    protected aOffset: number,
    protected /*temporary*/ size: number,
    protected bOffset: number,
  ) {
    super();
  }
}

export class SStore extends BaseStorageInstruction {
  static readonly type: string = 'SSTORE';
  static readonly opcode = Opcode.SSTORE;

  constructor(indirect: number, srcOffset: number, /*temporary*/ srcSize: number, slotOffset: number) {
    super(indirect, srcOffset, srcSize, slotOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const [srcOffset, slotOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.aOffset, this.bOffset],
      context.machineState.memory,
    );

    const slot = context.machineState.memory.get(slotOffset).toFr();
    const data = context.machineState.memory.getSlice(srcOffset, this.size).map(field => field.toFr());

    for (const [index, value] of Object.entries(data)) {
      const adjustedSlot = slot.add(new Fr(BigInt(index)));
      context.persistableState.writeStorage(context.environment.storageAddress, adjustedSlot, value);
    }

    context.machineState.incrementPc();
  }
}

export class SLoad extends BaseStorageInstruction {
  static readonly type: string = 'SLOAD';
  static readonly opcode = Opcode.SLOAD;

  constructor(indirect: number, slotOffset: number, size: number, dstOffset: number) {
    super(indirect, slotOffset, size, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    const [aOffset, size, bOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.aOffset, this.size, this.bOffset],
      context.machineState.memory,
    );

    const slot = context.machineState.memory.get(aOffset);

    // Write each read value from storage into memory
    for (let i = 0; i < size; i++) {
      const data: Fr = await context.persistableState.readStorage(
        context.environment.storageAddress,
        new Fr(slot.toBigInt() + BigInt(i)),
      );

      context.machineState.memory.set(bOffset + i, new Field(data));
    }

    context.machineState.incrementPc();
  }
}

/**
 * Error is thrown when a static call attempts to alter storage
 */
export class StaticCallStorageAlterError extends InstructionExecutionError {
  constructor() {
    super('Static calls cannot alter storage');
    this.name = 'StaticCallStorageAlterError';
  }
}
