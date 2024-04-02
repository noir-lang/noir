import type { AvmContext } from '../avm_context.js';
import { type GasCost, GasCostConstants, getGasCostMultiplierFromTypeTag, makeGasCost } from '../avm_gas_cost.js';
import { type Field, type MemoryValue, TypeTag } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing, AddressingMode } from './addressing_mode.js';
import { Instruction } from './instruction.js';
import { ThreeOperandInstruction } from './instruction_impl.js';

export abstract class ThreeOperandArithmeticInstruction extends ThreeOperandInstruction {
  async execute(context: AvmContext): Promise<void> {
    context.machineState.memory.checkTags(this.inTag, this.aOffset, this.bOffset);

    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);

    const dest = this.compute(a, b);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }

  protected gasCost(): GasCost {
    const indirectCount = Addressing.fromWire(this.indirect).modePerOperand.filter(
      mode => mode === AddressingMode.INDIRECT,
    ).length;

    const l2Gas =
      indirectCount * GasCostConstants.ARITHMETIC_COST_PER_INDIRECT_ACCESS +
      getGasCostMultiplierFromTypeTag(this.inTag) * GasCostConstants.ARITHMETIC_COST_PER_BYTE;
    return makeGasCost({ l2Gas });
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

  async execute(context: AvmContext): Promise<void> {
    context.machineState.memory.checkTags(TypeTag.FIELD, this.aOffset, this.bOffset);

    const a = context.machineState.memory.getAs<Field>(this.aOffset);
    const b = context.machineState.memory.getAs<Field>(this.bOffset);

    const dest = a.fdiv(b);
    context.machineState.memory.set(this.dstOffset, dest);

    context.machineState.incrementPc();
  }
}
