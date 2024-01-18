import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

/** -*/
export class Eq extends Instruction {
  static type: string = 'EQ';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() == b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}
/** -*/
export class Lt extends Instruction {
  static type: string = 'Lt';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() < b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** -*/
export class Lte extends Instruction {
  static type: string = 'LTE';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() < b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}
