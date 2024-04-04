import { FunctionSelector } from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';

import type { AvmContext } from '../avm_context.js';
import { gasLeftToGas, sumGas } from '../avm_gas.js';
import { Field, Uint8 } from '../avm_memory_types.js';
import { AvmSimulator } from '../avm_simulator.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

abstract class ExternalCall extends Instruction {
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
    private gasOffset: number /* Unused due to no formal gas implementation at this moment */,
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

  public async execute(context: AvmContext) {
    const memory = context.machineState.memory.track(this.type);
    const [gasOffset, addrOffset, argsOffset, retOffset, successOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.gasOffset, this.addrOffset, this.argsOffset, this.retOffset, this.successOffset],
      memory,
    );

    const callAddress = memory.getAs<Field>(addrOffset);
    const calldata = memory.getSlice(argsOffset, this.argsSize).map(f => f.toFr());
    const l1Gas = memory.get(gasOffset).toNumber();
    const l2Gas = memory.getAs<Field>(gasOffset + 1).toNumber();
    const daGas = memory.getAs<Field>(gasOffset + 2).toNumber();
    const functionSelector = memory.getAs<Field>(this.temporaryFunctionSelectorOffset).toFr();

    const allocatedGas = { l1Gas, l2Gas, daGas };
    const memoryOperations = { reads: this.argsSize + 5, writes: 1 + this.retSize, indirect: this.indirect };
    const totalGas = sumGas(this.gasCost(memoryOperations), allocatedGas);
    context.machineState.consumeGas(totalGas);

    const nestedContext = context.createNestedContractCallContext(
      callAddress.toFr(),
      calldata,
      allocatedGas,
      this.type,
      FunctionSelector.fromField(functionSelector),
    );

    const nestedCallResults = await new AvmSimulator(nestedContext).execute();
    const success = !nestedCallResults.reverted;

    // We only take as much data as was specified in the return size and pad with zeroes if the return data is smaller
    // than the specified size in order to prevent that memory to be left with garbage
    const returnData = nestedCallResults.output.slice(0, this.retSize);
    const convertedReturnData = padArrayEnd(
      returnData.map(f => new Field(f)),
      new Field(0),
      this.retSize,
    );

    // Write our return data into memory
    memory.set(successOffset, new Uint8(success ? 1 : 0));
    memory.setSlice(retOffset, convertedReturnData);

    // Refund unused gas
    context.machineState.refundGas(gasLeftToGas(nestedContext.machineState));

    // TODO: Should we merge the changes from a nested call in the case of a STATIC call?
    if (success) {
      context.persistableState.acceptNestedCallState(nestedContext.persistableState);
    } else {
      context.persistableState.rejectNestedCallState(nestedContext.persistableState);
    }

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }

  public abstract get type(): 'CALL' | 'STATICCALL';
}

export class Call extends ExternalCall {
  static type = 'CALL' as const;
  static readonly opcode: Opcode = Opcode.CALL;

  public get type() {
    return Call.type;
  }
}

export class StaticCall extends ExternalCall {
  static type = 'STATICCALL' as const;
  static readonly opcode: Opcode = Opcode.STATICCALL;

  public get type() {
    return StaticCall.type;
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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: this.copySize, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [returnOffset] = Addressing.fromWire(this.indirect).resolve([this.returnOffset], memory);

    const output = memory.getSlice(returnOffset, this.copySize).map(word => word.toFr());

    context.machineState.return(output);
    memory.assert(memoryOperations);
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

  public async execute(context: AvmContext): Promise<void> {
    const memoryOperations = { reads: this.retSize, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [returnOffset] = Addressing.fromWire(this.indirect).resolve([this.returnOffset], memory);

    const output = memory.getSlice(returnOffset, this.retSize).map(word => word.toFr());

    context.machineState.revert(output);
    memory.assert(memoryOperations);
  }
}
