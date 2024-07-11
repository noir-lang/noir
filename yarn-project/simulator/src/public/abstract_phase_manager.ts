import {
  AVM_REQUEST,
  type AvmProvingRequest,
  MerkleTreeId,
  type NestedProcessReturnValues,
  type PublicKernelNonTailRequest,
  PublicKernelType,
  type PublicProvingRequest,
  type SimulationError,
  type Tx,
  type UnencryptedFunctionL2Logs,
} from '@aztec/circuit-types';
import {
  type AvmExecutionHints,
  AztecAddress,
  CallRequest,
  ClientIvcProof,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  FunctionData,
  Gas,
  type GlobalVariables,
  type Header,
  type KernelCircuitPublicInputs,
  L2ToL1Message,
  LogHash,
  MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL,
  MAX_L2_TO_L1_MSGS_PER_CALL,
  MAX_NOTE_HASHES_PER_CALL,
  MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
  MAX_NULLIFIERS_PER_CALL,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
  MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_UNENCRYPTED_LOGS_PER_CALL,
  NESTED_RECURSIVE_PROOF_LENGTH,
  NoteHash,
  Nullifier,
  PublicCallData,
  type PublicCallRequest,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  PublicKernelData,
  ReadRequest,
  RevertCode,
  makeEmptyProof,
  makeEmptyRecursiveProof,
} from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { padArrayEnd } from '@aztec/foundation/collection';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import {
  type ProtocolArtifact,
  ProtocolCircuitVkIndexes,
  ProtocolCircuitVks,
  getVKIndex,
  getVKSiblingPath,
} from '@aztec/noir-protocol-circuits-types';
import {
  type PublicExecutionRequest,
  type PublicExecutionResult,
  type PublicExecutor,
  accumulateReturnValues,
  isPublicExecutionResult,
} from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { HintsBuilder } from './hints_builder.js';
import { type PublicKernelCircuitSimulator } from './public_kernel_circuit_simulator.js';

export const PhaseIsRevertible: Record<PublicKernelType, boolean> = {
  [PublicKernelType.NON_PUBLIC]: false,
  [PublicKernelType.SETUP]: false,
  [PublicKernelType.APP_LOGIC]: true,
  [PublicKernelType.TEARDOWN]: true,
  [PublicKernelType.TAIL]: false,
};

export type PublicProvingInformation = {
  functionName: string; // informational only
  calldata: Fr[];
  bytecode: Buffer;
  inputs: PublicKernelCircuitPrivateInputs;
  avmHints: AvmExecutionHints;
};

export function makeAvmProvingRequest(
  info: PublicProvingInformation,
  kernelType: PublicKernelNonTailRequest['type'],
): AvmProvingRequest {
  return {
    type: AVM_REQUEST,
    functionName: info.functionName,
    bytecode: info.bytecode,
    calldata: info.calldata,
    avmHints: info.avmHints,
    kernelRequest: {
      type: kernelType,
      inputs: info.inputs,
    },
  };
}

export type TxPublicCallsResult = {
  /** Inputs to be used for proving */
  publicProvingInformation: PublicProvingInformation[];
  /** The public kernel output at the end of the Tx */
  kernelOutput: PublicKernelCircuitPublicInputs;
  /** The last circuit ran by this phase */
  lastKernelArtifact: ProtocolArtifact;
  /** Unencrypted logs generated during the execution of this Tx */
  newUnencryptedLogs: UnencryptedFunctionL2Logs[];
  /** Revert reason, if any */
  revertReason?: SimulationError;
  /** Return values of simulating complete callstack */
  returnValues: NestedProcessReturnValues[];
  /** Gas used during the execution this Tx */
  gasUsed?: Gas;
};

