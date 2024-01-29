import { AvmMachineState } from '../avm_machine_state.js';
import { IntegralValue, TypeTag } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/index.js';
import { Instruction } from './instruction.js';

export class And extends Instruction {
  static type: string = 'AND';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.getAs<IntegralValue>(this.aOffset);
    const b = machineState.memory.getAs<IntegralValue>(this.bOffset);

    const res = a.and(b);
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}

export class Or extends Instruction {
  static type: string = 'OR';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.getAs<IntegralValue>(this.aOffset);
    const b = machineState.memory.getAs<IntegralValue>(this.bOffset);

    const res = a.or(b);
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}

export class Xor extends Instruction {
  static type: string = 'XOR';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.getAs<IntegralValue>(this.aOffset);
    const b = machineState.memory.getAs<IntegralValue>(this.bOffset);

    const res = a.xor(b);
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}

export class Not extends Instruction {
  static type: string = 'NOT';
  static numberOfOperands = 2;

  constructor(private inTag: TypeTag, private aOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset);

    const a = machineState.memory.getAs<IntegralValue>(this.aOffset);

    const res = a.not();
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}

export class Shl extends Instruction {
  static type: string = 'SHL';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.getAs<IntegralValue>(this.aOffset);
    const b = machineState.memory.getAs<IntegralValue>(this.bOffset);

    const res = a.shl(b);
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}

export class Shr extends Instruction {
  static type: string = 'SHR';
  static numberOfOperands = 3;

  constructor(private inTag: TypeTag, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset, this.bOffset);

    const a = machineState.memory.getAs<IntegralValue>(this.aOffset);
    const b = machineState.memory.getAs<IntegralValue>(this.bOffset);

    const res = a.shr(b);
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}
