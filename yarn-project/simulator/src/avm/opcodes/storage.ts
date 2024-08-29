import type { AvmContext } from '../avm_context.js';
import { Field, TypeTag } from '../avm_memory_types.js';
import { StaticCallAlterationError } from '../errors.js';
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
  ];

  constructor(protected indirect: number, protected aOffset: number, protected bOffset: number) {
    super();
  }
}

export class SStore extends BaseStorageInstruction {
  static readonly type: string = 'SSTORE';
  static readonly opcode = Opcode.SSTORE;

  constructor(indirect: number, srcOffset: number, slotOffset: number) {
    super(indirect, srcOffset, slotOffset);
  }

  public async execute(context: AvmContext): Promise<void> {
    if (context.environment.isStaticCall) {
      throw new StaticCallAlterationError();
    }

    const memoryOperations = { reads: 2, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost({ ...memoryOperations }));

    const [srcOffset, slotOffset] = Addressing.fromWire(this.indirect).resolve([this.aOffset, this.bOffset], memory);
    memory.checkTag(TypeTag.FIELD, slotOffset);
    memory.checkTag(TypeTag.FIELD, srcOffset);

    const slot = memory.get(slotOffset).toFr();
    const value = memory.get(srcOffset).toFr();
    context.persistableState.writeStorage(context.environment.storageAddress, slot, value);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}

export class SLoad extends BaseStorageInstruction {
  static readonly type: string = 'SLOAD';
  static readonly opcode = Opcode.SLOAD;

  constructor(indirect: number, slotOffset: number, dstOffset: number) {
    super(indirect, slotOffset, dstOffset);
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { writes: 1, reads: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost({ ...memoryOperations }));

    const [slotOffset, dstOffset] = Addressing.fromWire(this.indirect).resolve([this.aOffset, this.bOffset], memory);
    memory.checkTag(TypeTag.FIELD, slotOffset);

    const slot = memory.get(slotOffset).toFr();
    const value = await context.persistableState.readStorage(context.environment.storageAddress, slot);
    memory.set(dstOffset, new Field(value));

    context.machineState.incrementPc();
    memory.assert(memoryOperations);
  }
}
