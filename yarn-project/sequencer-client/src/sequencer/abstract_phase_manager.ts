import { MerkleTreeId, type SimulationError, type Tx, type UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  AztecAddress,
  CallRequest,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  type GlobalVariables,
  type Header,
  type KernelCircuitPublicInputs,
  L2ToL1Message,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NOTE_HASHES_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
  MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MembershipWitness,
  type PrivateKernelTailCircuitPublicInputs,
  type Proof,
  PublicCallData,
  type PublicCallRequest,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicDataRead,
  PublicDataUpdateRequest,
  PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  PublicKernelData,
  RETURN_VALUES_LENGTH,
  ReadRequest,
  RevertCode,
  SideEffect,
  SideEffectLinkedToNoteHash,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import {
  type ABIType,
  type DecodedReturn,
  type FunctionArtifact,
  type ProcessReturnValues,
  decodeReturnValues,
} from '@aztec/foundation/abi';
import { arrayNonEmptyLength, padArrayEnd } from '@aztec/foundation/collection';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';
import {
  type PublicExecution,
  type PublicExecutionResult,
  type PublicExecutor,
  collectPublicDataReads,
  collectPublicDataUpdateRequests,
  isPublicExecutionResult,
} from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { type PublicKernelCircuitSimulator } from '../simulator/index.js';
import { HintsBuilder } from './hints_builder.js';
import { lastSideEffectCounter } from './utils.js';

export enum PublicKernelPhase {
  SETUP = 'setup',
  APP_LOGIC = 'app-logic',
  TEARDOWN = 'teardown',
  TAIL = 'tail',
}

export const PhaseIsRevertible: Record<PublicKernelPhase, boolean> = {
  [PublicKernelPhase.SETUP]: false,
  [PublicKernelPhase.APP_LOGIC]: true,
  [PublicKernelPhase.TEARDOWN]: false,
  [PublicKernelPhase.TAIL]: false,
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
    public phase: PublicKernelPhase,
  ) {
    this.hintsBuilder = new HintsBuilder(db);
    this.log = createDebugLogger(`aztec:sequencer:${phase}`);
  }
  /**
   *
   * @param tx - the tx to be processed
   * @param publicKernelPublicInputs - the output of the public kernel circuit for the previous phase
   * @param previousPublicKernelProof - the proof of the public kernel circuit for the previous phase
   */
  abstract handle(
    tx: Tx,
    publicKernelPublicInputs: PublicKernelCircuitPublicInputs,
    previousPublicKernelProof: Proof,
  ): Promise<{
    /**
     * the output of the public kernel circuit for this phase
     */
    publicKernelOutput: PublicKernelCircuitPublicInputs;
    /**
     * the final output of the public kernel circuit for this phase
     */
    finalKernelOutput?: KernelCircuitPublicInputs;
    /**
     * the proof of the public kernel circuit for this phase
     */
    publicKernelProof: Proof;
    /**
     * revert reason, if any
     */
    revertReason: SimulationError | undefined;
    returnValues: ProcessReturnValues;
  }>;

  public static extractEnqueuedPublicCallsByPhase(
    publicInputs: PrivateKernelTailCircuitPublicInputs,
    enqueuedPublicFunctionCalls: PublicCallRequest[],
  ): Record<PublicKernelPhase, PublicCallRequest[]> {
    const publicCallsStack = enqueuedPublicFunctionCalls.slice().reverse();
    const nonRevertibleCallStack = publicInputs.forPublic!.endNonRevertibleData.publicCallStack.filter(
      i => !i.isEmpty(),
    );
    const revertibleCallStack = publicInputs.forPublic!.end.publicCallStack.filter(i => !i.isEmpty());

    const callRequestsStack = publicCallsStack
      .map(call => call.toCallRequest())
      .filter(
        // filter out enqueued calls that are not in the public call stack
        // TODO mitch left a question about whether this is only needed when unit testing
        // with mock data
        call => revertibleCallStack.find(p => p.equals(call)) || nonRevertibleCallStack.find(p => p.equals(call)),
      );

    if (callRequestsStack.length === 0) {
      return {
        [PublicKernelPhase.SETUP]: [],
        [PublicKernelPhase.APP_LOGIC]: [],
        [PublicKernelPhase.TEARDOWN]: [],
        [PublicKernelPhase.TAIL]: [],
      };
    }

    // find the first call that is revertible
    const firstRevertibleCallIndex = callRequestsStack.findIndex(
      c => revertibleCallStack.findIndex(p => p.equals(c)) !== -1,
    );

    if (firstRevertibleCallIndex === 0) {
      return {
        [PublicKernelPhase.SETUP]: [],
        [PublicKernelPhase.APP_LOGIC]: publicCallsStack,
        [PublicKernelPhase.TEARDOWN]: [],
        [PublicKernelPhase.TAIL]: [],
      };
    } else if (firstRevertibleCallIndex === -1) {
      // there's no app logic, split the functions between setup (many) and teardown (just one function call)
      return {
        [PublicKernelPhase.SETUP]: publicCallsStack.slice(0, -1),
        [PublicKernelPhase.APP_LOGIC]: [],
        [PublicKernelPhase.TEARDOWN]: [publicCallsStack[publicCallsStack.length - 1]],
        [PublicKernelPhase.TAIL]: [],
      };
    } else {
      return {
        [PublicKernelPhase.SETUP]: publicCallsStack.slice(0, firstRevertibleCallIndex - 1),
        [PublicKernelPhase.APP_LOGIC]: publicCallsStack.slice(firstRevertibleCallIndex),
        [PublicKernelPhase.TEARDOWN]: [publicCallsStack[firstRevertibleCallIndex - 1]],
        [PublicKernelPhase.TAIL]: [],
      };
    }
  }

  protected extractEnqueuedPublicCalls(tx: Tx): PublicCallRequest[] {
    const calls = AbstractPhaseManager.extractEnqueuedPublicCallsByPhase(tx.data, tx.enqueuedPublicFunctionCalls)[
      this.phase
    ];

    return calls;
  }

  protected async processEnqueuedPublicCalls(
    tx: Tx,
    previousPublicKernelOutput: PublicKernelCircuitPublicInputs,
    previousPublicKernelProof: Proof,
  ): Promise<
    [
      PublicKernelCircuitPublicInputs,
      Proof,
      UnencryptedFunctionL2Logs[],
      SimulationError | undefined,
      ProcessReturnValues,
    ]
  > {
    let kernelOutput = previousPublicKernelOutput;
    let kernelProof = previousPublicKernelProof;

    const enqueuedCalls = this.extractEnqueuedPublicCalls(tx);

    if (!enqueuedCalls || !enqueuedCalls.length) {
      return [kernelOutput, kernelProof, [], undefined, undefined];
    }

    const newUnencryptedFunctionLogs: UnencryptedFunctionL2Logs[] = [];

    // TODO(#1684): Should multiple separately enqueued public calls be treated as
    // separate public callstacks to be proven by separate public kernel sequences
    // and submitted separately to the base rollup?

    const returns = [];

    for (const enqueuedCall of enqueuedCalls) {
      const executionStack: (PublicExecution | PublicExecutionResult)[] = [enqueuedCall];

      let currentReturn: DecodedReturn | undefined = undefined;

      // Keep track of which result is for the top/enqueued call
      let enqueuedExecutionResult: PublicExecutionResult | undefined;

      while (executionStack.length) {
        const current = executionStack.pop()!;
        const isExecutionRequest = !isPublicExecutionResult(current);

        const sideEffectCounter = lastSideEffectCounter(tx) + 1;
        const result = isExecutionRequest
          ? await this.publicExecutor.simulate(current, this.globalVariables, sideEffectCounter)
          : current;

        const functionSelector = result.execution.functionData.selector.toString();
        if (result.reverted && !PhaseIsRevertible[this.phase]) {
          this.log.debug(
            `Simulation error on ${result.execution.contractAddress.toString()}:${functionSelector} with reason: ${
              result.revertReason
            }`,
          );
          throw result.revertReason;
        }

        newUnencryptedFunctionLogs.push(result.unencryptedLogs);

        this.log.debug(
          `Running public kernel circuit for ${result.execution.contractAddress.toString()}:${functionSelector}`,
        );
        executionStack.push(...result.nestedExecutions);
        const callData = await this.getPublicCallData(result, isExecutionRequest);

        [kernelOutput, kernelProof] = await this.runKernelCircuit(kernelOutput, kernelProof, callData);

        // sanity check. Note we can't expect them to just be equal, because e.g.
        // if the simulator reverts in app logic, it "resets" and result.reverted will be false when we run teardown,
        // but the kernel carries the reverted flag forward. But if the simulator reverts, so should the kernel.
        if (result.reverted && kernelOutput.revertCode.isOK()) {
          throw new Error(
            `Public kernel circuit did not revert on ${result.execution.contractAddress.toString()}:${functionSelector}, but simulator did.`,
          );
        }

        // We know the phase is revertible due to the above check.
        // So safely return the revert reason and the kernel output (which has had its revertible side effects dropped)
        if (result.reverted) {
          this.log.debug(
            `Reverting on ${result.execution.contractAddress.toString()}:${functionSelector} with reason: ${
              result.revertReason
            }`,
          );
          return [kernelOutput, kernelProof, [], result.revertReason, undefined];
        }

        if (!enqueuedExecutionResult) {
          enqueuedExecutionResult = result;

          // Padding as the AVM is not always returning the expected return size (4)
          // which is expected by the kernel.
          const paddedReturn = padArrayEnd(result.returnValues, Fr.ZERO, RETURN_VALUES_LENGTH);

          // TODO(#5450) Need to use the proper return values here
          const returnTypes: ABIType[] = [{ kind: 'array', length: 4, type: { kind: 'field' } }];
          const mockArtifact = { returnTypes } as any as FunctionArtifact;

          currentReturn = decodeReturnValues(mockArtifact, paddedReturn);
        }
      }
      // HACK(#1622): Manually patches the ordering of public state actions
      // TODO(#757): Enforce proper ordering of public state actions
      patchPublicStorageActionOrdering(kernelOutput, enqueuedExecutionResult!, this.phase);

      returns.push(currentReturn);
    }

    // TODO(#3675): This should be done in a public kernel circuit
    removeRedundantPublicDataWrites(kernelOutput, this.phase);

    return [kernelOutput, kernelProof, newUnencryptedFunctionLogs, undefined, returns];
  }

  protected async runKernelCircuit(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousProof: Proof,
    callData: PublicCallData,
  ): Promise<[PublicKernelCircuitPublicInputs, Proof]> {
    const output = await this.getKernelCircuitOutput(previousOutput, previousProof, callData);
    return [output, makeEmptyProof()];
  }

  protected getKernelCircuitOutput(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousProof: Proof,
    callData: PublicCallData,
  ): Promise<PublicKernelCircuitPublicInputs> {
    const previousKernel = this.getPreviousKernelData(previousOutput, previousProof);

    const inputs = new PublicKernelCircuitPrivateInputs(previousKernel, callData);
    switch (this.phase) {
      case PublicKernelPhase.SETUP:
        return this.publicKernel.publicKernelCircuitSetup(inputs);
      case PublicKernelPhase.APP_LOGIC:
        return this.publicKernel.publicKernelCircuitAppLogic(inputs);
      case PublicKernelPhase.TEARDOWN:
        return this.publicKernel.publicKernelCircuitTeardown(inputs);
      default:
        throw new Error(`No public kernel circuit for inputs`);
    }
  }

  protected getPreviousKernelData(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousProof: Proof,
  ): PublicKernelData {
    // TODO(@PhilWindle) Fix once we move this to prover-client
    const vk = VerificationKey.makeFake();
    const vkIndex = 0;
    const vkSiblingPath = MembershipWitness.random(VK_TREE_HEIGHT).siblingPath;
    return new PublicKernelData(previousOutput, previousProof, vk, vkIndex, vkSiblingPath);
  }

  protected async getPublicCircuitPublicInputs(result: PublicExecutionResult) {
    const publicDataTreeInfo = await this.db.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE);
    this.historicalHeader.state.partial.publicDataTree.root = Fr.fromBuffer(publicDataTreeInfo.root);

    const callStackPreimages = await this.getPublicCallStackPreimages(result);
    const publicCallStackHashes = padArrayEnd(
      callStackPreimages.map(c => c.hash()),
      Fr.ZERO,
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    );

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165) --> set this in Noir
    const unencryptedLogsHash = Fr.fromBuffer(result.unencryptedLogs.hash());
    const unencryptedLogPreimagesLength = new Fr(result.unencryptedLogs.getSerializedLength());

    return PublicCircuitPublicInputs.from({
      callContext: result.execution.callContext,
      proverAddress: AztecAddress.ZERO,
      argsHash: computeVarArgsHash(result.execution.args),
      newNoteHashes: padArrayEnd(result.newNoteHashes, SideEffect.empty(), MAX_NEW_NOTE_HASHES_PER_CALL),
      newNullifiers: padArrayEnd(result.newNullifiers, SideEffectLinkedToNoteHash.empty(), MAX_NEW_NULLIFIERS_PER_CALL),
      newL2ToL1Msgs: padArrayEnd(result.newL2ToL1Messages, L2ToL1Message.empty(), MAX_NEW_L2_TO_L1_MSGS_PER_CALL),
      startSideEffectCounter: result.startSideEffectCounter,
      endSideEffectCounter: result.endSideEffectCounter,
      returnValues: padArrayEnd(result.returnValues, Fr.ZERO, RETURN_VALUES_LENGTH),
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
      unencryptedLogsHash,
      unencryptedLogPreimagesLength,
      historicalHeader: this.historicalHeader,
      // TODO(@just-mitch): need better mapping from simulator to revert code.
      revertCode: result.reverted ? RevertCode.REVERTED : RevertCode.OK,
    });
  }

  protected async getPublicCallStackItem(result: PublicExecutionResult, isExecutionRequest = false) {
    return new PublicCallStackItem(
      result.execution.contractAddress,
      result.execution.functionData,
      await this.getPublicCircuitPublicInputs(result),
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
    const portalContractAddress = result.execution.callContext.portalContractAddress.toField();
    return new PublicCallData(callStackItem, publicCallStack, makeEmptyProof(), portalContractAddress, bytecodeHash);
  }
}

