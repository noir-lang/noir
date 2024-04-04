import type { AvmContext } from '../avm_context.js';
import { type IntegralValue } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';

export class Jump extends Instruction {
  static type: string = 'JUMP';
  static readonly opcode: Opcode = Opcode.JUMP;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [OperandType.UINT8, OperandType.UINT32];

  constructor(private jumpOffset: number) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    context.machineState.consumeGas(this.gasCost());

    context.machineState.pc = this.jumpOffset;

    context.machineState.memory.assert({});
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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: 1, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const condition = memory.getAs<IntegralValue>(this.condOffset);

    // TODO: reconsider this casting
    if (condition.toBigInt() == 0n) {
      context.machineState.incrementPc();
    } else {
      context.machineState.pc = this.loc;
    }

    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    context.machineState.consumeGas(this.gasCost());

    context.machineState.internalCallStack.push(context.machineState.pc + 1);
    context.machineState.pc = this.loc;

    context.machineState.memory.assert({});
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

  public async execute(context: AvmContext): Promise<void> {
    context.machineState.consumeGas(this.gasCost());

    const jumpOffset = context.machineState.internalCallStack.pop();
    if (jumpOffset === undefined) {
      throw new InstructionExecutionError('Internal call stack empty!');
    }
    context.machineState.pc = jumpOffset;

    context.machineState.memory.assert({});
  }
}
