import {
  Body,
  L2Block,
  MerkleTreeId,
  type ProcessedTx,
  type PublicKernelRequest,
  PublicKernelType,
  type TxEffect,
  toTxEffect,
} from '@aztec/circuit-types';
import {
  type BlockResult,
  PROVING_STATUS,
  type ProvingResult,
  type ProvingTicket,
  type PublicInputsAndProof,
} from '@aztec/circuit-types/interfaces';
import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  type BaseRollupInputs,
  Fr,
  type GlobalVariables,
  type KernelCircuitPublicInputs,
  L1_TO_L2_MSG_SUBTREE_HEIGHT,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  type Proof,
  type PublicKernelCircuitPublicInputs,
  type RECURSIVE_PROOF_LENGTH,
  type RootParityInput,
  RootParityInputs,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { promiseWithResolvers } from '@aztec/foundation/promise';
import { type Tuple } from '@aztec/foundation/serialize';
import { sleep } from '@aztec/foundation/sleep';
import { Timer } from '@aztec/foundation/timer';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { inspect } from 'util';

import { type CircuitProver } from '../prover/interface.js';
import {
  buildBaseRollupInput,
  createMergeRollupInputs,
  getRootRollupInput,
  getSubtreeSiblingPath,
  getTreeSnapshot,
  validatePartialState,
  validateRootOutput,
  validateTx,
} from './block-building-helpers.js';
import { type MergeRollupInputData, ProvingState, type TreeSnapshots } from './proving-state.js';
import { TX_PROVING_CODE, TxProvingState } from './tx-proving-state.js';

const logger = createDebugLogger('aztec:prover:proving-orchestrator');

/**
 * Implements an event driven proving scheduler to build the recursive proof tree. The idea being:
 * 1. Transactions are provided to the scheduler post simulation.
 * 2. Tree insertions are performed as required to generate transaction specific proofs
 * 3. Those transaction specific proofs are generated in the necessary order accounting for dependencies
 * 4. Once a transaction is proven, it will be incorporated into a merge proof
 * 5. Merge proofs are produced at each level of the tree until the root proof is produced
 *
 * The proving implementation is determined by the provided prover. This could be for example a local prover or a remote prover pool.
 */

const KernelTypesWithoutFunctions: Set<PublicKernelType> = new Set<PublicKernelType>([
  PublicKernelType.NON_PUBLIC,
  PublicKernelType.TAIL,
]);

/**
 * The orchestrator, managing the flow of recursive proving operations required to build the rollup proof tree.
 */
export class ProvingOrchestrator {
  private provingState: ProvingState | undefined = undefined;
  constructor(private db: MerkleTreeOperations, private prover: CircuitProver) {}

  /**
   * Starts off a new block
   * @param numTxs - The total number of transactions in the block. Must be a power of 2
   * @param globalVariables - The global variables for the block
   * @param l1ToL2Messages - The l1 to l2 messages for the block
   * @param emptyTx - The instance of an empty transaction to be used to pad this block
   * @returns A proving ticket, containing a promise notifying of proving completion
   */
  public async startNewBlock(
    numTxs: number,
    globalVariables: GlobalVariables,
    l1ToL2Messages: Fr[],
    emptyTx: ProcessedTx,
  ): Promise<ProvingTicket> {
    // Check that the length of the array of txs is a power of two
    // See https://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
    if (!Number.isInteger(numTxs) || numTxs < 2 || (numTxs & (numTxs - 1)) !== 0) {
      throw new Error(`Length of txs for the block should be a power of two and at least two (got ${numTxs})`);
    }
    // Cancel any currently proving block before starting a new one
    this.cancelBlock();
    logger.info(`Starting new block with ${numTxs} transactions`);
    // we start the block by enqueueing all of the base parity circuits
    let baseParityInputs: BaseParityInputs[] = [];
    let l1ToL2MessagesPadded: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>;
    try {
      l1ToL2MessagesPadded = padArrayEnd(l1ToL2Messages, Fr.ZERO, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);
    } catch (err) {
      throw new Error('Too many L1 to L2 messages');
    }
    baseParityInputs = Array.from({ length: NUM_BASE_PARITY_PER_ROOT_PARITY }, (_, i) =>
      BaseParityInputs.fromSlice(l1ToL2MessagesPadded, i),
    );

    const messageTreeSnapshot = await getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, this.db);

