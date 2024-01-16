import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';

/**
 * Opcode base class
 */
export abstract class Instruction {
  abstract execute(machineState: AvmMachineState, stateManager: AvmStateManager): void;
}
