import {
  type BlockProver,
  type FailedTx,
  type ProcessedTx,
  type PublicKernelRequest,
  type SimulationError,
  Tx,
  type TxValidator,
  makeEmptyProcessedTx,
  makeProcessedTx,
  toTxEffect,
  validateProcessedTx,
} from '@aztec/circuit-types';
import { type TxSequencerProcessingStats } from '@aztec/circuit-types/stats';
import { type GlobalVariables, type Header, type KernelCircuitPublicInputs } from '@aztec/circuits.js';
import { type ProcessReturnValues } from '@aztec/foundation/abi';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import { PublicExecutor, type PublicStateDB, type SimulationProvider } from '@aztec/simulator';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { type AbstractPhaseManager, PublicKernelPhase } from './abstract_phase_manager.js';
import { PhaseManagerFactory } from './phase_manager_factory.js';
import { ContractsDataSourcePublicDB, WorldStateDB, WorldStatePublicDB } from './public_executor.js';
import { RealPublicKernelCircuitSimulator } from './public_kernel.js';
import { type PublicKernelCircuitSimulator } from './public_kernel_circuit_simulator.js';

/**
 * Creates new instances of PublicProcessor given the provided merkle tree db and contract data source.
 */
export class PublicProcessorFactory {
  constructor(
    private merkleTree: MerkleTreeOperations,
    private contractDataSource: ContractDataSource,
    private simulator: SimulationProvider,
  ) {}

  /**
   * Creates a new instance of a PublicProcessor.
   * @param historicalHeader - The header of a block previous to the one in which the tx is included.
   * @param globalVariables - The global variables for the block being processed.
   * @param newContracts - Provides access to contract bytecode for public executions.
   * @returns A new instance of a PublicProcessor.
   */
  public async create(
    historicalHeader: Header | undefined,
    globalVariables: GlobalVariables,
  ): Promise<PublicProcessor> {
    historicalHeader = historicalHeader ?? (await this.merkleTree.buildInitialHeader());

    const publicContractsDB = new ContractsDataSourcePublicDB(this.contractDataSource);
    const worldStatePublicDB = new WorldStatePublicDB(this.merkleTree);
    const worldStateDB = new WorldStateDB(this.merkleTree);
    const publicExecutor = new PublicExecutor(worldStatePublicDB, publicContractsDB, worldStateDB, historicalHeader);
    return new PublicProcessor(
      this.merkleTree,
      publicExecutor,
      new RealPublicKernelCircuitSimulator(this.simulator),
      globalVariables,
      historicalHeader,
      publicContractsDB,
      worldStatePublicDB,
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
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,

    private log = createDebugLogger('aztec:sequencer:public-processor'),
  ) {}

  /**
   * Run each tx through the public circuit and the public kernel circuit if needed.
   * @param txs - Txs to process.
   * @returns The list of processed txs with their circuit simulation outputs.
   */
  public async process(
    txs: Tx[],
    maxTransactions = txs.length,
    blockProver?: BlockProver,
    txValidator?: TxValidator<ProcessedTx>,
  ): Promise<[ProcessedTx[], FailedTx[], ProcessReturnValues[]]> {
    // The processor modifies the tx objects in place, so we need to clone them.
    txs = txs.map(tx => Tx.clone(tx));
    const result: ProcessedTx[] = [];
    const failed: FailedTx[] = [];
    const returns: ProcessReturnValues[] = [];

    for (const tx of txs) {
      // only process up to the limit of the block
      if (result.length >= maxTransactions) {
        break;
      }
      try {
        const [processedTx, returnValues] = !tx.hasPublicCalls()
          ? [makeProcessedTx(tx, tx.data.toKernelCircuitPublicInputs(), tx.proof, [])]
          : await this.processTxWithPublicCalls(tx);
        validateProcessedTx(processedTx);
        // Re-validate the transaction
        if (txValidator) {
          // Only accept processed transactions that are not double-spends,
          // public functions emitting nullifiers would pass earlier check but fail here.
          // Note that we're checking all nullifiers generated in the private execution twice,
          // we could store the ones already checked and skip them here as an optimization.
          const [_, invalid] = await txValidator.validateTxs([processedTx]);
          if (invalid.length) {
            throw new Error(`Transaction ${invalid[0].hash} invalid after processing public functions`);
          }
        }
        // if we were given a prover then send the transaction to it for proving
        if (blockProver) {
          await blockProver.addNewTx(processedTx);
        }
        result.push(processedTx);
        returns.push(returnValues);
      } catch (err: any) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        this.log.warn(`Failed to process tx ${tx.getTxHash()}: ${errorMessage}`);

        failed.push({
          tx,
          error: err instanceof Error ? err : new Error(errorMessage),
        });
        returns.push([]);
      }
    }

    return [result, failed, returns];
  }

  /**
   * Makes an empty processed tx. Useful for padding a block to a power of two number of txs.
   * @returns A processed tx with empty data.
   */
  public makeEmptyProcessedTx(): ProcessedTx {
    const { chainId, version } = this.globalVariables;
    return makeEmptyProcessedTx(this.historicalHeader.clone(), chainId, version);
  }

  private async processTxWithPublicCalls(tx: Tx): Promise<[ProcessedTx, ProcessReturnValues | undefined]> {
    let returnValues: ProcessReturnValues = undefined;
    const publicRequests: PublicKernelRequest[] = [];
    let phase: AbstractPhaseManager | undefined = PhaseManagerFactory.phaseFromTx(
      tx,
      this.db,
      this.publicExecutor,
      this.publicKernel,
      this.globalVariables,
      this.historicalHeader,
      this.publicContractsDB,
      this.publicStateDB,
    );
    this.log.debug(`Beginning processing in phase ${phase?.phase} for tx ${tx.getTxHash()}`);
    let proof = tx.proof;
    let publicKernelPublicInput = tx.data.toPublicKernelCircuitPublicInputs();
    let finalKernelOutput: KernelCircuitPublicInputs | undefined;
    let revertReason: SimulationError | undefined;
    const timer = new Timer();
    while (phase) {
      const output = await phase.handle(tx, publicKernelPublicInput, proof);
      if (phase.phase === PublicKernelPhase.APP_LOGIC) {
        returnValues = output.returnValues;
      }
      publicRequests.push(...output.kernelRequests);
      publicKernelPublicInput = output.publicKernelOutput;
      finalKernelOutput = output.finalKernelOutput;
      proof = output.publicKernelProof;
      revertReason ??= output.revertReason;
      phase = PhaseManagerFactory.phaseFromOutput(
        publicKernelPublicInput,
        phase,
        this.db,
        this.publicExecutor,
        this.publicKernel,
        this.globalVariables,
        this.historicalHeader,
        this.publicContractsDB,
        this.publicStateDB,
      );
    }

    if (!finalKernelOutput) {
      throw new Error('Final public kernel was not executed.');
    }

    const processedTx = makeProcessedTx(tx, finalKernelOutput, proof, publicRequests, revertReason);

    this.log.debug(`Processed public part of ${tx.getTxHash()}`, {
      eventName: 'tx-sequencer-processing',
      duration: timer.ms(),
      effectsSize: toTxEffect(processedTx).toBuffer().length,
      publicDataUpdateRequests:
        processedTx.data.end.publicDataUpdateRequests.filter(x => !x.leafSlot.isZero()).length ?? 0,
      ...tx.getStats(),
    } satisfies TxSequencerProcessingStats);

    return [processedTx, returnValues];
  }
}
