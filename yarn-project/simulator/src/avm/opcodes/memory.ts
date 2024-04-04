import type { AvmContext } from '../avm_context.js';
import { type Gas, GasCostConstants, getGasCostMultiplierFromTypeTag, makeGasCost } from '../avm_gas.js';
import { Field, TaggedMemory, TypeTag } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { BufferCursor } from '../serialization/buffer_cursor.js';
import { Opcode, OperandType, deserialize, serialize } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';
import { TwoOperandInstruction } from './instruction_impl.js';

const TAG_TO_OPERAND_TYPE = new Map<TypeTag, OperandType>([
  [TypeTag.UINT8, OperandType.UINT8],
  [TypeTag.UINT16, OperandType.UINT16],
  [TypeTag.UINT32, OperandType.UINT32],
  [TypeTag.UINT64, OperandType.UINT64],
  [TypeTag.UINT128, OperandType.UINT128],
]);

function getOperandTypeFromInTag(inTag: number | bigint): OperandType {
  inTag = inTag as number;
  const tagOperandType = TAG_TO_OPERAND_TYPE.get(inTag);
  if (tagOperandType === undefined) {
    throw new Error(`Invalid tag ${inTag} for SET.`);
  }
  return tagOperandType;
}

export class Set extends Instruction {
  static readonly type: string = 'SET';
  static readonly opcode: Opcode = Opcode.SET;

  private static readonly wireFormatBeforeConst: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT8,
  ];
  private static readonly wireFormatAfterConst: OperandType[] = [OperandType.UINT32];

  constructor(
    private indirect: number,
    private inTag: number,
    private value: bigint | number,
    private dstOffset: number,
  ) {
    super();
  }

  /** We need to use a custom serialize function because of the variable length of the value. */
  public serialize(): Buffer {
    const format: OperandType[] = [
      ...Set.wireFormatBeforeConst,
      getOperandTypeFromInTag(this.inTag),
      ...Set.wireFormatAfterConst,
    ];
    return serialize(format, this);
  }

  /** We need to use a custom deserialize function because of the variable length of the value. */
  public static deserialize(this: typeof Set, buf: BufferCursor | Buffer): Set {
    if (buf instanceof Buffer) {
      buf = new BufferCursor(buf);
    }
    const beforeConst = deserialize(buf, Set.wireFormatBeforeConst);
    const tag = beforeConst[beforeConst.length - 1];
    const val = deserialize(buf, [getOperandTypeFromInTag(tag)]);
    const afterConst = deserialize(buf, Set.wireFormatAfterConst);
    const res = [...beforeConst, ...val, ...afterConst];
    const args = res.slice(1) as ConstructorParameters<typeof Set>; // Remove opcode.
    return new this(...args);
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

  protected gasCost(): Gas {
    return makeGasCost({ l2Gas: GasCostConstants.SET_COST_PER_BYTE * getGasCostMultiplierFromTypeTag(this.inTag) });
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
    const [srcOffset, dstOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.srcOffset, this.dstOffset],
      context.machineState.memory,
    );

    const a = context.machineState.memory.get(srcOffset);

    context.machineState.memory.set(dstOffset, a);

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
    const [dstOffset] = Addressing.fromWire(this.indirect).resolve([this.dstOffset], context.machineState.memory);

    const transformedData = context.environment.calldata
      .slice(this.cdOffset, this.cdOffset + this.copySize)
      .map(f => new Field(f));

    context.machineState.memory.setSlice(dstOffset, transformedData);

    context.machineState.incrementPc();
  }

  protected gasCost(): Gas {
    return makeGasCost({ l2Gas: GasCostConstants.CALLDATACOPY_COST_PER_BYTE * this.copySize });
  }
}
