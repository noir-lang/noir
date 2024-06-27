import { FunctionSelector, Gas } from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';

import type { AvmContext } from '../avm_context.js';
import { type AvmContractCallResult } from '../avm_contract_call_result.js';
import { gasLeftToGas } from '../avm_gas.js';
import { Field, TypeTag, Uint8 } from '../avm_memory_types.js';
import { AvmSimulator } from '../avm_simulator.js';
import { RethrownError } from '../errors.js';
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
    OperandType.UINT32,
  ];

  constructor(
    private indirect: number,
    private gasOffset: number /* Unused due to no formal gas implementation at this moment */,
    private addrOffset: number,
    private argsOffset: number,
    private argsSizeOffset: number,
    private retOffset: number,
    private retSize: number,
    private successOffset: number,
    // NOTE: Function selector is likely temporary since eventually public contract bytecode will be one
    // blob containing all functions, and function selector will become an application-level mechanism
    // (e.g. first few bytes of calldata + compiler-generated jump table)
    private functionSelectorOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext) {
    const memory = context.machineState.memory.track(this.type);
    const [gasOffset, addrOffset, argsOffset, argsSizeOffset, retOffset, successOffset] = Addressing.fromWire(
      this.indirect,
    ).resolve(
      [this.gasOffset, this.addrOffset, this.argsOffset, this.argsSizeOffset, this.retOffset, this.successOffset],
      memory,
    );
    memory.checkTags(TypeTag.FIELD, gasOffset, gasOffset + 1);
    memory.checkTag(TypeTag.FIELD, addrOffset);
    memory.checkTag(TypeTag.UINT32, argsSizeOffset);
    memory.checkTag(TypeTag.FIELD, this.functionSelectorOffset);

    const calldataSize = memory.get(argsSizeOffset).toNumber();
    memory.checkTagsRange(TypeTag.FIELD, argsOffset, calldataSize);

    const callAddress = memory.getAs<Field>(addrOffset);
    const calldata = memory.getSlice(argsOffset, calldataSize).map(f => f.toFr());
    const functionSelector = memory.getAs<Field>(this.functionSelectorOffset).toFr();
    // If we are already in a static call, we propagate the environment.
    const callType = context.environment.isStaticCall ? 'STATICCALL' : this.type;

    // First we consume the gas for this operation.
    const memoryOperations = { reads: calldataSize + 5, writes: 1 + this.retSize, indirect: this.indirect };
    context.machineState.consumeGas(this.gasCost(memoryOperations));
    // Then we consume the gas allocated for the nested call. The excess will be refunded later.
    // Gas allocation is capped by the amount of gas left in the current context.
    // We have to do some dancing here because the gas allocation is a field,
    // but in the machine state we track gas as a number.
    const allocatedL2Gas = Number(BigIntMin(memory.get(gasOffset).toBigInt(), BigInt(context.machineState.l2GasLeft)));
    const allocatedDaGas = Number(
      BigIntMin(memory.get(gasOffset + 1).toBigInt(), BigInt(context.machineState.daGasLeft)),
    );
    const allocatedGas = { l2Gas: allocatedL2Gas, daGas: allocatedDaGas };
    context.machineState.consumeGas(allocatedGas);

    const nestedContext = context.createNestedContractCallContext(
      callAddress.toFr(),
      calldata,
      allocatedGas,
      callType,
      FunctionSelector.fromField(functionSelector),
    );

    const simulator = new AvmSimulator(nestedContext);
    const nestedCallResults: AvmContractCallResult = await simulator.execute();
    const success = !nestedCallResults.reverted;

    // TRANSITIONAL: We rethrow here so that the MESSAGE gets propagated.
    //               This means that for now, the caller cannot recover from errors.
    if (!success) {
      if (!nestedCallResults.revertReason) {
        throw new Error('A reverted nested call should be assigned a revert reason in the AVM execution loop');
      }
      // The nested call's revertReason will be used to track the stack of error causes down to the root.
      throw new RethrownError(nestedCallResults.revertReason.message, nestedCallResults.revertReason);
    }

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

    // Accept the nested call's state and trace the nested call
    await context.persistableState.processNestedCall(
      /*nestedState=*/ nestedContext.persistableState,
      /*nestedEnvironment=*/ nestedContext.environment,
      /*startGasLeft=*/ Gas.from(allocatedGas),
      /*endGasLeft=*/ Gas.from(nestedContext.machineState.gasLeft),
      /*bytecode=*/ simulator.getBytecode()!,
      /*avmCallResults=*/ nestedCallResults,
    );

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }

  public abstract override get type(): 'CALL' | 'STATICCALL';
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

/** Returns the smaller of two bigints. */
function BigIntMin(a: bigint, b: bigint): bigint {
  return a < b ? a : b;
}
