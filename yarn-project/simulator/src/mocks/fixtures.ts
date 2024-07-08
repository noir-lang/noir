import { type FunctionCall, type SimulationError, UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  ARGS_LENGTH,
  AvmExecutionHints,
  type AztecAddress,
  CallContext,
  type ContractStorageRead,
  type ContractStorageUpdateRequest,
  Fr,
  Gas,
  type PublicCallRequest,
} from '@aztec/circuits.js';
import { makeAztecAddress, makeSelector } from '@aztec/circuits.js/testing';
import { FunctionType } from '@aztec/foundation/abi';
import { padArrayEnd } from '@aztec/foundation/collection';

import { type PublicExecutionRequest, type PublicExecutionResult } from '../public/execution.js';

export class PublicExecutionResultBuilder {
  private _executionRequest: PublicExecutionRequest;
  private _nestedExecutions: PublicExecutionResult[] = [];
  private _contractStorageUpdateRequests: ContractStorageUpdateRequest[] = [];
  private _contractStorageReads: ContractStorageRead[] = [];
  private _returnValues: Fr[] = [];
  private _reverted = false;
  private _revertReason: SimulationError | undefined = undefined;

  constructor(executionRequest: PublicExecutionRequest) {
    this._executionRequest = executionRequest;
  }

  static fromPublicCallRequest({
    request,
    returnValues = [new Fr(1n)],
    nestedExecutions = [],
    contractStorageUpdateRequests = [],
    contractStorageReads = [],
    revertReason = undefined,
  }: {
    request: PublicCallRequest;
    returnValues?: Fr[];
    nestedExecutions?: PublicExecutionResult[];
    contractStorageUpdateRequests?: ContractStorageUpdateRequest[];
    contractStorageReads?: ContractStorageRead[];
    revertReason?: SimulationError;
  }): PublicExecutionResultBuilder {
    const builder = new PublicExecutionResultBuilder(request);

    builder.withNestedExecutions(...nestedExecutions);
    builder.withContractStorageUpdateRequest(...contractStorageUpdateRequests);
    builder.withContractStorageRead(...contractStorageReads);
    builder.withReturnValues(...returnValues);
    if (revertReason) {
      builder.withReverted(revertReason);
    }

    return builder;
  }

  static fromFunctionCall({
    from,
    tx,
    returnValues = [new Fr(1n)],
    nestedExecutions = [],
    contractStorageUpdateRequests = [],
    contractStorageReads = [],
    revertReason,
  }: {
    from: AztecAddress;
    tx: FunctionCall;
    returnValues?: Fr[];
    nestedExecutions?: PublicExecutionResult[];
    contractStorageUpdateRequests?: ContractStorageUpdateRequest[];
    contractStorageReads?: ContractStorageRead[];
    revertReason?: SimulationError;
  }) {
    const builder = new PublicExecutionResultBuilder({
      callContext: new CallContext(from, tx.to, tx.selector, false, false),
      contractAddress: tx.to,
      functionSelector: tx.selector,
      args: tx.args,
    });

    builder.withNestedExecutions(...nestedExecutions);
    builder.withContractStorageUpdateRequest(...contractStorageUpdateRequests);
    builder.withContractStorageRead(...contractStorageReads);
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

  withContractStorageRead(...reads: ContractStorageRead[]): PublicExecutionResultBuilder {
    this._contractStorageReads.push(...reads);
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

  build(overrides: Partial<PublicExecutionResult> = {}): PublicExecutionResult {
    return {
      executionRequest: this._executionRequest,
      nestedExecutions: this._nestedExecutions,
      noteHashReadRequests: [],
      nullifierReadRequests: [],
      nullifierNonExistentReadRequests: [],
      l1ToL2MsgReadRequests: [],
      contractStorageUpdateRequests: this._contractStorageUpdateRequests,
      returnValues: padArrayEnd(this._returnValues, Fr.ZERO, 4), // TODO(#5450) Need to use the proper return values here
      noteHashes: [],
      nullifiers: [],
      l2ToL1Messages: [],
      contractStorageReads: [],
      unencryptedLogsHashes: [],
      unencryptedLogs: UnencryptedFunctionL2Logs.empty(),
      allUnencryptedLogs: UnencryptedFunctionL2Logs.empty(),
      startSideEffectCounter: Fr.ZERO,
      endSideEffectCounter: Fr.ZERO,
      reverted: this._reverted,
      revertReason: this._revertReason,
      startGasLeft: Gas.test(),
      endGasLeft: Gas.test(),
      transactionFee: Fr.ZERO,
      calldata: [],
      avmCircuitHints: AvmExecutionHints.empty(),
      functionName: 'unknown',
      ...overrides,
    };
  }
}

export const makeFunctionCall = (
  name = 'function',
  to = makeAztecAddress(30),
  selector = makeSelector(5),
  type = FunctionType.PUBLIC,
  args = new Array(ARGS_LENGTH).fill(Fr.ZERO),
  isStatic = false,
  returnTypes = [],
) => ({ name, to, selector, type, args, isStatic, returnTypes });
