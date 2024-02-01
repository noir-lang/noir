import { AvmMachineState } from '../avm_machine_state.js';
import { AvmJournal } from '../journal/index.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { ThreeOperandInstruction } from './instruction_impl.js';

export class Add extends ThreeOperandInstruction {
  static readonly type: string = 'ADD';
  static readonly opcode = Opcode.ADD;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.add(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Sub extends ThreeOperandInstruction {
  static readonly type: string = 'SUB';
  static readonly opcode = Opcode.SUB;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.sub(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Mul extends ThreeOperandInstruction {
  static type: string = 'MUL';
  static readonly opcode = Opcode.MUL;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.mul(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}

export class Div extends ThreeOperandInstruction {
  static type: string = 'DIV';
  static readonly opcode = Opcode.DIV;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);

    const dest = a.div(b);
    machineState.memory.set(this.dstOffset, dest);

    this.incrementPc(machineState);
  }
}
