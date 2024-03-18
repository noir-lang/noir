import { FunctionSelector } from '@aztec/circuits.js';

import type { AvmContext } from '../avm_context.js';
import { Field, Uint8 } from '../avm_memory_types.js';
import { AvmSimulator } from '../avm_simulator.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
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
    /* temporary function selector */
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
    // Function selector is temporary since eventually public contract bytecode will be one blob
    // containing all functions, and function selector will become an application-level mechanism
    // (e.g. first few bytes of calldata + compiler-generated jump table)
    private temporaryFunctionSelectorOffset: number,
  ) {
    super();
  }

  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3992): there is no concept of remaining / available gas at this moment
  async execute(context: AvmContext): Promise<void> {
    const [_gasOffset, addrOffset, argsOffset, retOffset, successOffset] = Addressing.fromWire(this.indirect).resolve(
      [this._gasOffset, this.addrOffset, this.argsOffset, this.retOffset, this.successOffset],
      context.machineState.memory,
    );

    const callAddress = context.machineState.memory.getAs<Field>(addrOffset);
    const calldata = context.machineState.memory.getSlice(argsOffset, this.argsSize).map(f => f.toFr());
    const functionSelector = context.machineState.memory.getAs<Field>(this.temporaryFunctionSelectorOffset).toFr();

    const nestedContext = context.createNestedContractCallContext(
      callAddress.toFr(),
      calldata,
      FunctionSelector.fromField(functionSelector),
    );

    const nestedCallResults = await new AvmSimulator(nestedContext).execute();
    const success = !nestedCallResults.reverted;

    // We only take as much data as was specified in the return size -> TODO: should we be reverting here
    const returnData = nestedCallResults.output.slice(0, this.retSize);
    const convertedReturnData = returnData.map(f => new Field(f));

    // Write our return data into memory
    context.machineState.memory.set(successOffset, new Uint8(success ? 1 : 0));
    context.machineState.memory.setSlice(retOffset, convertedReturnData);

    if (success) {
      context.persistableState.acceptNestedCallState(nestedContext.persistableState);
    } else {
      context.persistableState.rejectNestedCallState(nestedContext.persistableState);
    }

    context.machineState.incrementPc();
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
    /* temporary function selector */
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
    private temporaryFunctionSelectorOffset: number,
  ) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const [_gasOffset, addrOffset, argsOffset, retOffset, successOffset] = Addressing.fromWire(this.indirect).resolve(
      [this._gasOffset, this.addrOffset, this.argsOffset, this.retOffset, this.successOffset],
      context.machineState.memory,
    );

    const callAddress = context.machineState.memory.get(addrOffset);
    const calldata = context.machineState.memory.getSlice(argsOffset, this.argsSize).map(f => f.toFr());
    const functionSelector = context.machineState.memory.getAs<Field>(this.temporaryFunctionSelectorOffset).toFr();

    const nestedContext = context.createNestedContractStaticCallContext(
      callAddress.toFr(),
      calldata,
      FunctionSelector.fromField(functionSelector),
    );

    const nestedCallResults = await new AvmSimulator(nestedContext).execute();
    const success = !nestedCallResults.reverted;

    // We only take as much data as was specified in the return size -> TODO: should we be reverting here
    const returnData = nestedCallResults.output.slice(0, this.retSize);
    const convertedReturnData = returnData.map(f => new Field(f));

    // Write our return data into memory
    context.machineState.memory.set(successOffset, new Uint8(success ? 1 : 0));
    context.machineState.memory.setSlice(retOffset, convertedReturnData);

    if (success) {
      context.persistableState.acceptNestedCallState(nestedContext.persistableState);
    } else {
      context.persistableState.rejectNestedCallState(nestedContext.persistableState);
    }

    context.machineState.incrementPc();
  }
}

export class Return extends Instruction {
  static type: string = 'RETURN';
  static readonly opcode: Opcode = Opcode.RETURN;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private returnOffset: number, private copySize: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const [returnOffset] = Addressing.fromWire(this.indirect).resolve([this.returnOffset], context.machineState.memory);

    const output = context.machineState.memory.getSlice(returnOffset, this.copySize).map(word => word.toFr());

    context.machineState.return(output);
  }
}

export class Revert extends Instruction {
  static type: string = 'REVERT';
  static readonly opcode: Opcode = Opcode.REVERT;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private returnOffset: number, private retSize: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const [returnOffset] = Addressing.fromWire(this.indirect).resolve([this.returnOffset], context.machineState.memory);

    const output = context.machineState.memory.getSlice(returnOffset, this.retSize).map(word => word.toFr());

    context.machineState.revert(output);
  }
}
