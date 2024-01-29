import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { IntegralValue } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/journal.js';
import { Instruction, InstructionExecutionError } from './instruction.js';

export class Return extends Instruction {
  static type: string = 'RETURN';
  static numberOfOperands = 2;

  constructor(private returnOffset: number, private copySize: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const returnData = machineState.memory
      .getSlice(this.returnOffset, this.copySize)
      .map(word => new Fr(word.toBigInt()));

    machineState.setReturnData(returnData);

    this.halt(machineState);
  }
}

export class Jump extends Instruction {
  static type: string = 'JUMP';
  static numberOfOperands = 1;

  constructor(private jumpOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    machineState.pc = this.jumpOffset;
  }
}

export class JumpI extends Instruction {
  static type: string = 'JUMPI';
  static numberOfOperands = 1;

  constructor(private jumpOffset: number, private condOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    const condition = machineState.memory.getAs<IntegralValue>(this.condOffset);

    // TODO: reconsider this casting
    if (condition.toBigInt() == 0n) {
      this.incrementPc(machineState);
    } else {
      machineState.pc = this.jumpOffset;
    }
  }
}

export class InternalCall extends Instruction {
  static type: string = 'INTERNALCALL';
  static numberOfOperands = 1;

  constructor(private jumpOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, _journal: AvmJournal): Promise<void> {
    machineState.internalCallStack.push(machineState.pc + 1);
    machineState.pc = this.jumpOffset;
  }
}

export class InternalReturn extends Instruction {
  static type: string = 'INTERNALRETURN';
  static numberOfOperands = 0;

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
