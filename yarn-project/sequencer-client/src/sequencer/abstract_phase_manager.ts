import {
  PublicExecution,
  PublicExecutionResult,
  PublicExecutor,
  collectPublicDataReads,
  collectPublicDataUpdateRequests,
  isPublicExecutionResult,
} from '@aztec/acir-simulator';
import { FunctionL2Logs, MerkleTreeId, Tx } from '@aztec/circuit-types';
import {
  AztecAddress,
  CallRequest,
  CombinedAccumulatedData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  GlobalVariables,
  Header,
  KernelCircuitPublicInputs,
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MembershipWitness,
  PreviousKernelData,
  Proof,
  PublicCallData,
  PublicCallRequest,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicDataRead,
  PublicDataUpdateRequest,
  PublicKernelInputs,
  PublicKernelPublicInputs,
  RETURN_VALUES_LENGTH,
  SideEffect,
  SideEffectLinkedToNoteHash,
  VK_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/abis';
import { arrayNonEmptyLength, padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { to2Fields } from '@aztec/foundation/serialize';
import { MerkleTreeOperations } from '@aztec/world-state';

import { getVerificationKeys } from '../mocks/verification_keys.js';
import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { FailedTx } from './processed_tx.js';

/**
 * A phase manager is responsible for performing/rolling back a phase of a transaction.
 *
 * The phases are as follows:
 * 1. Fee Preparation
 * 2. Application Logic
 * 3. Fee Distribution
 */
export abstract class AbstractPhaseManager {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected publicProver: PublicProver,
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    protected log = createDebugLogger('aztec:sequencer:phase-manager'),
  ) {}
  /**
   *
   * @param tx - the tx to be processed
   * @param previousPublicKernelOutput - the output of the public kernel circuit for the previous phase
   * @param previousPublicKernelProof - the proof of the public kernel circuit for the previous phase
   */
  abstract handle(
    tx: Tx,
    previousPublicKernelOutput?: PublicKernelPublicInputs,
    previousPublicKernelProof?: Proof,
  ): Promise<{
    /**
     * the output of the public kernel circuit for this phase
     */
    publicKernelOutput?: PublicKernelPublicInputs;
    /**
     * the proof of the public kernel circuit for this phase
     */
    publicKernelProof?: Proof;
  }>;
  abstract nextPhase(): AbstractPhaseManager | null;
  abstract rollback(tx: Tx, err: unknown): Promise<FailedTx>;

  // Extract the public calls from the tx for this phase
  abstract extractEnqueuedPublicCalls(tx: Tx): PublicCallRequest[];

  protected getKernelOutputAndProof(
    tx: Tx,
    previousPublicKernelOutput?: PublicKernelPublicInputs,
    previousPublicKernelProof?: Proof,
  ): {
    /**
     * the output of the public kernel circuit for this phase
     */
    publicKernelOutput: PublicKernelPublicInputs;
    /**
     * the proof of the public kernel circuit for this phase
     */
    publicKernelProof: Proof;
  } {
    if (previousPublicKernelOutput && previousPublicKernelProof) {
      return {
        publicKernelOutput: previousPublicKernelOutput,
        publicKernelProof: previousPublicKernelProof,
      };
    } else {
      const publicKernelOutput = new KernelCircuitPublicInputs(
        CombinedAccumulatedData.fromFinalAccumulatedData(tx.data.end),
        tx.data.constants,
        tx.data.isPrivate,
      );
      const publicKernelProof = previousPublicKernelProof || tx.proof;
      return {
        publicKernelOutput,
        publicKernelProof,
      };
    }
  }

  protected async processEnqueuedPublicCalls(
    enqueuedCalls: PublicCallRequest[],
    previousPublicKernelOutput: PublicKernelPublicInputs,
    previousPublicKernelProof: Proof,
  ): Promise<[PublicKernelPublicInputs, Proof, FunctionL2Logs[]]> {
    if (!enqueuedCalls || !enqueuedCalls.length) {
      throw new Error(`Missing preimages for enqueued public calls`);
    }
    let kernelOutput = previousPublicKernelOutput;
    let kernelProof = previousPublicKernelProof;

    const newUnencryptedFunctionLogs: FunctionL2Logs[] = [];

    // TODO(#1684): Should multiple separately enqueued public calls be treated as
    // separate public callstacks to be proven by separate public kernel sequences
    // and submitted separately to the base rollup?

    for (const enqueuedCall of enqueuedCalls) {
      const executionStack: (PublicExecution | PublicExecutionResult)[] = [enqueuedCall];

      // Keep track of which result is for the top/enqueued call
      let enqueuedExecutionResult: PublicExecutionResult | undefined;

      while (executionStack.length) {
        const current = executionStack.pop()!;
        const isExecutionRequest = !isPublicExecutionResult(current);
        const result = isExecutionRequest ? await this.publicExecutor.simulate(current, this.globalVariables) : current;
        newUnencryptedFunctionLogs.push(result.unencryptedLogs);
        const functionSelector = result.execution.functionData.selector.toString();
        this.log(
          `Running public kernel circuit for ${functionSelector}@${result.execution.contractAddress.toString()}`,
        );
        executionStack.push(...result.nestedExecutions);
        const callData = await this.getPublicCallData(result, isExecutionRequest);

        [kernelOutput, kernelProof] = await this.runKernelCircuit(callData, kernelOutput, kernelProof);

        if (!enqueuedExecutionResult) {
          enqueuedExecutionResult = result;
        }
      }
      // HACK(#1622): Manually patches the ordering of public state actions
      // TODO(#757): Enforce proper ordering of public state actions
      this.patchPublicStorageActionOrdering(kernelOutput, enqueuedExecutionResult!);
    }

    // TODO(#3675): This should be done in a public kernel circuit
    this.removeRedundantPublicDataWrites(kernelOutput);

    return [kernelOutput, kernelProof, newUnencryptedFunctionLogs];
  }

  protected async runKernelCircuit(
    callData: PublicCallData,
    previousOutput: KernelCircuitPublicInputs,
    previousProof: Proof,
  ): Promise<[KernelCircuitPublicInputs, Proof]> {
    const output = await this.getKernelCircuitOutput(callData, previousOutput, previousProof);
    const proof = await this.publicProver.getPublicKernelCircuitProof(output);
    return [output, proof];
  }

  protected getKernelCircuitOutput(
    callData: PublicCallData,
    previousOutput: KernelCircuitPublicInputs,
    previousProof: Proof,
  ): Promise<KernelCircuitPublicInputs> {
    if (previousOutput?.isPrivate && previousProof) {
      // Run the public kernel circuit with previous private kernel
      const previousKernel = this.getPreviousKernelData(previousOutput, previousProof);
      const inputs = new PublicKernelInputs(previousKernel, callData);
      return this.publicKernel.publicKernelCircuitPrivateInput(inputs);
    } else if (previousOutput && previousProof) {
      // Run the public kernel circuit with previous public kernel
      const previousKernel = this.getPreviousKernelData(previousOutput, previousProof);
      const inputs = new PublicKernelInputs(previousKernel, callData);
      return this.publicKernel.publicKernelCircuitNonFirstIteration(inputs);
    } else {
      throw new Error(`No public kernel circuit for inputs`);
    }
  }

  protected getPreviousKernelData(previousOutput: KernelCircuitPublicInputs, previousProof: Proof): PreviousKernelData {
    const vk = getVerificationKeys().publicKernelCircuit;
    const vkIndex = 0;
    const vkSiblingPath = MembershipWitness.random(VK_TREE_HEIGHT).siblingPath;
    return new PreviousKernelData(previousOutput, previousProof, vk, vkIndex, vkSiblingPath);
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
    const unencryptedLogsHash = to2Fields(result.unencryptedLogs.hash());
    const unencryptedLogPreimagesLength = new Fr(result.unencryptedLogs.getSerializedLength());

    return PublicCircuitPublicInputs.from({
      callContext: result.execution.callContext,
      proverAddress: AztecAddress.ZERO,
      argsHash: computeVarArgsHash(result.execution.args),
      newCommitments: padArrayEnd(result.newCommitments, SideEffect.empty(), MAX_NEW_COMMITMENTS_PER_CALL),
      newNullifiers: padArrayEnd(result.newNullifiers, SideEffectLinkedToNoteHash.empty(), MAX_NEW_NULLIFIERS_PER_CALL),
      newL2ToL1Msgs: padArrayEnd(result.newL2ToL1Messages, Fr.ZERO, MAX_NEW_L2_TO_L1_MSGS_PER_CALL),
      returnValues: padArrayEnd(result.returnValues, Fr.ZERO, RETURN_VALUES_LENGTH),
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
    const publicCallRequests = (await this.getPublicCallStackPreimages(result)).map(c => c.toCallRequest());
    const publicCallStack = padArrayEnd(publicCallRequests, CallRequest.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
    const portalContractAddress = result.execution.callContext.portalContractAddress.toField();
    const proof = await this.publicProver.getPublicCircuitProof(callStackItem.publicInputs);
    return new PublicCallData(callStackItem, publicCallStack, proof, portalContractAddress, bytecodeHash);
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
  private patchPublicStorageActionOrdering(publicInputs: KernelCircuitPublicInputs, execResult: PublicExecutionResult) {
    // Convert ContractStorage* objects to PublicData* objects and sort them in execution order
    const simPublicDataReads = collectPublicDataReads(execResult);
    const simPublicDataUpdateRequests = collectPublicDataUpdateRequests(execResult);

    const { publicDataReads, publicDataUpdateRequests } = publicInputs.end; // from kernel

    // Validate all items in enqueued public calls are in the kernel emitted stack
    const readsAreEqual = simPublicDataReads.reduce(
      (accum, read) =>
        accum && !!publicDataReads.find(item => item.leafSlot.equals(read.leafSlot) && item.value.equals(read.value)),
      true,
    );
    const updatesAreEqual = simPublicDataUpdateRequests.reduce(
      (accum, update) =>
        accum &&
        !!publicDataUpdateRequests.find(
          item =>
            item.leafSlot.equals(update.leafSlot) &&
            item.oldValue.equals(update.oldValue) &&
            item.newValue.equals(update.newValue),
        ),
      true,
    );

    if (!readsAreEqual) {
      throw new Error(
        `Public data reads from simulator do not match those from public kernel.\nFrom simulator: ${simPublicDataReads
          .map(p => p.toFriendlyJSON())
          .join(', ')}\nFrom public kernel: ${publicDataReads.map(i => i.toFriendlyJSON()).join(', ')}`,
      );
    }
    if (!updatesAreEqual) {
      throw new Error(
        `Public data update requests from simulator do not match those from public kernel.\nFrom simulator: ${simPublicDataUpdateRequests
          .map(p => p.toFriendlyJSON())
          .join(', ')}\nFrom public kernel: ${publicDataUpdateRequests.map(i => i.toFriendlyJSON()).join(', ')}`,
      );
    }

    // Assume that kernel public inputs has the right number of items.
    // We only want to reorder the items from the public inputs of the
    // most recently processed top/enqueued call.
    const numTotalReadsInKernel = arrayNonEmptyLength(
      publicInputs.end.publicDataReads,
      f => f.leafSlot.equals(Fr.ZERO) && f.value.equals(Fr.ZERO),
    );
    const numTotalUpdatesInKernel = arrayNonEmptyLength(
      publicInputs.end.publicDataUpdateRequests,
      f => f.leafSlot.equals(Fr.ZERO) && f.oldValue.equals(Fr.ZERO) && f.newValue.equals(Fr.ZERO),
    );
    const numReadsBeforeThisEnqueuedCall = numTotalReadsInKernel - simPublicDataReads.length;
    const numUpdatesBeforeThisEnqueuedCall = numTotalUpdatesInKernel - simPublicDataUpdateRequests.length;

    // Override kernel output
    publicInputs.end.publicDataReads = padArrayEnd(
      [
        // do not mess with items from previous top/enqueued calls in kernel output
        ...publicDataReads.slice(0, numReadsBeforeThisEnqueuedCall),
        ...simPublicDataReads,
      ],
      PublicDataRead.empty(),
      MAX_PUBLIC_DATA_READS_PER_TX,
    );

    // Override kernel output
    publicInputs.end.publicDataUpdateRequests = padArrayEnd(
      [
        // do not mess with items from previous top/enqueued calls in kernel output
        ...publicDataUpdateRequests.slice(0, numUpdatesBeforeThisEnqueuedCall),
        ...simPublicDataUpdateRequests,
      ],
      PublicDataUpdateRequest.empty(),
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    );
  }

  private removeRedundantPublicDataWrites(publicInputs: KernelCircuitPublicInputs) {
    const lastWritesMap = new Map();
    for (const write of publicInputs.end.publicDataUpdateRequests) {
      const key = write.leafSlot.toString();
      lastWritesMap.set(key, write);
    }

    const lastWrites = publicInputs.end.publicDataUpdateRequests.filter(
      write => lastWritesMap.get(write.leafSlot.toString()) === write,
    );

    publicInputs.end.publicDataUpdateRequests = padArrayEnd(
      lastWrites,

      PublicDataUpdateRequest.empty(),
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    );
  }
}
