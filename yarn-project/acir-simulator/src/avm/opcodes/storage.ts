import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/journal.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction, InstructionExecutionError } from './instruction.js';

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

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    if (machineState.executionEnvironment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const slot = machineState.memory.get(this.aOffset);
    const data = machineState.memory.get(this.bOffset);

    journal.writeStorage(
      machineState.executionEnvironment.storageAddress,
      new Fr(slot.toBigInt()),
      new Fr(data.toBigInt()),
    );

    this.incrementPc(machineState);
  }
}

export class SLoad extends BaseStorageInstruction {
  static readonly type: string = 'SLOAD';
  static readonly opcode = Opcode.SLOAD;

  constructor(indirect: number, slotOffset: number, dstOffset: number) {
    super(indirect, slotOffset, dstOffset);
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    const slot = machineState.memory.get(this.aOffset);

    const data: Fr = await journal.readStorage(
      machineState.executionEnvironment.storageAddress,
      new Fr(slot.toBigInt()),
    );

    machineState.memory.set(this.bOffset, new Field(data));

    this.incrementPc(machineState);
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