function removeRedundantPublicDataWrites(publicInputs: PublicKernelCircuitPublicInputs, phase: PublicKernelPhase) {
  const lastWritesMap = new Map<string, boolean>();
  const patch = <N extends number>(requests: Tuple<PublicDataUpdateRequest, N>) =>
    requests.filter(write => {
      const leafSlot = write.leafSlot.toString();
      const exists = lastWritesMap.get(leafSlot);
      lastWritesMap.set(leafSlot, true);
      return !exists;
    });

  const [prev, curr] = PhaseIsRevertible[phase]
    ? [publicInputs.endNonRevertibleData, publicInputs.end]
    : [publicInputs.end, publicInputs.endNonRevertibleData];

  curr.publicDataUpdateRequests = padArrayEnd(
    patch(curr.publicDataUpdateRequests.reverse()).reverse(),
    PublicDataUpdateRequest.empty(),
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  );

  prev.publicDataUpdateRequests = padArrayEnd(
    patch(prev.publicDataUpdateRequests.reverse()),
    PublicDataUpdateRequest.empty(),
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  );
}

// HACK(#1622): this is a hack to fix ordering of public state in the call stack. Since the private kernel
// cannot keep track of side effects that happen after or before a nested call, we override the public
// state actions it emits with whatever we got from the simulator. As a sanity check, we at least verify
// that the elements are the same, so we are only tweaking their ordering.
// See yarn-project/end-to-end/src/e2e_ordering.test.ts
// See https://github.com/AztecProtocol/aztec-packages/issues/1616
// TODO(#757): Enforce proper ordering of public state actions
/**
 * Patch the ordering of storage actions output from the public kernel.
 * @param publicInputs - to be patched here: public inputs to the kernel iteration up to this point
 * @param execResult - result of the top/first execution for this enqueued public call
 */
