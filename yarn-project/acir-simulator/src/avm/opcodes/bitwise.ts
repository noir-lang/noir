import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

/** - */
export class And implements Instruction {
  static type: string = 'AND';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {}

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() & b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);
  }
}

/** - */
export class Or implements Instruction {
  static type: string = 'OR';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {}

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() | b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);
  }
}

/** - */
export class Xor implements Instruction {
  static type: string = 'XOR';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {}

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() ^ b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);
  }
}

/** - */
export class Not implements Instruction {
  static type: string = 'NOT';
  static numberOfOperands = 2;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {}

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);

    const dest = new Fr(~a.toBigInt());
    machineState.writeMemory(this.destOffset, dest);
  }
}

/** -*/
export class Shl implements Instruction {
  static type: string = 'SHL';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {}

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() << b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);
  }
}

/** -*/
export class Shr implements Instruction {
  static type: string = 'SHR';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {}

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() >> b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);
  }
}
