import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

/** -*/
export class Add extends Instruction {
  static type: string = 'ADD';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.readMemory(this.aOffset);
    const b = machineState.readMemory(this.bOffset);

    const dest = a.add(b);
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** -*/
export class Sub extends Instruction {
  static type: string = 'SUB';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.readMemory(this.aOffset);
    const b = machineState.readMemory(this.bOffset);

    const dest = a.sub(b);
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** -*/
export class Mul extends Instruction {
  static type: string = 'MUL';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = a.mul(b);
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** -*/
export class Div extends Instruction {
  static type: string = 'DIV';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = a.div(b);
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}
