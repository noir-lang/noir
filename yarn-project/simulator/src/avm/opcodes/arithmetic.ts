import type { AvmContext } from '../avm_context.js';
import { getBaseGasCost, getGasCostForTypeTag, getMemoryGasCost, sumGas } from '../avm_gas.js';
import { type Field, type MemoryOperations, type MemoryValue, TypeTag } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';
import { ThreeOperandInstruction } from './instruction_impl.js';

export abstract class ThreeOperandArithmeticInstruction extends ThreeOperandInstruction {
  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 2, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = memory.get(this.aOffset);
    const b = memory.get(this.bOffset);

    const dest = this.compute(a, b);
    memory.set(this.dstOffset, dest);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }

  protected gasCost(memoryOps: Partial<MemoryOperations & { indirect: number }>) {
    const baseGasCost = getGasCostForTypeTag(this.inTag, getBaseGasCost(this.opcode));
    const memoryGasCost = getMemoryGasCost(memoryOps);
    return sumGas(baseGasCost, memoryGasCost);
  }

  protected abstract compute(a: MemoryValue, b: MemoryValue): MemoryValue;
}

export class Add extends ThreeOperandArithmeticInstruction {
  static readonly type: string = 'ADD';
  static readonly opcode = Opcode.ADD;

  protected compute(a: MemoryValue, b: MemoryValue): MemoryValue {
    return a.add(b);
  }
}

export class Sub extends ThreeOperandArithmeticInstruction {
  static readonly type: string = 'SUB';
  static readonly opcode = Opcode.SUB;

  protected compute(a: MemoryValue, b: MemoryValue): MemoryValue {
    return a.sub(b);
  }
}

export class Mul extends ThreeOperandArithmeticInstruction {
  static type: string = 'MUL';
  static readonly opcode = Opcode.MUL;

  protected compute(a: MemoryValue, b: MemoryValue): MemoryValue {
    return a.mul(b);
  }
}

export class Div extends ThreeOperandArithmeticInstruction {
  static type: string = 'DIV';
  static readonly opcode = Opcode.DIV;

  protected compute(a: MemoryValue, b: MemoryValue): MemoryValue {
    return a.div(b);
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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 2, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    memory.checkTags(TypeTag.FIELD, this.aOffset, this.bOffset);

    const a = memory.getAs<Field>(this.aOffset);
    const b = memory.getAs<Field>(this.bOffset);

    const dest = a.fdiv(b);
    memory.set(this.dstOffset, dest);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }

  protected gasCost(memoryOps: Partial<MemoryOperations & { indirect: number }>) {
    const baseGasCost = getGasCostForTypeTag(TypeTag.FIELD, getBaseGasCost(this.opcode));
    const memoryGasCost = getMemoryGasCost(memoryOps);
    return sumGas(baseGasCost, memoryGasCost);
  }
}
