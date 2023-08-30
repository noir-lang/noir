import {
  PublicExecution,
  PublicExecutionResult,
  PublicExecutor,
  collectPublicDataReads,
  collectPublicDataUpdateRequests,
  isPublicExecutionResult,
} from '@aztec/acir-simulator';
import {
  AztecAddress,
  CircuitsWasm,
  CombinedAccumulatedData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  GlobalVariables,
  HistoricBlockData,
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
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicDataRead,
  PublicDataUpdateRequest,
  PublicKernelInputs,
  PublicKernelPublicInputs,
  RETURN_VALUES_LENGTH,
  VK_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { computeCallStackItemHash, computeVarArgsHash } from '@aztec/circuits.js/abis';
import { arrayNonEmptyLength, isArrayEmpty, padArrayEnd, padArrayStart } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { Tuple, mapTuple, to2Fields } from '@aztec/foundation/serialize';
import { ContractDataSource, FunctionL2Logs, L1ToL2MessageSource, MerkleTreeId, Tx } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';

import { getVerificationKeys } from '../index.js';
import { EmptyPublicProver } from '../prover/empty.js';
import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator, getPublicExecutor } from '../simulator/index.js';
import { WasmPublicKernelCircuitSimulator } from '../simulator/public_kernel.js';
import { FailedTx, ProcessedTx, makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';
import { getHistoricBlockData } from './utils.js';

/**
 * Creates new instances of PublicProcessor given the provided merkle tree db and contract data source.
 */
export class PublicProcessorFactory {
  constructor(
    private merkleTree: MerkleTreeOperations,
    private contractDataSource: ContractDataSource,
    private l1Tol2MessagesDataSource: L1ToL2MessageSource,
  ) {}

  /**
   * Creates a new instance of a PublicProcessor.
   * @param prevGlobalVariables - The global variables for the previous block, used to calculate the prev global variables hash.
   * @param globalVariables - The global variables for the block being processed.
   * @returns A new instance of a PublicProcessor.
   */
  public async create(
    prevGlobalVariables: GlobalVariables,
    globalVariables: GlobalVariables,
  ): Promise<PublicProcessor> {
    const blockData = await getHistoricBlockData(this.merkleTree, prevGlobalVariables);
    return new PublicProcessor(
      this.merkleTree,
      getPublicExecutor(this.merkleTree, this.contractDataSource, this.l1Tol2MessagesDataSource, blockData),
      new WasmPublicKernelCircuitSimulator(),
      new EmptyPublicProver(),
      this.contractDataSource,
      globalVariables,
      blockData,
    );
  }
}

/**
 * Converts Txs lifted from the P2P module into ProcessedTx objects by executing
 * any public function calls in them. Txs with private calls only are unaffected.
 */
export class PublicProcessor {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected publicProver: PublicProver,
    protected contractDataSource: ContractDataSource,
    protected globalVariables: GlobalVariables,
    protected blockData: HistoricBlockData,

    private log = createDebugLogger('aztec:sequencer:public-processor'),
  ) {}

  /**
   * Run each tx through the public circuit and the public kernel circuit if needed.
   * @param txs - Txs to process.
   * @returns The list of processed txs with their circuit simulation outputs.
   */
  public async process(txs: Tx[]): Promise<[ProcessedTx[], FailedTx[]]> {
    // The processor modifies the tx objects in place, so we need to clone them.
    txs = txs.map(tx => Tx.fromJSON(tx.toJSON()));
    const result: ProcessedTx[] = [];
    const failed: FailedTx[] = [];

    for (const tx of txs) {
      this.log(`Processing tx ${await tx.getTxHash()}`);
      try {
        result.push(await this.processTx(tx));
      } catch (err) {
        this.log.warn(`Error processing tx ${await tx.getTxHash()}: ${err}`);
        failed.push({
          tx,
          error: err instanceof Error ? err : new Error('Unknown error'),
        });
      }
    }
    return [result, failed];
  }

  /**
   * Makes an empty processed tx. Useful for padding a block to a power of two number of txs.
   * @returns A processed tx with empty data.
   */
  public makeEmptyProcessedTx(): Promise<ProcessedTx> {
    const { chainId, version } = this.globalVariables;
    return makeEmptyProcessedTx(this.blockData, chainId, version);
  }

  protected async processTx(tx: Tx): Promise<ProcessedTx> {
    if (!isArrayEmpty(tx.data.end.publicCallStack, item => item.isZero())) {
      const [publicKernelOutput, publicKernelProof, newUnencryptedFunctionLogs] = await this.processEnqueuedPublicCalls(
        tx,
      );
      tx.unencryptedLogs.addFunctionLogs(newUnencryptedFunctionLogs);

      return makeProcessedTx(tx, publicKernelOutput, publicKernelProof);
    } else {
      return makeProcessedTx(tx);
    }
  }

  protected async processEnqueuedPublicCalls(tx: Tx): Promise<[PublicKernelPublicInputs, Proof, FunctionL2Logs[]]> {
    this.log(`Executing enqueued public calls for tx ${await tx.getTxHash()}`);
    if (!tx.enqueuedPublicFunctionCalls) throw new Error(`Missing preimages for enqueued public calls`);

    let kernelOutput = new KernelCircuitPublicInputs(
      CombinedAccumulatedData.fromFinalAccumulatedData(tx.data.end),
      tx.data.constants,
      tx.data.isPrivate,
    );
    let kernelProof = tx.proof;
    const newUnencryptedFunctionLogs: FunctionL2Logs[] = [];

    // TODO(#1684): Should multiple separately enqueued public calls be treated as
    // separate public callstacks to be proven by separate public kernel sequences
    // and submitted separately to the base rollup?

    // TODO(dbanks12): why must these be reversed?
    const enqueuedCallsReversed = tx.enqueuedPublicFunctionCalls.slice().reverse();
    for (const enqueuedCall of enqueuedCallsReversed) {
      const executionStack: (PublicExecution | PublicExecutionResult)[] = [enqueuedCall];

      // Keep track of which result is for the top/enqueued call
      let enqueuedExecutionResult: PublicExecutionResult | undefined;

      while (executionStack.length) {
        const current = executionStack.pop()!;
        const isExecutionRequest = !isPublicExecutionResult(current);
        const result = isExecutionRequest ? await this.publicExecutor.execute(current, this.globalVariables) : current;
        newUnencryptedFunctionLogs.push(result.unencryptedLogs);
        const functionSelector = result.execution.functionData.selector.toString();
        this.log(
          `Running public kernel circuit for ${functionSelector}@${result.execution.contractAddress.toString()}`,
        );
        executionStack.push(...result.nestedExecutions);
        const preimages = await this.getPublicCallStackPreimages(result);
        const callData = await this.getPublicCallData(result, preimages, isExecutionRequest);

        [kernelOutput, kernelProof] = await this.runKernelCircuit(callData, kernelOutput, kernelProof);

        if (!enqueuedExecutionResult) enqueuedExecutionResult = result;
      }
      // HACK(#1622): Manually patches the ordering of public state actions
      // TODO(#757): Enforce proper ordering of public state actions
      await this.patchPublicStorageActionOrdering(kernelOutput, enqueuedExecutionResult!);
    }

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
    this.blockData.publicDataTreeRoot = Fr.fromBuffer(publicDataTreeInfo.root);

    const callStackPreimages = await this.getPublicCallStackPreimages(result);
    const wasm = await CircuitsWasm.get();

    const publicCallStack = mapTuple(callStackPreimages, item =>
      item.isEmpty() ? Fr.zero() : computeCallStackItemHash(wasm, item),
    );

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165) --> set this in Noir
    const unencryptedLogsHash = to2Fields(result.unencryptedLogs.hash());
    const unencryptedLogPreimagesLength = new Fr(result.unencryptedLogs.getSerializedLength());

    return PublicCircuitPublicInputs.from({
      callContext: result.execution.callContext,
      proverAddress: AztecAddress.ZERO,
      argsHash: await computeVarArgsHash(wasm, result.execution.args),
      newCommitments: padArrayEnd(result.newCommitments, Fr.ZERO, MAX_NEW_COMMITMENTS_PER_CALL),
      newNullifiers: padArrayEnd(result.newNullifiers, Fr.ZERO, MAX_NEW_NULLIFIERS_PER_CALL),
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
      publicCallStack,
      unencryptedLogsHash,
      unencryptedLogPreimagesLength,
      historicBlockData: this.blockData,
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

  protected async getPublicCallStackPreimages(result: PublicExecutionResult) {
    const nested = result.nestedExecutions;
    const preimages: PublicCallStackItem[] = await Promise.all(nested.map(n => this.getPublicCallStackItem(n)));
    if (preimages.length > MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL) {
      throw new Error(
        `Public call stack size exceeded (max ${MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL}, got ${preimages.length})`,
      );
    }

    // Top of the stack is at the end of the array, so we padStart
    return padArrayStart(preimages, PublicCallStackItem.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
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
  protected async getPublicCallData(
    result: PublicExecutionResult,
    preimages: Tuple<PublicCallStackItem, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    isExecutionRequest = false,
  ) {
    const bytecodeHash = await this.getBytecodeHash(result);
    const callStackItem = await this.getPublicCallStackItem(result, isExecutionRequest);
    const portalContractAddress = result.execution.callContext.portalContractAddress.toField();
    const proof = await this.publicProver.getPublicCircuitProof(callStackItem.publicInputs);
    return new PublicCallData(callStackItem, preimages, proof, portalContractAddress, bytecodeHash);
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
  private async patchPublicStorageActionOrdering(
    publicInputs: KernelCircuitPublicInputs,
    execResult: PublicExecutionResult,
  ) {
    // Convert ContractStorage* objects to PublicData* objects and sort them in execution order
    const wasm = await CircuitsWasm.get();
    const simPublicDataReads = collectPublicDataReads(wasm, execResult);
    const simPublicDataUpdateRequests = collectPublicDataUpdateRequests(wasm, execResult);

    const { publicDataReads, publicDataUpdateRequests } = publicInputs.end; // from kernel

    // Validate all items in enqueued public calls are in the kernel emitted stack
    const readsAreEqual = simPublicDataReads.reduce(
      (accum, read) =>
        accum && !!publicDataReads.find(item => item.leafIndex.equals(read.leafIndex) && item.value.equals(read.value)),
      true,
    );
    const updatesAreEqual = simPublicDataUpdateRequests.reduce(
      (accum, update) =>
        accum &&
        !!publicDataUpdateRequests.find(
          item =>
            item.leafIndex.equals(update.leafIndex) &&
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
      f => f.leafIndex.equals(Fr.ZERO) && f.value.equals(Fr.ZERO),
    );
    const numTotalUpdatesInKernel = arrayNonEmptyLength(
      publicInputs.end.publicDataUpdateRequests,
      f => f.leafIndex.equals(Fr.ZERO) && f.oldValue.equals(Fr.ZERO) && f.newValue.equals(Fr.ZERO),
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
}
