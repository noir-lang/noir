import type { AvmContext } from '../avm_context.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { ThreeOperandInstruction } from './instruction_impl.js';

export class Eq extends ThreeOperandInstruction {
  static readonly type: string = 'EQ';
  static readonly opcode = Opcode.EQ;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    context.machineState.memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    // Result will be of the same type as 'a'.
    const dest = a.build(a.equals(b) ? 1n : 0n);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}

export class Lt extends ThreeOperandInstruction {
  static readonly type: string = 'LT';
  static readonly opcode = Opcode.LT;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    context.machineState.memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    // Result will be of the same type as 'a'.
    const dest = a.build(a.lt(b) ? 1n : 0n);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}

export class Lte extends ThreeOperandInstruction {
  static readonly type: string = 'LTE';
  static readonly opcode = Opcode.LTE;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    context.machineState.memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    // Result will be of the same type as 'a'.
    const dest = a.build(a.equals(b) || a.lt(b) ? 1n : 0n);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}