    const newL1ToL2MessageTreeRootSiblingPathArray = await getSubtreeSiblingPath(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      L1_TO_L2_MSG_SUBTREE_HEIGHT,
      this.db,
    );

    const newL1ToL2MessageTreeRootSiblingPath = makeTuple(
      L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
      i =>
        i < newL1ToL2MessageTreeRootSiblingPathArray.length ? newL1ToL2MessageTreeRootSiblingPathArray[i] : Fr.ZERO,
      0,
    );

    // Update the local trees to include the new l1 to l2 messages
    await this.db.appendLeaves(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, l1ToL2MessagesPadded);

    const { promise: _promise, resolve, reject } = promiseWithResolvers<ProvingResult>();
    const promise = _promise.catch(
      (reason): ProvingResult => ({
        status: PROVING_STATUS.FAILURE,
        reason,
      }),
    );

    const provingState = new ProvingState(
      numTxs,
      resolve,
      reject,
      globalVariables,
      l1ToL2MessagesPadded,
      baseParityInputs.length,
      emptyTx,
      messageTreeSnapshot,
      newL1ToL2MessageTreeRootSiblingPath,
    );

    for (let i = 0; i < baseParityInputs.length; i++) {
      this.enqueueBaseParityCircuit(provingState, baseParityInputs[i], i);
    }

    this.provingState = provingState;

