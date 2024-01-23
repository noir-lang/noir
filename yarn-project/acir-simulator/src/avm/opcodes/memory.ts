import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

/** - */
export class Set extends Instruction {
  static type: string = 'SET';
  static numberOfOperands = 2;

  constructor(private value: bigint, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    machineState.writeMemory(this.destOffset, new Fr(this.value));

    this.incrementPc(machineState);
  }
}

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/3987): tags are not implemented yet - this will behave as a mov
/** - */
export class Cast extends Instruction {
  static type: string = 'CAST';
  static numberOfOperands = 2;

  constructor(private aOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.readMemory(this.aOffset);

    machineState.writeMemory(this.destOffset, a);

    this.incrementPc(machineState);
  }
}

/** - */
export class Mov extends Instruction {
  static type: string = 'MOV';
  static numberOfOperands = 2;

  constructor(private aOffset: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.readMemory(this.aOffset);

    machineState.writeMemory(this.destOffset, a);

    this.incrementPc(machineState);
  }
}

/** - */
export class CMov extends Instruction {
  static type: string = 'MOV';
  static numberOfOperands = 4;

  constructor(
    private aOffset: number,
    private bOffset: number,
    private condOffset: number,
    private destOffset: number,
  ) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.readMemory(this.aOffset);
    const b = machineState.readMemory(this.bOffset);
    const cond = machineState.readMemory(this.condOffset);

    machineState.writeMemory(this.destOffset, cond.toBigInt() ? a : b);

    this.incrementPc(machineState);
  }
}

/** - */
export class CalldataCopy extends Instruction {
  static type: string = 'CALLDATACOPY';
  static numberOfOperands = 3;

  constructor(private cdOffset: number, private copySize: number, private destOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const calldata = machineState.calldata.slice(this.cdOffset, this.cdOffset + this.copySize);
    machineState.writeMemoryChunk(this.destOffset, calldata);

    this.incrementPc(machineState);
  }
}
