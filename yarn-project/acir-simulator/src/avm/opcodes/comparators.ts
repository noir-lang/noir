import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/index.js';
import { Instruction } from './instruction.js';

export class Eq extends Instruction {
  static type: string = 'EQ';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _journal: AvmJournal): void {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = new Field(a.toBigInt() == b.toBigInt() ? 1 : 0);
    machineState.memory.set(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Lt extends Instruction {
  static type: string = 'Lt';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _journal: AvmJournal): void {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = new Field(a.toBigInt() < b.toBigInt() ? 1 : 0);
    machineState.memory.set(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Lte extends Instruction {
  static type: string = 'LTE';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _journal: AvmJournal): void {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = new Field(a.toBigInt() < b.toBigInt() ? 1 : 0);
    machineState.memory.set(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}
