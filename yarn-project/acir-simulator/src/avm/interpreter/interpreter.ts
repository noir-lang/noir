import { Fr } from '@aztec/foundation/fields';

import { strict as assert } from 'assert';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmMessageCallResult } from '../avm_message_call_result.js';
import { AvmJournal } from '../journal/index.js';
import { Instruction } from '../opcodes/index.js';

/**
 * Avm Interpreter
 *
 * Executes an Avm context
 */
export class AvmInterpreter {
  private instructions: Instruction[] = [];
  private machineState: AvmMachineState;
  private journal: AvmJournal;

  constructor(machineState: AvmMachineState, stateManager: AvmJournal, instructions: Instruction[]) {
    this.machineState = machineState;
    this.journal = stateManager;
    this.instructions = instructions;
  }

  /**
   * Run the avm
   * @returns bool - successful execution will return true
   *               - reverted execution will return false
   *               - any other panic will throw
   */
  async run(): Promise<AvmMessageCallResult> {
    assert(this.instructions.length > 0);

    try {
      while (!this.machineState.halted) {
        const instruction = this.instructions[this.machineState.pc];
        assert(!!instruction); // This should never happen

        await instruction.execute(this.machineState, this.journal);

        if (this.machineState.pc >= this.instructions.length) {
          throw new InvalidProgramCounterError(this.machineState.pc, /*max=*/ this.instructions.length);
        }
      }

      const returnData = this.machineState.getReturnData();
      return AvmMessageCallResult.success(returnData);
    } catch (_e) {
      if (!(_e instanceof AvmInterpreterError)) {
        throw _e;
      }

      const revertReason: AvmInterpreterError = _e;
      const revertData = this.machineState.getReturnData();
      return AvmMessageCallResult.revert(revertData, revertReason);
    }
  }

  /**
   * Get the return data from avm execution
   * TODO: this should fail if the code has not been executed
   *  - maybe move the return in run into a variable and track it
   */
  returnData(): Fr[] {
    return this.machineState.getReturnData();
  }
}

/**
 * Avm-specific errors should derive from this
 */
export abstract class AvmInterpreterError extends Error {
  constructor(message: string, ...rest: any[]) {
    super(message, ...rest);
    this.name = 'AvmInterpreterError';
  }
}

/**
 * Error is thrown when the program counter goes to an invalid location.
 * There is no instruction at the provided pc
 */
export class InvalidProgramCounterError extends AvmInterpreterError {
  constructor(pc: number, max: number) {
    super(`Invalid program counter ${pc}, max is ${max}`);
    this.name = 'InvalidProgramCounterError';
  }
}
