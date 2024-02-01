import { strict as assert } from 'assert';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmMessageCallResult } from '../avm_message_call_result.js';
import { AvmJournal } from '../journal/index.js';
import { Instruction, InstructionExecutionError } from '../opcodes/instruction.js';

/**
 * Run the avm
 * @returns bool - successful execution will return true
 *               - reverted execution will return false
 *               - any other panic will throw
 */
export async function executeAvm(
  machineState: AvmMachineState,
  journal: AvmJournal,
  instructions: Instruction[] = [],
): Promise<AvmMessageCallResult> {
  assert(instructions.length > 0);

  try {
    while (!machineState.halted) {
      const instruction = instructions[machineState.pc];
      assert(!!instruction); // This should never happen

      await instruction.execute(machineState, journal);

      if (machineState.pc >= instructions.length) {
        throw new InvalidProgramCounterError(machineState.pc, /*max=*/ instructions.length);
      }
    }

    const returnData = machineState.getReturnData();
    if (machineState.reverted) {
      return AvmMessageCallResult.revert(returnData);
    }

    return AvmMessageCallResult.success(returnData);
  } catch (e) {
    if (!(e instanceof AvmInterpreterError || e instanceof InstructionExecutionError)) {
      throw e;
    }

    const revertData = machineState.getReturnData();
    return AvmMessageCallResult.revert(revertData, /*revertReason=*/ e);
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
