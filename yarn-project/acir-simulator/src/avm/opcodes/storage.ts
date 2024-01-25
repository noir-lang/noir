import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

/** - */
export class SStore extends Instruction {
  static type: string = 'SSTORE';
  static numberOfOperands = 2;

  constructor(private slotOffset: number, private dataOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, stateManager: AvmStateManager): void {
    const slot = machineState.memory.get(this.slotOffset);
    const data = machineState.memory.get(this.dataOffset);

    stateManager.store(
      machineState.executionEnvironment.storageAddress,
      new Fr(slot.toBigInt()),
      new Fr(data.toBigInt()),
    );

    this.incrementPc(machineState);
  }
}

/** - */
export class SLoad extends Instruction {
  static type: string = 'SLOAD';
  static numberOfOperands = 2;

  constructor(private slotOffset: number, private destOffset: number) {
    super();
  }

  async execute(machineState: AvmMachineState, stateManager: AvmStateManager): Promise<void> {
    const slot = machineState.memory.get(this.slotOffset);

    const data = stateManager.read(machineState.executionEnvironment.storageAddress, new Fr(slot.toBigInt()));

    machineState.memory.set(this.destOffset, await data);

    this.incrementPc(machineState);
  }
}
