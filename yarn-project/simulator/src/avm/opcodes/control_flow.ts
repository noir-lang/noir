import type { AvmContext } from '../avm_context.js';
import { IntegralValue } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';

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

  async execute(context: AvmContext): Promise<void> {
    const output = context.machineState.memory.getSlice(this.returnOffset, this.copySize).map(word => word.toFr());

    context.machineState.return(output);
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

  async execute(context: AvmContext): Promise<void> {
    const output = context.machineState.memory
      .getSlice(this.returnOffset, this.returnOffset + this.retSize)
      .map(word => word.toFr());

    context.machineState.revert(output);
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

  async execute(context: AvmContext): Promise<void> {
    context.machineState.pc = this.jumpOffset;
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

  async execute(context: AvmContext): Promise<void> {
    const condition = context.machineState.memory.getAs<IntegralValue>(this.condOffset);

    // TODO: reconsider this casting
    if (condition.toBigInt() == 0n) {
      context.machineState.incrementPc();
    } else {
      context.machineState.pc = this.loc;
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

  async execute(context: AvmContext): Promise<void> {
    context.machineState.internalCallStack.push(context.machineState.pc + 1);
    context.machineState.pc = this.loc;
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

  async execute(context: AvmContext): Promise<void> {
    const jumpOffset = context.machineState.internalCallStack.pop();
    if (jumpOffset === undefined) {
      throw new InstructionExecutionError('Internal call empty!');
    }
    context.machineState.pc = jumpOffset;
  }
}
