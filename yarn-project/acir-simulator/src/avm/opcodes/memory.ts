import { AvmMachineState } from '../avm_machine_state.js';
import { Field, TaggedMemory, TypeTag } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/index.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction, InstructionExecutionError } from './instruction.js';
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

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    // Per the YP, the tag cannot be a field.
    if ([TypeTag.FIELD, TypeTag.UNINITIALIZED, TypeTag.INVALID].includes(this.inTag)) {
      throw new InstructionExecutionError(`Invalid tag ${TypeTag[this.inTag]} for SET.`);
    }

    const res = TaggedMemory.integralFromTag(this.value, this.inTag);
    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
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

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);
    const cond = machineState.memory.get(this.condOffset);

    // TODO: reconsider toBigInt() here
    machineState.memory.set(this.dstOffset, cond.toBigInt() > 0 ? a : b);

    this.incrementPc(machineState);
  }
}

export class Cast extends TwoOperandInstruction {
  static readonly type: string = 'CAST';
  static readonly opcode = Opcode.CAST;

  constructor(indirect: number, dstTag: number, aOffset: number, dstOffset: number) {
    super(indirect, dstTag, aOffset, dstOffset);
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.aOffset);

    // TODO: consider not using toBigInt()
    const casted =
      this.inTag == TypeTag.FIELD ? new Field(a.toBigInt()) : TaggedMemory.integralFromTag(a.toBigInt(), this.inTag);

    machineState.memory.set(this.dstOffset, casted);

    this.incrementPc(machineState);
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

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const a = machineState.memory.get(this.srcOffset);

    machineState.memory.set(this.dstOffset, a);

    this.incrementPc(machineState);
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

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const transformedData = machineState.executionEnvironment.calldata
      .slice(this.cdOffset, this.cdOffset + this.copySize)
      .map(f => new Field(f));
    machineState.memory.setSlice(this.dstOffset, transformedData);

    this.incrementPc(machineState);
  }
}
