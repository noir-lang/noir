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

    const dest = new Fr((a.toBigInt() + b.toBigInt()) % Fr.MODULUS);
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

    const dest = new Fr((a.toBigInt() - b.toBigInt()) % Fr.MODULUS);
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

    const dest = new Fr((a.toBigInt() * b.toBigInt()) % Fr.MODULUS);
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

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3993): proper field division
    const dest = new Fr(a.toBigInt() / b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}
