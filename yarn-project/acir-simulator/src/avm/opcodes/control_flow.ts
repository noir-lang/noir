import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

/** - */
export class Return implements Instruction {
  static type: string = 'RETURN';
  static numberOfOperands = 2;

  constructor(private returnOffset: number, private copySize: number) {}

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const returnData = machineState.readMemoryChunk(this.returnOffset, this.returnOffset + this.copySize);
    machineState.setReturnData(returnData);
  }
}
