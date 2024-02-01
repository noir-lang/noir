import { AvmMachineState } from '../avm_machine_state.js';
import { IntegralValue } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/journal.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction, InstructionExecutionError } from './instruction.js';

export class Return extends Instruction {
  static type: string = 'RETURN';
  static readonly opcode: Opcode = Opcode.RETURN;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private returnOffset: number, private copySize: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const returnData = machineState.memory.getSlice(this.returnOffset, this.copySize).map(word => word.toFr());

    machineState.setReturnData(returnData);

    this.halt(machineState);
  }
}

export class Revert extends Instruction {
  static type: string = 'RETURN';
  static readonly opcode: Opcode = Opcode.REVERT;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private returnOffset: number, private retSize: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const returnData = machineState.memory
      .getSlice(this.returnOffset, this.returnOffset + this.retSize)
      .map(word => word.toFr());
    machineState.setReturnData(returnData);

    this.revert(machineState);
  }
}

export class Jump extends Instruction {
  static type: string = 'JUMP';
  static readonly opcode: Opcode = Opcode.JUMP;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [OperandType.UINT8, OperandType.UINT32];

  constructor(private jumpOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    machineState.pc = this.jumpOffset;
  }
}

export class JumpI extends Instruction {
  static type: string = 'JUMPI';
  static readonly opcode: Opcode = Opcode.JUMPI;

  // Instruction wire format with opcode.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private loc: number, private condOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const condition = machineState.memory.getAs<IntegralValue>(this.condOffset);

    // TODO: reconsider this casting
    if (condition.toBigInt() == 0n) {
      this.incrementPc(machineState);
    } else {
      machineState.pc = this.loc;
    }
  }
}

export class InternalCall extends Instruction {
  static readonly type: string = 'INTERNALCALL';
  static readonly opcode: Opcode = Opcode.INTERNALCALL;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [OperandType.UINT8, OperandType.UINT32];

  constructor(private loc: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    machineState.internalCallStack.push(machineState.pc + 1);
    machineState.pc = this.loc;
  }
}

export class InternalReturn extends Instruction {
  static readonly type: string = 'INTERNALRETURN';
  static readonly opcode: Opcode = Opcode.INTERNALRETURN;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [OperandType.UINT8];

  constructor() {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const jumpOffset = machineState.internalCallStack.pop();
    if (jumpOffset === undefined) {
      throw new InstructionExecutionError('Internal call empty!');
    }
    machineState.pc = jumpOffset;
  }
}
