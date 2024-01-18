import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';

export const AVM_OPERAND_BYTE_LENGTH = 4;
export const AVM_OPCODE_BYTE_LENGTH = 1;

/**
 * Opcode base class
 */
export abstract class Instruction {
  abstract execute(machineState: AvmMachineState, stateManager: AvmStateManager): void;

  incrementPc(machineState: AvmMachineState): void {
    machineState.pc++;
  }

  halt(machineState: AvmMachineState): void {
    machineState.halted = true;
  }
}
