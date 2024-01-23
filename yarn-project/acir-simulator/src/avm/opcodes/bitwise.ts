import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

/** - */
export class And extends Instruction {
  static type: string = 'AND';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() & b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** - */
export class Or extends Instruction {
  static type: string = 'OR';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() | b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** - */
export class Xor extends Instruction {
  static type: string = 'XOR';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() ^ b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** - */
export class Not extends Instruction {
  static type: string = 'NOT';
  static numberOfOperands = 2;

  constructor(private aOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);

    // TODO: hack -> Bitwise operations should not occur over field elements
    // It should only work over integers
    const result = ~a.toBigInt();

    const dest = new Fr(result < 0 ? Fr.MODULUS + /* using a + as result is -ve*/ result : result);
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** -*/
export class Shl extends Instruction {
  static type: string = 'SHL';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    const dest = new Fr(a.toBigInt() << b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}

/** -*/
export class Shr extends Instruction {
  static type: string = 'SHR';
  static numberOfOperands = 3;

  constructor(private aOffset: number, private bOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a: Fr = machineState.readMemory(this.aOffset);
    const b: Fr = machineState.readMemory(this.bOffset);

    // Here we are assuming that the field element maps to a positive number.
    // The >> operator is *signed* in JS (and it sign extends).
    // E.g.: -1n >> 3n == -1n.
    const dest = new Fr(a.toBigInt() >> b.toBigInt());
    machineState.writeMemory(this.destOffset, dest);

    this.incrementPc(machineState);
  }
}
