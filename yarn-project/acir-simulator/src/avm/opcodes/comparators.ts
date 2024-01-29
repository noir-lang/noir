import { AvmMachineState } from '../avm_machine_state.js';
import { TypeTag } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/index.js';
import { Instruction } from './instruction.js';

export class Eq extends Instruction {
  static type: string = 'EQ';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    // Result will be of the same type as 'a'.
    const dest = a.build(a.equals(b) ? 1n : 0n);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Lt extends Instruction {
  static type: string = 'Lt';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    // Result will be of the same type as 'a'.
    const dest = a.build(a.lt(b) ? 1n : 0n);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Lte extends Instruction {
  static type: string = 'LTE';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    // Result will be of the same type as 'a'.
    const dest = a.build(a.equals(b) || a.lt(b) ? 1n : 0n);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}
