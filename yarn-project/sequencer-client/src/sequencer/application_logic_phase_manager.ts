import { PublicExecutor, PublicStateDB } from '@aztec/acir-simulator';
import { Tx } from '@aztec/circuit-types';
import { GlobalVariables, Header, Proof, PublicCallRequest, PublicKernelPublicInputs } from '@aztec/circuits.js';
import { isArrayEmpty } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { MerkleTreeOperations } from '@aztec/world-state';

import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ContractsDataSourcePublicDB } from '../simulator/public_executor.js';
import { AbstractPhaseManager } from './abstract_phase_manager.js';
import { FeeDistributionPhaseManager } from './fee_distribution_phase_manager.js';
import { FailedTx } from './processed_tx.js';

/**
 * The phase manager responsible for performing the fee preparation phase.
 */
export class ApplicationLogicPhaseManager extends AbstractPhaseManager {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected publicProver: PublicProver,
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,

    protected log = createDebugLogger('aztec:sequencer:application-logic'),
  ) {
    super(db, publicExecutor, publicKernel, publicProver, globalVariables, historicalHeader);
  }

  extractEnqueuedPublicCalls(tx: Tx): PublicCallRequest[] {
    if (!tx.enqueuedPublicFunctionCalls) {
      throw new Error(`Missing preimages for enqueued public calls`);
    }
    // Note: the first enqueued public call is for fee payments
    // TODO(dbanks12): why must these be reversed?
    return tx.enqueuedPublicFunctionCalls.slice().reverse();
  }

  async handle(
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
  }> {
    // add new contracts to the contracts db so that their functions may be found and called
    this.log(`Processing tx ${await tx.getTxHash()}`);
    await this.publicContractsDB.addNewContracts(tx);
    if (!isArrayEmpty(tx.data.end.publicCallStack, item => item.isEmpty())) {
      const outputAndProof = this.getKernelOutputAndProof(tx, previousPublicKernelOutput, previousPublicKernelProof);

      this.log(`Executing enqueued public calls for tx ${await tx.getTxHash()}`);
      const [publicKernelOutput, publicKernelProof, newUnencryptedFunctionLogs] = await this.processEnqueuedPublicCalls(
        this.extractEnqueuedPublicCalls(tx),
        outputAndProof.publicKernelOutput,
        outputAndProof.publicKernelProof,
      );
      tx.unencryptedLogs.addFunctionLogs(newUnencryptedFunctionLogs);

      // commit the state updates from this transaction
      await this.publicStateDB.commit();

      return { publicKernelOutput, publicKernelProof };
    } else {
      return {
        publicKernelOutput: undefined,
        publicKernelProof: undefined,
      };
    }
  }

  nextPhase() {
    return new FeeDistributionPhaseManager(
      this.db,
      this.publicExecutor,
      this.publicKernel,
      this.publicProver,
      this.globalVariables,
      this.historicalHeader,
      this.publicContractsDB,
      this.publicStateDB,
    );
  }

  async rollback(tx: Tx, err: unknown): Promise<FailedTx> {
    this.log.warn(`Error processing tx ${await tx.getTxHash()}: ${err}`);
    // remove contracts on failure
    await this.publicContractsDB.removeNewContracts(tx);
    // rollback any state updates from this failed transaction
    await this.publicStateDB.rollback();
    return {
      tx,
      error: err instanceof Error ? err : new Error('Unknown error'),
    };
  }
}
