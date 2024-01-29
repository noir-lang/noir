import { AvmMachineState } from '../avm_machine_state.js';
import { AvmJournal } from '../journal/index.js';
import { Instruction } from './instruction.js';

export class Add extends Instruction {
  static type: string = 'ADD';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.add(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Sub extends Instruction {
  static type: string = 'SUB';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.sub(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Mul extends Instruction {
  static type: string = 'MUL';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.mul(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

/** -*/
export class Div extends Instruction {
  static type: string = 'DIV';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.div(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}
