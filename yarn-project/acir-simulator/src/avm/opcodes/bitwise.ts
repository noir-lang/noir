import { AvmMachineState } from '../avm_machine_state.js';
import { IntegralValue } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/index.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';
import { ThreeOperandInstruction, TwoOperandInstruction } from './instruction_impl.js';

export class And extends ThreeOperandInstruction {
  static readonly type: string = 'AND';
  static readonly opcode = Opcode.AND;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
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

export class Or extends ThreeOperandInstruction {
  static readonly type: string = 'OR';
  static readonly opcode = Opcode.OR;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
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

export class Xor extends ThreeOperandInstruction {
  static readonly type: string = 'XOR';
  static readonly opcode = Opcode.XOR;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
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

export class Not extends TwoOperandInstruction {
  static readonly type: string = 'NOT';
  static readonly opcode = Opcode.NOT;

  constructor(indirect: number, inTag: number, aOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, dstOffset);
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    Instruction.checkTags(machineState, this.inTag, this.aOffset);

    const a = machineState.memory.getAs<IntegralValue>(this.aOffset);

    const res = a.not();
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}

export class Shl extends ThreeOperandInstruction {
  static readonly type: string = 'SHL';
  static readonly opcode = Opcode.SHL;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
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

export class Shr extends ThreeOperandInstruction {
  static readonly type: string = 'SHR';
  static readonly opcode = Opcode.SHR;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
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
