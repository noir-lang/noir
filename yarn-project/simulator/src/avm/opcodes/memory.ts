import type { AvmContext } from '../avm_context.js';
import { Field, TaggedMemory, TypeTag } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';
import { TwoOperandInstruction } from './instruction_impl.js';

export class Set extends Instruction {
  static readonly type: string = 'SET';
  static readonly opcode: Opcode = Opcode.SET;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT128,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private inTag: number, private value: bigint, private dstOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    // Per the YP, the tag cannot be a field.
    if ([TypeTag.FIELD, TypeTag.UNINITIALIZED, TypeTag.INVALID].includes(this.inTag)) {
      throw new InstructionExecutionError(`Invalid tag ${TypeTag[this.inTag]} for SET.`);
    }

    const res = TaggedMemory.integralFromTag(this.value, this.inTag);
    context.machineState.memory.set(this.dstOffset, res);

    context.machineState.incrementPc();
  }
}

export class CMov extends Instruction {
  static readonly type: string = 'CMOV';
  static readonly opcode: Opcode = Opcode.CMOV;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private aOffset: number,
    private bOffset: number,
    private condOffset: number,
    private dstOffset: number,
  ) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const a = context.machineState.memory.get(this.aOffset);
    const b = context.machineState.memory.get(this.bOffset);
    const cond = context.machineState.memory.get(this.condOffset);

    // TODO: reconsider toBigInt() here
    context.machineState.memory.set(this.dstOffset, cond.toBigInt() > 0 ? a : b);

    context.machineState.incrementPc();
  }
}

export class Cast extends TwoOperandInstruction {
  static readonly type: string = 'CAST';
  static readonly opcode = Opcode.CAST;

  constructor(indirect: number, dstTag: number, aOffset: number, dstOffset: number) {
    super(indirect, dstTag, aOffset, dstOffset);
  }

  async execute(context: AvmContext): Promise<void> {
    const a = context.machineState.memory.get(this.aOffset);

    // TODO: consider not using toBigInt()
    const casted =
      this.inTag == TypeTag.FIELD ? new Field(a.toBigInt()) : TaggedMemory.integralFromTag(a.toBigInt(), this.inTag);

    context.machineState.memory.set(this.dstOffset, casted);

    context.machineState.incrementPc();
  }
}

export class Mov extends Instruction {
  static readonly type: string = 'MOV';
  static readonly opcode: Opcode = Opcode.MOV;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private srcOffset: number, private dstOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const a = context.machineState.memory.get(this.srcOffset);

    context.machineState.memory.set(this.dstOffset, a);

    context.machineState.incrementPc();
  }
}

export class CalldataCopy extends Instruction {
  static readonly type: string = 'CALLDATACOPY';
  static readonly opcode: Opcode = Opcode.CALLDATACOPY;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private cdOffset: number, private copySize: number, private dstOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const transformedData = context.environment.calldata
      .slice(this.cdOffset, this.cdOffset + this.copySize)
      .map(f => new Field(f));
    context.machineState.memory.setSlice(this.dstOffset, transformedData);

    context.machineState.incrementPc();
  }
}
