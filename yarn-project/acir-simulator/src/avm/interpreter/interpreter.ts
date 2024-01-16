// import { AvmContext } from "../avm_machineState.js";
import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmMessageCallResult } from '../avm_message_call_result.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from '../opcodes/index.js';

/**
 * Avm Interpreter
 *
 * Executes an Avm context
 */
export class AvmInterpreter {
  private instructions: Instruction[] = [];
  private machineState: AvmMachineState;
  private stateManager: AvmStateManager;

  constructor(machineState: AvmMachineState, stateManager: AvmStateManager, bytecode: Instruction[]) {
    this.machineState = machineState;
    this.stateManager = stateManager;
    this.instructions = bytecode;
  }

  /**
   * Run the avm
   * @returns bool - successful execution will return true
   *               - reverted execution will return false
   *               - any other panic will throw
   */
  run(): AvmMessageCallResult {
    try {
      for (const instruction of this.instructions) {
        instruction.execute(this.machineState, this.stateManager);
      }

      const returnData = this.machineState.getReturnData();
      return AvmMessageCallResult.success(returnData);
    } catch (e) {
      // TODO: This should only accept AVM defined errors, anything else SHOULD be thrown upstream
      const revertData = this.machineState.getReturnData();
      return AvmMessageCallResult.revert(revertData);
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
