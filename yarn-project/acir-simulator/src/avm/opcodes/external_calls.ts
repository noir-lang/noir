import { Fr } from '@aztec/foundation/fields';

import { AvmContext } from '../avm_context.js';
import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { AvmJournal } from '../journal/journal.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Instruction } from './instruction.js';

export class Call extends Instruction {
  static type: string = 'CALL';
  static readonly opcode: Opcode = Opcode.CALL;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private _gasOffset: number /* Unused due to no formal gas implementation at this moment */,
    private addrOffset: number,
    private argsOffset: number,
    private argsSize: number,
    private retOffset: number,
    private retSize: number,
    private successOffset: number,
  ) {
    super();
  }

  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3992): there is no concept of remaining / available gas at this moment
  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    const callAddress = machineState.memory.getAs<Field>(this.addrOffset);
    const calldata = machineState.memory.getSlice(this.argsOffset, this.argsSize).map(f => new Fr(f.toBigInt()));

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
      avmContext.mergeJournalSuccess();
    } else {
      avmContext.mergeJournalFailure();
    }
    this.incrementPc(machineState);
  }
}

export class StaticCall extends Instruction {
  static type: string = 'STATICCALL';
  static readonly opcode: Opcode = Opcode.STATICCALL;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private _gasOffset: number /* Unused due to no formal gas implementation at this moment */,
    private addrOffset: number,
    private argsOffset: number,
    private argsSize: number,
    private retOffset: number,
    private retSize: number,
    private successOffset: number,
  ) {
    super();
  }

  async execute(machineState: AvmMachineState, journal: AvmJournal): Promise<void> {
    const callAddress = machineState.memory.get(this.addrOffset);
    const calldata = machineState.memory.getSlice(this.argsOffset, this.argsSize).map(f => new Fr(f.toBigInt()));

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
      avmContext.mergeJournalSuccess();
    } else {
      avmContext.mergeJournalFailure();
    }

    this.incrementPc(machineState);
  }
}
