import { AvmMachineState } from '../avm_machine_state.js';
import { Field, TaggedMemory, TypeTag } from '../avm_memory_types.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Instruction } from './instruction.js';

export class Set extends Instruction {
  static type: string = 'SET';
  static numberOfOperands = 2;

  constructor(private value: bigint, private dstOffset: number, private dstTag: TypeTag) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const res = TaggedMemory.integralFromTag(this.value, this.dstTag);

    machineState.memory.set(this.dstOffset, res);

    this.incrementPc(machineState);
  }
}

export class Cast extends Instruction {
  static type: string = 'CAST';
  static numberOfOperands = 2;

  constructor(private aOffset: number, private dstOffset: number, private dstTag: TypeTag) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.memory.get(this.aOffset);

    // TODO: consider not using toBigInt()
    const casted =
      this.dstTag == TypeTag.FIELD ? new Field(a.toBigInt()) : TaggedMemory.integralFromTag(a.toBigInt(), this.dstTag);

    machineState.memory.set(this.dstOffset, casted);

    this.incrementPc(machineState);
  }
}

export class Mov extends Instruction {
  static type: string = 'MOV';
  static numberOfOperands = 2;

  constructor(private aOffset: number, private dstOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.memory.get(this.aOffset);

    machineState.memory.set(this.dstOffset, a);

    this.incrementPc(machineState);
  }
}

export class CMov extends Instruction {
  static type: string = 'CMOV';
  static numberOfOperands = 4;

  constructor(private aOffset: number, private bOffset: number, private condOffset: number, private dstOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const a = machineState.memory.get(this.aOffset);
    const b = machineState.memory.get(this.bOffset);
    const cond = machineState.memory.get(this.condOffset);

    // TODO: reconsider toBigInt() here
    machineState.memory.set(this.dstOffset, cond.toBigInt() > 0 ? a : b);

    this.incrementPc(machineState);
  }
}

export class CalldataCopy extends Instruction {
  static type: string = 'CALLDATACOPY';
  static numberOfOperands = 3;

  constructor(private cdOffset: number, private copySize: number, private dstOffset: number) {
    super();
  }

  execute(machineState: AvmMachineState, _stateManager: AvmStateManager): void {
    const transformedData = machineState.executionEnvironment.calldata
      .slice(this.cdOffset, this.cdOffset + this.copySize)
      .map(f => new Field(f));
    machineState.memory.setSlice(this.dstOffset, transformedData);

    this.incrementPc(machineState);
  }
}
