import type { AvmContext } from '../avm_context.js';
import { type IntegralValue } from '../avm_memory_types.js';
import { Opcode } from '../serialization/instruction_serialization.js';
import { ThreeOperandInstruction, TwoOperandInstruction } from './instruction_impl.js';

abstract class ThreeOperandBitwiseInstruction extends ThreeOperandInstruction {
  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 2, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = memory.getAs<IntegralValue>(this.aOffset);
    const b = memory.getAs<IntegralValue>(this.bOffset);

    const res = this.compute(a, b);
    memory.set(this.dstOffset, res);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }

  protected abstract compute(a: IntegralValue, b: IntegralValue): IntegralValue;
}

export class And extends ThreeOperandBitwiseInstruction {
  static readonly type: string = 'AND';
  static readonly opcode = Opcode.AND;

  protected compute(a: IntegralValue, b: IntegralValue): IntegralValue {
    return a.and(b);
  }
}

export class Or extends ThreeOperandBitwiseInstruction {
  static readonly type: string = 'OR';
  static readonly opcode = Opcode.OR;

  protected compute(a: IntegralValue, b: IntegralValue): IntegralValue {
    return a.or(b);
  }
}

export class Xor extends ThreeOperandBitwiseInstruction {
  static readonly type: string = 'XOR';
  static readonly opcode = Opcode.XOR;

  protected compute(a: IntegralValue, b: IntegralValue): IntegralValue {
    return a.xor(b);
  }
}

export class Shl extends ThreeOperandBitwiseInstruction {
  static readonly type: string = 'SHL';
  static readonly opcode = Opcode.SHL;

  protected compute(a: IntegralValue, b: IntegralValue): IntegralValue {
    return a.shl(b);
  }
}

export class Shr extends ThreeOperandBitwiseInstruction {
  static readonly type: string = 'SHR';
  static readonly opcode = Opcode.SHR;

  protected compute(a: IntegralValue, b: IntegralValue): IntegralValue {
    return a.shr(b);
  }
}

export class Not extends TwoOperandInstruction {
  static readonly type: string = 'NOT';
  static readonly opcode = Opcode.NOT;

  constructor(indirect: number, inTag: number, aOffset: number, dstOffset: number) {
    super(indirect, inTag, aOffset, dstOffset);
  }

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 1, writes: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    memory.checkTags(this.inTag, this.aOffset);

    const a = memory.getAs<IntegralValue>(this.aOffset);

    const res = a.not();
    memory.set(this.dstOffset, res);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