    const ticket: ProvingTicket = {
      provingPromise: promise,
    };
    return ticket;
  }

  /**
   * The interface to add a simulated transaction to the scheduler
   * @param tx - The transaction to be proven
   */
  public async addNewTx(tx: ProcessedTx): Promise<void> {
    if (!this.provingState) {
      throw new Error(`Invalid proving state, call startNewBlock before adding transactions`);
    }

    if (!this.provingState.isAcceptingTransactions()) {
      throw new Error(`Rollup not accepting further transactions`);
    }

    validateTx(tx);

    logger.info(`Received transaction: ${tx.hash}`);

    await this.startTransaction(tx, this.provingState);
  }

  /**
   * Marks the block as full and pads it to the full power of 2 block size, no more transactions will be accepted.
   */
  public async setBlockCompleted() {
    if (!this.provingState) {
      throw new Error(`Invalid proving state, call startNewBlock before adding transactions or completing the block`);
    }

    // we need to pad the rollup with empty transactions
    logger.info(
      `Padding rollup with ${
        this.provingState.totalNumTxs - this.provingState.transactionsReceived
      } empty transactions`,
    );
    for (let i = this.provingState.transactionsReceived; i < this.provingState.totalNumTxs; i++) {
      await this.startTransaction(this.provingState.emptyTx, this.provingState);
    }
  }

  /**
   * Cancel any further proving of the block
   */
  public cancelBlock() {
    this.provingState?.cancel();
  }

  /**
   * Performs the final tree update for the block and returns the fully proven block.
   * @returns The fully proven block and proof.
   */
  public async finaliseBlock() {
    if (!this.provingState || !this.provingState.rootRollupPublicInputs || !this.provingState.finalProof) {
      throw new Error(`Invalid proving state, a block must be proven before it can be finalised`);
    }
    if (this.provingState.block) {
      throw new Error('Block already finalised');
    }

    const rootRollupOutputs = this.provingState.rootRollupPublicInputs;

    logger?.debug(`Updating and validating root trees`);
    await this.db.updateArchive(rootRollupOutputs.header);

    await validateRootOutput(rootRollupOutputs, this.db);

    // Collect all new nullifiers, commitments, and contracts from all txs in this block
    const nonEmptyTxEffects: TxEffect[] = this.provingState!.allTxs.map(txProvingState =>
      toTxEffect(txProvingState.processedTx),
    ).filter(txEffect => !txEffect.isEmpty());
    const blockBody = new Body(nonEmptyTxEffects);

    const l2Block = L2Block.fromFields({
      archive: rootRollupOutputs.archive,
      header: rootRollupOutputs.header,
      body: blockBody,
    });

    if (!l2Block.body.getTxsEffectsHash().equals(rootRollupOutputs.header.contentCommitment.txsEffectsHash)) {
      logger.debug(inspect(blockBody));
      throw new Error(
        `Txs effects hash mismatch, ${l2Block.body
          .getTxsEffectsHash()
          .toString('hex')} == ${rootRollupOutputs.header.contentCommitment.txsEffectsHash.toString('hex')} `,
      );
    }

    logger.info(`Successfully proven block ${l2Block.number}!`);

    this.provingState.block = l2Block;

    const blockResult: BlockResult = {
      proof: this.provingState.finalProof,
      block: l2Block,
    };

    return blockResult;
  }

  /**
   * Starts the proving process for the given transaction and adds it to our state
   * @param tx - The transaction whose proving we wish to commence
   * @param provingState - The proving state being worked on
   */
  private async startTransaction(tx: ProcessedTx, provingState: ProvingState) {
    const txInputs = await this.prepareBaseRollupInputs(provingState, tx);
    if (!txInputs) {
      // This should not be possible
      throw new Error(`Unable to add padding transaction, preparing base inputs failed`);
    }
    const [inputs, treeSnapshots] = txInputs;
    const txProvingState = new TxProvingState(tx, inputs, treeSnapshots);
    const txIndex = provingState.addNewTx(txProvingState);
    const numPublicKernels = txProvingState.getNumPublicKernels();
    if (!numPublicKernels) {
      // no public functions, go straight to the base rollup
      logger.debug(`Enqueueing base rollup for tx ${txIndex}`);
      this.enqueueBaseRollup(provingState, BigInt(txIndex), txProvingState);
      return;
    }
    // Enqueue all of the VM proving requests
    // Rather than handle the Kernel Tail as a special case here, we will just handle it inside executeVM
    for (let i = 0; i < numPublicKernels; i++) {
      logger.debug(`Enqueueing public VM ${i} for tx ${txIndex}`);
      this.enqueueVM(provingState, txIndex, i);
    }
  }

  /**
   * Enqueue a job to be scheduled
   * @param provingState - The proving state object being operated on
   * @param jobType - The type of job to be queued
   * @param job - The actual job, returns a promise notifying of the job's completion
   */
  private deferredProving<T>(
    provingState: ProvingState | undefined,
    request: () => Promise<T>,
    callback: (result: T, durationMs: number) => void | Promise<void>,
  ) {
    if (!provingState?.verifyState()) {
      logger.debug(`Not enqueuing job, state no longer valid`);
      return;
    }
    // We use a 'safeJob'. We don't want promise rejections in the proving pool, we want to capture the error here
    // and reject the proving job whilst keeping the event loop free of rejections
    const safeJob = async () => {
      try {
        const timer = new Timer();
        const result = await request();
        const duration = timer.ms();

        if (!provingState?.verifyState()) {
          logger.debug(`State no longer valid, discarding result`);
          return;
        }

        await callback(result, duration);
      } catch (err) {
        logger.error(`Error thrown when proving job`);
        provingState!.reject(`${err}`);
      }
    };

    // let the callstack unwind before adding the job to the queue
    setImmediate(safeJob);
  }

  private emitCircuitSimulationStats(
    circuitName: CircuitSimulationStats['circuitName'] | null,
    inputSize: number,
    outputSize: number,
    duration: number,
  ) {
    const stats: CircuitSimulationStats | undefined = circuitName
      ? {
          eventName: 'circuit-simulation',
          circuitName,
          duration,
          inputSize,
          outputSize,
        }
      : undefined;
    logger.debug(`Simulated ${circuitName} circuit duration=${duration}ms`, stats);
  }

  private getPublicKernelCircuitName(request: PublicKernelRequest) {
    switch (request.type) {
      case PublicKernelType.SETUP:
        return 'public-kernel-setup';
      case PublicKernelType.APP_LOGIC:
        return 'public-kernel-app-logic';
      case PublicKernelType.TEARDOWN:
        return 'public-kernel-teardown';
      case PublicKernelType.TAIL:
        return 'public-kernel-tail';
      default:
        return null;
    }
  }

  // Updates the merkle trees for a transaction. The first enqueued job for a transaction
  private async prepareBaseRollupInputs(
    provingState: ProvingState | undefined,
    tx: ProcessedTx,
  ): Promise<[BaseRollupInputs, TreeSnapshots] | undefined> {
    if (!provingState?.verifyState()) {
      logger.debug('Not preparing base rollup inputs, state invalid');
      return;
    }
    const inputs = await buildBaseRollupInput(tx, provingState.globalVariables, this.db);
    const promises = [MerkleTreeId.NOTE_HASH_TREE, MerkleTreeId.NULLIFIER_TREE, MerkleTreeId.PUBLIC_DATA_TREE].map(
      async (id: MerkleTreeId) => {
        return { key: id, value: await getTreeSnapshot(id, this.db) };
      },
    );
    const treeSnapshots: TreeSnapshots = new Map((await Promise.all(promises)).map(obj => [obj.key, obj.value]));

    if (!provingState?.verifyState()) {
      logger.debug(`Discarding proving job, state no longer valid`);
      return;
    }
    return [inputs, treeSnapshots];
  }

  // Stores the intermediate inputs prepared for a merge proof
  private storeMergeInputs(
    provingState: ProvingState,
    currentLevel: bigint,
    currentIndex: bigint,
    mergeInputs: [BaseOrMergeRollupPublicInputs, Proof],
  ) {
    const mergeLevel = currentLevel - 1n;
    const indexWithinMergeLevel = currentIndex >> 1n;
    const mergeIndex = 2n ** mergeLevel - 1n + indexWithinMergeLevel;
    const subscript = Number(mergeIndex);
    const indexWithinMerge = Number(currentIndex & 1n);
    const ready = provingState.storeMergeInputs(mergeInputs, indexWithinMerge, subscript);
    return { ready, indexWithinMergeLevel, mergeLevel, mergeInputData: provingState.getMergeInputs(subscript) };
  }

  // Executes the base rollup circuit and stored the output as intermediate state for the parent merge/root circuit
  // Executes the next level of merge if all inputs are available
  private enqueueBaseRollup(provingState: ProvingState | undefined, index: bigint, tx: TxProvingState) {
    if (!provingState?.verifyState()) {
      logger.debug('Not running base rollup, state invalid');
      return;
    }
    if (
      !tx.baseRollupInputs.kernelData.publicInputs.end.encryptedLogsHash
        .toBuffer()
        .equals(tx.processedTx.encryptedLogs.hash())
    ) {
      provingState.reject(
        `Encrypted logs hash mismatch: ${
          tx.baseRollupInputs.kernelData.publicInputs.end.encryptedLogsHash
        } === ${Fr.fromBuffer(tx.processedTx.encryptedLogs.hash())}`,
      );
      return;
    }
    if (
      !tx.baseRollupInputs.kernelData.publicInputs.end.unencryptedLogsHash
        .toBuffer()
        .equals(tx.processedTx.unencryptedLogs.hash())
    ) {
      provingState.reject(
        `Unencrypted logs hash mismatch: ${
          tx.baseRollupInputs.kernelData.publicInputs.end.unencryptedLogsHash
        } === ${Fr.fromBuffer(tx.processedTx.unencryptedLogs.hash())}`,
      );
      return;
    }

    this.deferredProving(
      provingState,
      () => this.prover.getBaseRollupProof(tx.baseRollupInputs),
      (result, duration) => {
        this.emitCircuitSimulationStats(
          'base-rollup',
          tx.baseRollupInputs.toBuffer().length,
          result.inputs.toBuffer().length,
          duration,
        );

        validatePartialState(result.inputs.end, tx.treeSnapshots);
        const currentLevel = provingState.numMergeLevels + 1n;
        this.storeAndExecuteNextMergeLevel(provingState, currentLevel, index, [result.inputs, result.proof]);
      },
    );
  }

  // Executes the merge rollup circuit and stored the output as intermediate state for the parent merge/root circuit
  // Enqueues the next level of merge if all inputs are available
  private enqueueMergeRollup(
    provingState: ProvingState,
    level: bigint,
    index: bigint,
    mergeInputData: MergeRollupInputData,
  ) {
    const inputs = createMergeRollupInputs(
      [mergeInputData.inputs[0]!, mergeInputData.proofs[0]!],
      [mergeInputData.inputs[1]!, mergeInputData.proofs[1]!],
    );

    this.deferredProving(
      provingState,
      () => this.prover.getMergeRollupProof(inputs),
      (result, duration) => {
        this.emitCircuitSimulationStats(
          'merge-rollup',
          inputs.toBuffer().length,
          result.inputs.toBuffer().length,
          duration,
        );
        this.storeAndExecuteNextMergeLevel(provingState, level, index, [result.inputs, result.proof]);
      },
    );
  }

  // Executes the root rollup circuit
  private async enqueueRootRollup(provingState: ProvingState | undefined) {
    if (!provingState?.verifyState()) {
      logger.debug('Not running root rollup, state no longer valid');
      return;
    }
    const mergeInputData = provingState.getMergeInputs(0);
    const rootParityInput = provingState.finalRootParityInput!;

    const inputs = await getRootRollupInput(
      mergeInputData.inputs[0]!,
      mergeInputData.proofs[0]!,
      mergeInputData.inputs[1]!,
      mergeInputData.proofs[1]!,
      rootParityInput,
      provingState.newL1ToL2Messages,
      provingState.messageTreeSnapshot,
      provingState.messageTreeRootSiblingPath,
      this.db,
    );

    this.deferredProving(
      provingState,
      () => this.prover.getRootRollupProof(inputs),
      (result, duration) => {
        this.emitCircuitSimulationStats(
          'root-rollup',
          inputs.toBuffer().length,
          result.inputs.toBuffer().length,
          duration,
        );

        provingState.rootRollupPublicInputs = result.inputs;
        provingState.finalProof = result.proof;

        const provingResult: ProvingResult = {
          status: PROVING_STATUS.SUCCESS,
        };
        provingState.resolve(provingResult);
      },
    );
  }

  // Executes the base parity circuit and stores the intermediate state for the root parity circuit
  // Enqueues the root parity circuit if all inputs are available
  private enqueueBaseParityCircuit(provingState: ProvingState, inputs: BaseParityInputs, index: number) {
    this.deferredProving(
      provingState,
      () => this.prover.getBaseParityProof(inputs),
      (rootInput, duration) => {
        this.emitCircuitSimulationStats(
          'base-parity',
          inputs.toBuffer().length,
          rootInput.publicInputs.toBuffer().length,
          duration,
        );
        provingState.setRootParityInputs(rootInput, index);
        if (provingState.areRootParityInputsReady()) {
          const rootParityInputs = new RootParityInputs(
            provingState.rootParityInput as Tuple<
              RootParityInput<typeof RECURSIVE_PROOF_LENGTH>,
              typeof NUM_BASE_PARITY_PER_ROOT_PARITY
            >,
          );
          this.enqueueRootParityCircuit(provingState, rootParityInputs);
        }
      },
    );
  }

  // Runs the root parity circuit ans stored the outputs
  // Enqueues the root rollup proof if all inputs are available
  private enqueueRootParityCircuit(provingState: ProvingState | undefined, inputs: RootParityInputs) {
    this.deferredProving(
      provingState,
      () => this.prover.getRootParityProof(inputs),
      async (rootInput, duration) => {
        this.emitCircuitSimulationStats(
          'root-parity',
          inputs.toBuffer().length,
          rootInput.publicInputs.toBuffer().length,
          duration,
        );
        provingState!.finalRootParityInput = rootInput;
        await this.checkAndEnqueueRootRollup(provingState);
      },
    );
  }

  private async checkAndEnqueueRootRollup(provingState: ProvingState | undefined) {
    if (!provingState?.isReadyForRootRollup()) {
      logger.debug('Not ready for root rollup');
      return;
    }
    await this.enqueueRootRollup(provingState);
  }

  /**
   * Stores the inputs to a merge/root circuit and enqueues the circuit if ready
   * @param provingState - The proving state being operated on
   * @param currentLevel - The level of the merge/root circuit
   * @param currentIndex - The index of the merge/root circuit
   * @param mergeInputData - The inputs to be stored
   */
  private storeAndExecuteNextMergeLevel(
    provingState: ProvingState,
    currentLevel: bigint,
    currentIndex: bigint,
    mergeInputData: [BaseOrMergeRollupPublicInputs, Proof],
  ) {
    const result = this.storeMergeInputs(provingState, currentLevel, currentIndex, mergeInputData);

    // Are we ready to execute the next circuit?
    if (!result.ready) {
      return;
    }

    if (result.mergeLevel === 0n) {
      // TODO (alexg) remove this `void`
      void this.checkAndEnqueueRootRollup(provingState);
    } else {
      // onto the next merge level
      this.enqueueMergeRollup(provingState, result.mergeLevel, result.indexWithinMergeLevel, result.mergeInputData);
    }
  }

  /**
   * Executes the VM circuit for a public function, will enqueue the corresponding kernel if the
   * previous kernel is ready
   * @param provingState - The proving state being operated on
   * @param txIndex - The index of the transaction being proven
   * @param functionIndex - The index of the function/kernel being proven
   */
  private enqueueVM(provingState: ProvingState | undefined, txIndex: number, functionIndex: number) {
    if (!provingState?.verifyState()) {
      logger.debug(`Not running VM circuit as state is no longer valid`);
      return;
    }

    const txProvingState = provingState.getTxProvingState(txIndex);
    const publicFunction = txProvingState.getPublicFunctionState(functionIndex);

    // Prove the VM if this is a kernel that requires one
    if (!KernelTypesWithoutFunctions.has(publicFunction.publicKernelRequest.type)) {
      // Just sleep for a small amount of time
      this.deferredProving(
        provingState,
        () => sleep(100),
        () => {
          logger.debug(`Proven VM for function index ${functionIndex} of tx index ${txIndex}`);
          this.checkAndEnqueuePublicKernel(provingState, txIndex, functionIndex);
        },
      );
    } else {
      this.checkAndEnqueuePublicKernel(provingState, txIndex, functionIndex);
    }
  }

  private checkAndEnqueuePublicKernel(provingState: ProvingState, txIndex: number, functionIndex: number) {
    const txProvingState = provingState.getTxProvingState(txIndex);
    const kernelRequest = txProvingState.getNextPublicKernelFromVMProof(functionIndex, makeEmptyProof());
    if (kernelRequest.code === TX_PROVING_CODE.READY) {
      if (kernelRequest.function === undefined) {
        // Should not be possible
        throw new Error(`Error occurred, public function request undefined after VM proof completed`);
      }
      logger.debug(`Enqueuing kernel from VM for tx ${txIndex}, function ${functionIndex}`);
      this.enqueuePublicKernel(provingState, txIndex, functionIndex);
    }
  }

  /**
   * Executes the kernel circuit for a public function, will enqueue the next kernel circuit if it's VM is already proven
   * or the base rollup circuit if there are no more kernels to be proven
   * @param provingState - The proving state being operated on
   * @param txIndex - The index of the transaction being proven
   * @param functionIndex - The index of the function/kernel being proven
   */
  private enqueuePublicKernel(provingState: ProvingState | undefined, txIndex: number, functionIndex: number) {
    if (!provingState?.verifyState()) {
      logger.debug(`Not running public kernel circuit as state is no longer valid`);
      return;
    }

    const txProvingState = provingState.getTxProvingState(txIndex);
    const request = txProvingState.getPublicFunctionState(functionIndex).publicKernelRequest;

    this.deferredProving(
      provingState,
      (): Promise<PublicInputsAndProof<KernelCircuitPublicInputs | PublicKernelCircuitPublicInputs>> => {
        if (request.type === PublicKernelType.TAIL) {
          return this.prover.getPublicTailProof(request);
        } else {
          return this.prover.getPublicKernelProof(request);
        }
      },
      (result, duration) => {
        this.emitCircuitSimulationStats(
          this.getPublicKernelCircuitName(request),
          request.inputs.toBuffer().length,
          0,
          duration,
        );

        const nextKernelRequest = txProvingState.getNextPublicKernelFromKernelProof(functionIndex, result.proof);
        // What's the status of the next kernel?
        if (nextKernelRequest.code === TX_PROVING_CODE.NOT_READY) {
          // Must be waiting on a VM proof
          return;
        }

        if (nextKernelRequest.code === TX_PROVING_CODE.COMPLETED) {
          // We must have completed all public function proving, we now move to the base rollup
          logger.debug(`Public functions completed for tx ${txIndex} enqueueing base rollup`);
          this.enqueueBaseRollup(provingState, BigInt(txIndex), txProvingState);
          return;
        }
        // There must be another kernel ready to be proven
        if (nextKernelRequest.function === undefined) {
          // Should not be possible
          throw new Error(`Error occurred, public function request undefined after kernel proof completed`);
        }

        this.enqueuePublicKernel(provingState, txIndex, functionIndex + 1);
      },
    );
  }
}
