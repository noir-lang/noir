import type { AvmContext } from '../avm_context.js';
import { Field, TypeTag } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';
import { ThreeOperandInstruction } from './instruction_impl.js';

export class Add extends ThreeOperandInstruction {
  static readonly type: string = 'ADD';
  static readonly opcode = Opcode.ADD;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    context.machineState.memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    const dest = a.add(b);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}

export class Sub extends ThreeOperandInstruction {
  static readonly type: string = 'SUB';
  static readonly opcode = Opcode.SUB;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    const dest = a.sub(b);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}

export class Mul extends ThreeOperandInstruction {
  static type: string = 'MUL';
  static readonly opcode = Opcode.MUL;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    const dest = a.mul(b);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}

export class Div extends ThreeOperandInstruction {
  static type: string = 'DIV';
  static readonly opcode = Opcode.DIV;

  constructor(indirect: number, inTag: number, aOffset: number, bOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, bOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    const dest = a.div(b);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}

export class FieldDiv extends Instruction {
  static type: string = 'FDIV';
  static readonly opcode = Opcode.FDIV;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private aOffset: number, private bOffset: number, private dstOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    context.machineState.memory.checkTags(TypeTag.FIELD, this.aOffset, this.bOffset);

    const a = context.machineState.memory.getAs<Field>(this.aOffset);
    const b = context.machineState.memory.getAs<Field>(this.bOffset);

    const dest = a.fdiv(b);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}
