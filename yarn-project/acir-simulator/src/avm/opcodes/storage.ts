import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { AvmInterpreterError } from '../interpreter/interpreter.js';
import { AvmJournal } from '../journal/journal.js';
import { Instruction } from './instruction.js';

/** - */
export class SStore extends Instruction {
  static type: string = 'SSTORE';
  static numberOfOperands = 2;

  constructor(private slotOffset: number, private dataOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    if (machineState.executionEnvironment.isStaticCall) {
      throw new StaticCallStorageAlterError();
    }

    const slot = machineState.memory.get(this.slotOffset);
    const data = machineState.memory.get(this.dataOffset);

    journal.writeStorage(
      machineState.executionEnvironment.storageAddress,
      new Fr(slot.toBigInt()),
      new Fr(data.toBigInt()),
    );

    this.incrementPc(machineState);
  }
}

/** - */
export class SLoad extends Instruction {
  static type: string = 'SLOAD';
  static numberOfOperands = 2;

  constructor(private slotOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    const slot = machineState.memory.get(this.slotOffset);

    const data: Fr = await journal.readStorage(
      machineState.executionEnvironment.storageAddress,
      new Fr(slot.toBigInt()),
    );

    machineState.memory.set(this.dstOffset, new Field(data));

    this.incrementPc(machineState);
  }
}

/**
 * Error is thrown when a static call attempts to alter storage
 */
export class StaticCallStorageAlterError extends AvmInterpreterError {
  constructor() {
    super('Static calls cannot alter storage');
    this.name = 'StaticCallStorageAlterError';
  }
}