export type PhaseResult = {
  /** The collection of public proving requests */
  publicProvingRequests: PublicProvingRequest[];
  /** The output of the public kernel circuit simulation for this phase */
  publicKernelOutput: PublicKernelCircuitPublicInputs;
  /** The last circuit ran by this phase */
  lastKernelArtifact: ProtocolArtifact;
  /** The final output of the public kernel circuit for this phase */
  finalKernelOutput?: KernelCircuitPublicInputs;
  /** Revert reason, if any */
  revertReason?: SimulationError;
  /** Return values of simulating complete callstack */
  returnValues: NestedProcessReturnValues[];
  /** Gas used during the execution this phase */
  gasUsed?: Gas;
};

export abstract class AbstractPhaseManager {
  protected hintsBuilder: HintsBuilder;
  protected log: DebugLogger;
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    public phase: PublicKernelType,
  ) {
    this.hintsBuilder = new HintsBuilder(db);
    this.log = createDebugLogger(`aztec:sequencer:${phase}`);
  }

  /**
   * @param tx - the tx to be processed
   * @param publicKernelPublicInputs - the output of the public kernel circuit for the previous phase
   */
  abstract handle(
    tx: Tx,
    publicKernelPublicInputs: PublicKernelCircuitPublicInputs,
    previousKernelArtifact: ProtocolArtifact,
  ): Promise<PhaseResult>;

  public static extractEnqueuedPublicCallsByPhase(tx: Tx): Record<PublicKernelType, PublicCallRequest[]> {
    const data = tx.data.forPublic;
    if (!data) {
      return {
        [PublicKernelType.NON_PUBLIC]: [],
        [PublicKernelType.SETUP]: [],
        [PublicKernelType.APP_LOGIC]: [],
        [PublicKernelType.TEARDOWN]: [],
        [PublicKernelType.TAIL]: [],
      };
    }
    const publicCallsStack = tx.enqueuedPublicFunctionCalls.slice().reverse();
    const nonRevertibleCallStack = data.endNonRevertibleData.publicCallStack.filter(i => !i.isEmpty());
    const revertibleCallStack = data.end.publicCallStack.filter(i => !i.isEmpty());

    const callRequestsStack = publicCallsStack
      .map(call => call.toCallRequest())
      .filter(
        // filter out enqueued calls that are not in the public call stack
        // TODO mitch left a question about whether this is only needed when unit testing
        // with mock data
        call => revertibleCallStack.find(p => p.equals(call)) || nonRevertibleCallStack.find(p => p.equals(call)),
      );

    const teardownCallStack = tx.publicTeardownFunctionCall.isEmpty() ? [] : [tx.publicTeardownFunctionCall];

    if (callRequestsStack.length === 0) {
      return {
        [PublicKernelType.NON_PUBLIC]: [],
        [PublicKernelType.SETUP]: [],
        [PublicKernelType.APP_LOGIC]: [],
        [PublicKernelType.TEARDOWN]: teardownCallStack,
        [PublicKernelType.TAIL]: [],
      };
    }

    // find the first call that is revertible
    const firstRevertibleCallIndex = callRequestsStack.findIndex(
      c => revertibleCallStack.findIndex(p => p.equals(c)) !== -1,
    );

    if (firstRevertibleCallIndex === 0) {
      return {
        [PublicKernelType.NON_PUBLIC]: [],
        [PublicKernelType.SETUP]: [],
        [PublicKernelType.APP_LOGIC]: publicCallsStack,
        [PublicKernelType.TEARDOWN]: teardownCallStack,
        [PublicKernelType.TAIL]: [],
      };
    } else if (firstRevertibleCallIndex === -1) {
      // there's no app logic, split the functions between setup (many) and teardown (just one function call)
      return {
        [PublicKernelType.NON_PUBLIC]: [],
        [PublicKernelType.SETUP]: publicCallsStack,
        [PublicKernelType.APP_LOGIC]: [],
        [PublicKernelType.TEARDOWN]: teardownCallStack,
        [PublicKernelType.TAIL]: [],
      };
    } else {
      return {
        [PublicKernelType.NON_PUBLIC]: [],
        [PublicKernelType.SETUP]: publicCallsStack.slice(0, firstRevertibleCallIndex),
        [PublicKernelType.APP_LOGIC]: publicCallsStack.slice(firstRevertibleCallIndex),
        [PublicKernelType.TEARDOWN]: teardownCallStack,
        [PublicKernelType.TAIL]: [],
      };
    }
  }

  protected extractEnqueuedPublicCalls(tx: Tx): PublicCallRequest[] {
    const calls = AbstractPhaseManager.extractEnqueuedPublicCallsByPhase(tx)[this.phase];

    return calls;
  }

  protected async processEnqueuedPublicCalls(
    tx: Tx,
    previousPublicKernelOutput: PublicKernelCircuitPublicInputs,
    previousKernelArtifact: ProtocolArtifact,
  ): Promise<TxPublicCallsResult> {
    const enqueuedCalls = this.extractEnqueuedPublicCalls(tx);

    if (!enqueuedCalls || !enqueuedCalls.length) {
      return {
        publicProvingInformation: [],
        kernelOutput: previousPublicKernelOutput,
        lastKernelArtifact: previousKernelArtifact,
        newUnencryptedLogs: [],
        returnValues: [],
        gasUsed: Gas.empty(),
      };
    }

    // TODO(#1684): Should multiple separately enqueued public calls be treated as
    // separate public callstacks to be proven by separate public kernel sequences
    // and submitted separately to the base rollup?

    const provingInformationList: PublicProvingInformation[] = [];
    const newUnencryptedFunctionLogs: UnencryptedFunctionL2Logs[] = [];
    // Transaction fee is zero for all phases except teardown
    const transactionFee = this.getTransactionFee(tx, previousPublicKernelOutput);
    let gasUsed = Gas.empty();
    let kernelPublicOutput: PublicKernelCircuitPublicInputs = previousPublicKernelOutput;

    const enqueuedCallResults = [];

    for (const enqueuedCall of enqueuedCalls) {
      const executionStack: (PublicExecutionRequest | PublicExecutionResult)[] = [enqueuedCall];

      // Keep track of which result is for the top/enqueued call
      let enqueuedExecutionResult: PublicExecutionResult | undefined;

      while (executionStack.length) {
        const current = executionStack.pop()!;
        const isExecutionRequest = !isPublicExecutionResult(current);
        const result = isExecutionRequest
          ? await this.publicExecutor.simulate(
              /*executionRequest=*/ current,
              this.globalVariables,
              /*availableGas=*/ this.getAvailableGas(tx, kernelPublicOutput),
              tx.data.constants.txContext,
              /*pendingNullifiers=*/ this.getSiloedPendingNullifiers(kernelPublicOutput),
              transactionFee,
              /*startSideEffectCounter=*/ AbstractPhaseManager.getMaxSideEffectCounter(kernelPublicOutput) + 1,
              // NOTE: startSideEffectCounter is not the same as the executionRequest's sideEffectCounter
              // (which counts the request itself)
            )
          : current;

        // Accumulate gas used in this execution
        gasUsed = gasUsed.add(Gas.from(result.startGasLeft).sub(Gas.from(result.endGasLeft)));

        // Sanity check for a current upstream assumption.
        // Consumers of the result seem to expect "reverted <=> revertReason !== undefined".
        const functionSelector = result.executionRequest.functionSelector.toString();
        if (result.reverted && !result.revertReason) {
          throw new Error(
            `Simulation of ${result.executionRequest.contractAddress.toString()}:${functionSelector}(${
              result.functionName
            }) reverted with no reason.`,
          );
        }

        if (result.reverted && !PhaseIsRevertible[this.phase]) {
          this.log.debug(
            `Simulation error on ${result.executionRequest.contractAddress.toString()}:${functionSelector}(${
              result.functionName
            }) with reason: ${result.revertReason}`,
          );
          throw result.revertReason;
        }

        if (isExecutionRequest) {
          newUnencryptedFunctionLogs.push(result.allUnencryptedLogs);
        }
        executionStack.push(...result.nestedExecutions);

        // Simulate the public kernel circuit.
        this.log.debug(
          `Running public kernel circuit for ${result.executionRequest.contractAddress.toString()}:${functionSelector}(${
            result.functionName
          })`,
        );
        const callData = await this.getPublicCallData(result, isExecutionRequest);
        const [privateInputs, publicInputs, artifact] = await this.runKernelCircuit(
          kernelPublicOutput,
          previousKernelArtifact,
          callData,
        );
        kernelPublicOutput = publicInputs;
        previousKernelArtifact = artifact;

        // Capture the inputs for later proving in the AVM and kernel.
        const publicProvingInformation: PublicProvingInformation = {
          functionName: result.functionName,
          calldata: result.calldata,
          bytecode: result.bytecode!,
          inputs: privateInputs,
          avmHints: result.avmCircuitHints,
        };
        provingInformationList.push(publicProvingInformation);

        // Sanity check: Note we can't expect them to just be equal, because e.g.
        // if the simulator reverts in app logic, it "resets" and result.reverted will be false when we run teardown,
        // but the kernel carries the reverted flag forward. But if the simulator reverts, so should the kernel.
        if (result.reverted && kernelPublicOutput.revertCode.isOK()) {
          throw new Error(
            `Public kernel circuit did not revert on ${result.executionRequest.contractAddress.toString()}:${functionSelector}(${
              result.functionName
            }), but simulator did.`,
          );
        }

        // We know the phase is revertible due to the above check.
        // So safely return the revert reason and the kernel output (which has had its revertible side effects dropped)
        if (result.reverted) {
          this.log.debug(
            `Reverting on ${result.executionRequest.contractAddress.toString()}:${functionSelector}(${
              result.functionName
            }) with reason: ${result.revertReason}`,
          );
          // TODO(@spalladino): Check gasUsed is correct. The AVM should take care of setting gasLeft to zero upon a revert.
          return {
            publicProvingInformation: [],
            kernelOutput: kernelPublicOutput,
            lastKernelArtifact: previousKernelArtifact,
            newUnencryptedLogs: [],
            revertReason: result.revertReason,
            returnValues: [],
            gasUsed,
          };
        }

        if (!enqueuedExecutionResult) {
          enqueuedExecutionResult = result;
        }

        enqueuedCallResults.push(accumulateReturnValues(enqueuedExecutionResult));
      }
    }

    return {
      publicProvingInformation: provingInformationList,
      kernelOutput: kernelPublicOutput,
      lastKernelArtifact: previousKernelArtifact,
      newUnencryptedLogs: newUnencryptedFunctionLogs,
      returnValues: enqueuedCallResults,
      gasUsed,
    };
  }

  /** Returns all pending private and public nullifiers. */
  private getSiloedPendingNullifiers(ko: PublicKernelCircuitPublicInputs) {
    return [...ko.end.nullifiers, ...ko.endNonRevertibleData.nullifiers].filter(n => !n.isEmpty());
  }

  protected getAvailableGas(tx: Tx, previousPublicKernelOutput: PublicKernelCircuitPublicInputs) {
    return tx.data.constants.txContext.gasSettings
      .getLimits() // No need to subtract teardown limits since they are already included in end.gasUsed
      .sub(previousPublicKernelOutput.end.gasUsed)
      .sub(previousPublicKernelOutput.endNonRevertibleData.gasUsed);
  }

  protected getTransactionFee(_tx: Tx, _previousPublicKernelOutput: PublicKernelCircuitPublicInputs) {
    return Fr.ZERO;
  }

  private async runKernelCircuit(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousCircuit: ProtocolArtifact,
    callData: PublicCallData,
  ): Promise<[PublicKernelCircuitPrivateInputs, PublicKernelCircuitPublicInputs, ProtocolArtifact]> {
    const previousKernel = this.getPreviousKernelData(previousOutput, previousCircuit);

    // We take a deep copy (clone) of these inputs to be passed to the prover
    const inputs = new PublicKernelCircuitPrivateInputs(previousKernel, ClientIvcProof.empty(), callData);
    switch (this.phase) {
      case PublicKernelType.SETUP:
        return [inputs.clone(), await this.publicKernel.publicKernelCircuitSetup(inputs), 'PublicKernelSetupArtifact'];
      case PublicKernelType.APP_LOGIC:
        return [
          inputs.clone(),
          await this.publicKernel.publicKernelCircuitAppLogic(inputs),
          'PublicKernelAppLogicArtifact',
        ];
      case PublicKernelType.TEARDOWN:
        return [
          inputs.clone(),
          await this.publicKernel.publicKernelCircuitTeardown(inputs),
          'PublicKernelTeardownArtifact',
        ];
      default:
        throw new Error(`No public kernel circuit for inputs`);
    }
  }

  protected getPreviousKernelData(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousCircuit: ProtocolArtifact,
  ): PublicKernelData {
    // The proof is not used in simulation
    const proof = makeEmptyRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH);

    const vk = ProtocolCircuitVks[previousCircuit];
    const vkIndex = ProtocolCircuitVkIndexes[previousCircuit];

    const leafIndex = getVKIndex(vk);

    return new PublicKernelData(previousOutput, proof, vk, vkIndex, getVKSiblingPath(leafIndex));
  }

  protected async getPublicCallStackItem(result: PublicExecutionResult, isExecutionRequest = false) {
    const publicDataTreeInfo = await this.db.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE);
    this.historicalHeader.state.partial.publicDataTree.root = Fr.fromBuffer(publicDataTreeInfo.root);

    const callStackPreimages = await this.getPublicCallStackPreimages(result);
    const publicCallStackHashes = padArrayEnd(
      callStackPreimages.map(c => c.getCompressed().hash()),
      Fr.ZERO,
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    );

    const publicCircuitPublicInputs = PublicCircuitPublicInputs.from({
      callContext: result.executionRequest.callContext,
      proverAddress: AztecAddress.ZERO,
      argsHash: computeVarArgsHash(result.executionRequest.args),
      noteHashes: padArrayEnd(result.noteHashes, NoteHash.empty(), MAX_NOTE_HASHES_PER_CALL),
      nullifiers: padArrayEnd(result.nullifiers, Nullifier.empty(), MAX_NULLIFIERS_PER_CALL),
      l2ToL1Msgs: padArrayEnd(result.l2ToL1Messages, L2ToL1Message.empty(), MAX_L2_TO_L1_MSGS_PER_CALL),
      startSideEffectCounter: result.startSideEffectCounter,
      endSideEffectCounter: result.endSideEffectCounter,
      returnsHash: computeVarArgsHash(result.returnValues),
      noteHashReadRequests: padArrayEnd(
        result.noteHashReadRequests,
        ReadRequest.empty(),
        MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
      ),
      nullifierReadRequests: padArrayEnd(
        result.nullifierReadRequests,
        ReadRequest.empty(),
        MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
      ),
      nullifierNonExistentReadRequests: padArrayEnd(
        result.nullifierNonExistentReadRequests,
        ReadRequest.empty(),
        MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
      ),
      l1ToL2MsgReadRequests: padArrayEnd(
        result.l1ToL2MsgReadRequests,
        ReadRequest.empty(),
        MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL,
      ),
      contractStorageReads: padArrayEnd(
        result.contractStorageReads,
        ContractStorageRead.empty(),
        MAX_PUBLIC_DATA_READS_PER_CALL,
      ),
      contractStorageUpdateRequests: padArrayEnd(
        result.contractStorageUpdateRequests,
        ContractStorageUpdateRequest.empty(),
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
      ),
      publicCallStackHashes,
      unencryptedLogsHashes: padArrayEnd(result.unencryptedLogsHashes, LogHash.empty(), MAX_UNENCRYPTED_LOGS_PER_CALL),
      historicalHeader: this.historicalHeader,
      globalVariables: this.globalVariables,
      startGasLeft: Gas.from(result.startGasLeft),
      endGasLeft: Gas.from(result.endGasLeft),
      transactionFee: result.transactionFee,
      // TODO(@just-mitch): need better mapping from simulator to revert code.
      revertCode: result.reverted ? RevertCode.APP_LOGIC_REVERTED : RevertCode.OK,
    });

    return new PublicCallStackItem(
      result.executionRequest.contractAddress,
      new FunctionData(result.executionRequest.functionSelector, false),
      publicCircuitPublicInputs,
      isExecutionRequest,
    );
  }

  protected async getPublicCallStackPreimages(result: PublicExecutionResult): Promise<PublicCallStackItem[]> {
    const nested = result.nestedExecutions;
    if (nested.length > MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL) {
      throw new Error(
        `Public call stack size exceeded (max ${MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL}, got ${nested.length})`,
      );
    }

    return await Promise.all(nested.map(n => this.getPublicCallStackItem(n)));
  }

  /**
   * Looks at the side effects of a transaction and returns the highest counter
   * @param tx - A transaction
   * @returns The highest side effect counter in the transaction so far
   */
  static getMaxSideEffectCounter(inputs: PublicKernelCircuitPublicInputs): number {
    const sideEffectCounters = [
      ...inputs.endNonRevertibleData.noteHashes,
      ...inputs.endNonRevertibleData.nullifiers,
      ...inputs.endNonRevertibleData.noteEncryptedLogsHashes,
      ...inputs.endNonRevertibleData.encryptedLogsHashes,
      ...inputs.endNonRevertibleData.unencryptedLogsHashes,
      ...inputs.endNonRevertibleData.publicCallStack,
      ...inputs.endNonRevertibleData.publicDataUpdateRequests,
      ...inputs.end.noteHashes,
      ...inputs.end.nullifiers,
      ...inputs.end.noteEncryptedLogsHashes,
      ...inputs.end.encryptedLogsHashes,
      ...inputs.end.unencryptedLogsHashes,
      ...inputs.end.publicCallStack,
      ...inputs.end.publicDataUpdateRequests,
    ];

    let max = 0;
    for (const sideEffect of sideEffectCounters) {
      if ('startSideEffectCounter' in sideEffect) {
        // look at both start and end counters because for enqueued public calls start > 0 while end === 0
        max = Math.max(max, sideEffect.startSideEffectCounter.toNumber(), sideEffect.endSideEffectCounter.toNumber());
      } else if ('counter' in sideEffect) {
        max = Math.max(max, sideEffect.counter);
      } else {
        throw new Error('Unknown side effect type');
      }
    }

    return max;
  }

  protected getBytecodeHash(_result: PublicExecutionResult) {
    // TODO: Determine how to calculate bytecode hash. Circuits just check it isn't zero for now.
    // See https://github.com/AztecProtocol/aztec3-packages/issues/378
    const bytecodeHash = new Fr(1n);
    return Promise.resolve(bytecodeHash);
  }

  /**
   * Calculates the PublicCircuitOutput for this execution result along with its proof,
   * and assembles a PublicCallData object from it.
   * @param result - The execution result.
   * @param preimages - The preimages of the callstack items.
   * @param isExecutionRequest - Whether the current callstack item should be considered a public fn execution request.
   * @returns A corresponding PublicCallData object.
   */
  protected async getPublicCallData(result: PublicExecutionResult, isExecutionRequest = false) {
    const bytecodeHash = await this.getBytecodeHash(result);
    const callStackItem = await this.getPublicCallStackItem(result, isExecutionRequest);
    const publicCallRequests = (await this.getPublicCallStackPreimages(result)).map(c =>
      c.toCallRequest(callStackItem.publicInputs.callContext),
    );
    const publicCallStack = padArrayEnd(publicCallRequests, CallRequest.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
    return new PublicCallData(callStackItem, publicCallStack, makeEmptyProof(), bytecodeHash);
  }
}
