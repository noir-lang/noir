import { type FunctionCall, type SimulationError, UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  ARGS_LENGTH,
  type AztecAddress,
  CallContext,
  CallRequest,
  type ContractStorageUpdateRequest,
  Fr,
  FunctionData,
  Gas,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  type PrivateKernelTailCircuitPublicInputs,
  type PublicCallRequest,
} from '@aztec/circuits.js';
import { makeAztecAddress, makeSelector } from '@aztec/circuits.js/testing';
import { padArrayEnd } from '@aztec/foundation/collection';

import { type PublicExecution, type PublicExecutionResult } from '../public/execution.js';

export class PublicExecutionResultBuilder {
  private _execution: PublicExecution;
  private _nestedExecutions: PublicExecutionResult[] = [];
  private _contractStorageUpdateRequests: ContractStorageUpdateRequest[] = [];
  private _returnValues: Fr[] = [];
  private _reverted = false;
  private _revertReason: SimulationError | undefined = undefined;

  constructor(execution: PublicExecution) {
    this._execution = execution;
  }

  static fromPublicCallRequest({
    request,
    returnValues = [new Fr(1n)],
    nestedExecutions = [],
    contractStorageUpdateRequests = [],
  }: {
    request: PublicCallRequest;
    returnValues?: Fr[];
    nestedExecutions?: PublicExecutionResult[];
    contractStorageUpdateRequests?: ContractStorageUpdateRequest[];
  }): PublicExecutionResultBuilder {
    const builder = new PublicExecutionResultBuilder(request);

    builder.withNestedExecutions(...nestedExecutions);
    builder.withContractStorageUpdateRequest(...contractStorageUpdateRequests);
    builder.withReturnValues(...returnValues);

    return builder;
  }

  static fromFunctionCall({
    from,
    tx,
    returnValues = [new Fr(1n)],
    nestedExecutions = [],
    contractStorageUpdateRequests = [],
    revertReason,
  }: {
    from: AztecAddress;
    tx: FunctionCall;
    returnValues?: Fr[];
    nestedExecutions?: PublicExecutionResult[];
    contractStorageUpdateRequests?: ContractStorageUpdateRequest[];
    revertReason?: SimulationError;
  }) {
    const builder = new PublicExecutionResultBuilder({
      callContext: new CallContext(from, tx.to, tx.functionData.selector, false, false, 0),
      contractAddress: tx.to,
      functionData: tx.functionData,
      args: tx.args,
    });

    builder.withNestedExecutions(...nestedExecutions);
    builder.withContractStorageUpdateRequest(...contractStorageUpdateRequests);
    builder.withReturnValues(...returnValues);
    if (revertReason) {
      builder.withReverted(revertReason);
    }

    return builder;
  }

  withNestedExecutions(...nested: PublicExecutionResult[]): PublicExecutionResultBuilder {
    this._nestedExecutions.push(...nested);
    return this;
  }

  withContractStorageUpdateRequest(...request: ContractStorageUpdateRequest[]): PublicExecutionResultBuilder {
    this._contractStorageUpdateRequests.push(...request);
    return this;
  }

  withReturnValues(...values: Fr[]): PublicExecutionResultBuilder {
    this._returnValues.push(...values);
    return this;
  }

  withReverted(reason: SimulationError): PublicExecutionResultBuilder {
    this._reverted = true;
    this._revertReason = reason;
    return this;
  }

  build(): PublicExecutionResult {
    return {
      execution: this._execution,
      nestedExecutions: this._nestedExecutions,
      nullifierReadRequests: [],
      nullifierNonExistentReadRequests: [],
      contractStorageUpdateRequests: this._contractStorageUpdateRequests,
      returnValues: padArrayEnd(this._returnValues, Fr.ZERO, 4), // TODO(#5450) Need to use the proper return values here
      newNoteHashes: [],
      newNullifiers: [],
      newL2ToL1Messages: [],
      contractStorageReads: [],
      unencryptedLogsHashes: [],
      unencryptedLogs: UnencryptedFunctionL2Logs.empty(),
      startSideEffectCounter: Fr.ZERO,
      endSideEffectCounter: Fr.ZERO,
      reverted: this._reverted,
      revertReason: this._revertReason,
      gasLeft: Gas.test(), // TODO(palla/gas): Set a proper value
    };
  }
}

export const makeFunctionCall = (
  to = makeAztecAddress(30),
  selector = makeSelector(5),
  args = new Array(ARGS_LENGTH).fill(Fr.ZERO),
) => ({ to, functionData: new FunctionData(selector, false), args });

export function addKernelPublicCallStack(
  kernelOutput: PrivateKernelTailCircuitPublicInputs,
  calls: {
    setupCalls: PublicCallRequest[];
    appLogicCalls: PublicCallRequest[];
    teardownCall: PublicCallRequest;
  },
) {
  // the first two calls are non-revertible
  // the first is for setup, the second is for teardown
  kernelOutput.forPublic!.endNonRevertibleData.publicCallStack = padArrayEnd(
    // this is a stack, so the first item is the last call
    // and callRequests is in the order of the calls
    [calls.teardownCall.toCallRequest(), ...calls.setupCalls.map(c => c.toCallRequest())],
    CallRequest.empty(),
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  );

  kernelOutput.forPublic!.end.publicCallStack = padArrayEnd(
    calls.appLogicCalls.map(c => c.toCallRequest()),
    CallRequest.empty(),
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  );
}
