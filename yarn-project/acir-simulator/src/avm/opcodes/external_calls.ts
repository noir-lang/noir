import { Fr } from '@aztec/foundation/fields';

import { AvmContext } from '../avm_context.js';
import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/journal.js';
import { Instruction } from './instruction.js';

export class Call extends Instruction {
  static type: string = 'CALL';
  static numberOfOperands = 7;

  constructor(
    private /* Unused due to no formal gas implementation at this moment */ _gasOffset: number,
    private addrOffset: number,
    private argsOffset: number,
    private argSize: number,
    private retOffset: number,
    private retSize: number,
    private successOffset: number,
  ) {
    super();
  }

  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3992): there is no concept of remaining / available gas at this moment
  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    const callAddress = machineState.memory.getAs<Field>(this.addrOffset);
    const calldata = machineState.memory.getSlice(this.argsOffset, this.argSize).map(f => new Fr(f.toBigInt()));

    const avmContext = AvmContext.prepExternalCallContext(
      new Fr(callAddress.toBigInt()),
      calldata,
      machineState.executionEnvironment,
      journal,
    );

    const returnObject = await avmContext.call();
    const success = !returnObject.reverted;

    // We only take as much data as was specified in the return size -> TODO: should we be reverting here
    const returnData = returnObject.output.slice(0, this.retSize);
    const convertedReturnData = returnData.map(f => new Field(f));

    // Write our return data into memory
    machineState.memory.set(this.successOffset, new Field(success ? 1 : 0));
    machineState.memory.setSlice(this.retOffset, convertedReturnData);

    if (success) {
      avmContext.mergeJournal();
    }

    this.incrementPc(machineState);
  }
}

export class StaticCall extends Instruction {
  static type: string = 'STATICCALL';
  static numberOfOperands = 7;

  constructor(
    private /* Unused due to no formal gas implementation at this moment */ _gasOffset: number,
    private addrOffset: number,
    private argsOffset: number,
    private argSize: number,
    private retOffset: number,
    private retSize: number,
    private successOffset: number,
  ) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    const callAddress = machineState.memory.get(this.addrOffset);
    const calldata = machineState.memory.getSlice(this.argsOffset, this.argSize).map(f => new Fr(f.toBigInt()));

    const avmContext = AvmContext.prepExternalStaticCallContext(
      new Fr(callAddress.toBigInt()),
      calldata,
      machineState.executionEnvironment,
      journal,
    );

    const returnObject = await avmContext.call();
    const success = !returnObject.reverted;

    // We only take as much data as was specified in the return size -> TODO: should we be reverting here
    const returnData = returnObject.output.slice(0, this.retSize);
    const convertedReturnData = returnData.map(f => new Field(f));

    // Write our return data into memory
    machineState.memory.set(this.successOffset, new Field(success ? 1 : 0));
    machineState.memory.setSlice(this.retOffset, convertedReturnData);

    if (success) {
      avmContext.mergeJournal();
    }

    this.incrementPc(machineState);
  }
}