function patchPublicStorageActionOrdering(
  publicInputs: PublicKernelCircuitPublicInputs,
  execResult: PublicExecutionResult,
  phase: PublicKernelPhase,
) {
  const { publicDataUpdateRequests } = PhaseIsRevertible[phase] ? publicInputs.end : publicInputs.endNonRevertibleData;
  const { publicDataReads } = publicInputs.validationRequests;

  // Convert ContractStorage* objects to PublicData* objects and sort them in execution order.
  // Note, this only pulls simulated reads/writes from the current phase,
  // so the returned result will be a subset of the public kernel output.

  const simPublicDataReads = collectPublicDataReads(execResult);
  // verify that each read is in the kernel output
  for (const read of simPublicDataReads) {
    if (!publicDataReads.find(item => item.equals(read))) {
      throw new Error(
        `Public data reads from simulator do not match those from public kernel.\nFrom simulator: ${simPublicDataReads
          .map(p => p.toFriendlyJSON())
          .join(', ')}\nFrom public kernel: ${publicDataReads.map(i => i.toFriendlyJSON()).join(', ')}`,
      );
    }
  }

  const simPublicDataUpdateRequests = collectPublicDataUpdateRequests(execResult);
  for (const update of simPublicDataUpdateRequests) {
    if (!publicDataUpdateRequests.find(item => item.equals(update))) {
      throw new Error(
        `Public data update requests from simulator do not match those from public kernel.\nFrom simulator: ${simPublicDataUpdateRequests
          .map(p => p.toFriendlyJSON())
          .join(', ')}\nFrom public kernel revertible: ${publicDataUpdateRequests
          .map(i => i.toFriendlyJSON())
          .join(', ')}`,
      );
    }
  }
  // We only want to reorder the items from the public inputs of the
  // most recently processed top/enqueued call.

  const effectSet = PhaseIsRevertible[phase] ? 'end' : 'endNonRevertibleData';

  const numReadsInKernel = arrayNonEmptyLength(publicDataReads, f => f.isEmpty());
  const numReadsBeforeThisEnqueuedCall = numReadsInKernel - simPublicDataReads.length;
  publicInputs.validationRequests.publicDataReads = padArrayEnd(
    [
      // do not mess with items from previous top/enqueued calls in kernel output
      ...publicInputs.validationRequests.publicDataReads.slice(0, numReadsBeforeThisEnqueuedCall),
      ...simPublicDataReads,
    ],
    PublicDataRead.empty(),
    MAX_PUBLIC_DATA_READS_PER_TX,
  );

  const numUpdatesInKernel = arrayNonEmptyLength(publicDataUpdateRequests, f => f.isEmpty());
  const numUpdatesBeforeThisEnqueuedCall = numUpdatesInKernel - simPublicDataUpdateRequests.length;
  publicInputs[effectSet].publicDataUpdateRequests = padArrayEnd(
    [
      ...publicInputs[effectSet].publicDataUpdateRequests.slice(0, numUpdatesBeforeThisEnqueuedCall),
      ...simPublicDataUpdateRequests,
    ],
    PublicDataUpdateRequest.empty(),
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  );
}
