import { Fr } from '@aztec/foundation/fields';

import type { AvmContext } from '../avm_context.js';
import { type Gas, getBaseGasCost, getMemoryGasCost, mulGas, sumGas } from '../avm_gas.js';
import { Field, type MemoryOperations } from '../avm_memory_types.js';
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

  protected gasCost(memoryOps: Partial<MemoryOperations & { indirect: number }>): Gas {
    const baseGasCost = mulGas(getBaseGasCost(this.opcode), this.size);
    const memoryGasCost = getMemoryGasCost(memoryOps);
    return sumGas(baseGasCost, memoryGasCost);
  }
}

export class SStore extends BaseStorageInstruction {
  static readonly type: string = 'SSTORE';
  static readonly opcode = Opcode.SSTORE;

  constructor(indirect: number, srcOffset: number, /*temporary*/ srcSize: number, slotOffset: number) {
    super(indirect, srcOffset, srcSize, slotOffset);
  }

  public async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const memoryOperations = { reads: this.size + 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [srcOffset, slotOffset] = Addressing.fromWire(this.indirect).resolve([this.aOffset, this.bOffset], memory);

    const slot = memory.get(slotOffset).toFr();
    const data = memory.getSlice(srcOffset, this.size).map(field => field.toFr());

    for (const [index, value] of Object.entries(data)) {
      const adjustedSlot = slot.add(new Fr(BigInt(index)));
      context.persistableState.writeStorage(context.environment.storageAddress, adjustedSlot, value);
    }

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}

export class SLoad extends BaseStorageInstruction {
  static readonly type: string = 'SLOAD';
  static readonly opcode = Opcode.SLOAD;

  constructor(indirect: number, slotOffset: number, size: number, dstOffset: number) {
    super(indirect, slotOffset, size, dstOffset);
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { writes: this.size, reads: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [aOffset, size, bOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.aOffset, this.size, this.bOffset],
      memory,
    );

    const slot = memory.get(aOffset);

    // Write each read value from storage into memory
    for (let i = 0; i < size; i++) {
      const data: Fr = await context.persistableState.readStorage(
        context.environment.storageAddress,
        new Fr(slot.toBigInt() + BigInt(i)),
      );

      memory.set(bOffset + i, new Field(data));
    }

    context.machineState.incrementPc();
    memory.assert(memoryOperations);
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
