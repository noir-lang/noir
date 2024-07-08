import { type BBProverConfig } from '@aztec/bb-prover';
import {
  type BlockProver,
  type BlockResult,
  type ProcessedTx,
  type ProvingTicket,
  type ServerCircuitProver,
  type Tx,
  type TxValidator,
} from '@aztec/circuit-types';
import { type Gas, GlobalVariables, Header, type Nullifier, type TxContext } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';
import { type DebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import {
  type ContractsDataSourcePublicDB,
  type PublicExecutionRequest,
  type PublicExecutionResult,
  PublicExecutionResultBuilder,
  type PublicExecutor,
  PublicProcessor,
  RealPublicKernelCircuitSimulator,
  type SimulationProvider,
  WASMSimulator,
  type WorldStatePublicDB,
} from '@aztec/simulator';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import * as fs from 'fs/promises';
import { type MockProxy, mock } from 'jest-mock-extended';

import { TestCircuitProver } from '../../../bb-prover/src/test/test_circuit_prover.js';
import { ProvingOrchestrator } from '../orchestrator/orchestrator.js';
import { MemoryProvingQueue } from '../prover-agent/memory-proving-queue.js';
import { ProverAgent } from '../prover-agent/prover-agent.js';
import { getEnvironmentConfig, getSimulationProvider, makeGlobals } from './fixtures.js';

class DummyProverClient implements BlockProver {
  constructor(private orchestrator: ProvingOrchestrator) {}
  startNewBlock(numTxs: number, globalVariables: GlobalVariables, l1ToL2Messages: Fr[]): Promise<ProvingTicket> {
    return this.orchestrator.startNewBlock(numTxs, globalVariables, l1ToL2Messages);
  }
  addNewTx(tx: ProcessedTx): Promise<void> {
    return this.orchestrator.addNewTx(tx);
  }
  cancelBlock(): void {
    return this.orchestrator.cancelBlock();
  }
  finaliseBlock(): Promise<BlockResult> {
    return this.orchestrator.finaliseBlock();
  }
  setBlockCompleted(): Promise<void> {
    return this.orchestrator.setBlockCompleted();
  }
}

export class TestContext {
  public blockProver: BlockProver;
  constructor(
    public publicExecutor: MockProxy<PublicExecutor>,
    public publicContractsDB: MockProxy<ContractsDataSourcePublicDB>,
    public publicWorldStateDB: MockProxy<WorldStatePublicDB>,
    public publicProcessor: PublicProcessor,
    public simulationProvider: SimulationProvider,
    public globalVariables: GlobalVariables,
    public actualDb: MerkleTreeOperations,
    public prover: ServerCircuitProver,
    public proverAgent: ProverAgent,
    public orchestrator: ProvingOrchestrator,
    public blockNumber: number,
    public directoriesToCleanup: string[],
    public logger: DebugLogger,
  ) {
    this.blockProver = new DummyProverClient(this.orchestrator);
  }

  static async new(
    logger: DebugLogger,
    proverCount = 4,
    createProver: (bbConfig: BBProverConfig) => Promise<ServerCircuitProver> = _ =>
      Promise.resolve(new TestCircuitProver(new NoopTelemetryClient(), new WASMSimulator())),
    blockNumber = 3,
  ) {
    const globalVariables = makeGlobals(blockNumber);

    const publicExecutor = mock<PublicExecutor>();
    const publicContractsDB = mock<ContractsDataSourcePublicDB>();
    const publicWorldStateDB = mock<WorldStatePublicDB>();
    const publicKernel = new RealPublicKernelCircuitSimulator(new WASMSimulator());
    const actualDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    const telemetry = new NoopTelemetryClient();
    const processor = new PublicProcessor(
      actualDb,
      publicExecutor,
      publicKernel,
      GlobalVariables.empty(),
      Header.empty(),
      publicContractsDB,
      publicWorldStateDB,
      telemetry,
    );

    let localProver: ServerCircuitProver;
    const config = await getEnvironmentConfig(logger);
    const simulationProvider = await getSimulationProvider({
      acvmWorkingDirectory: config?.acvmWorkingDirectory,
      acvmBinaryPath: config?.expectedAcvmPath,
    });
    if (!config) {
      localProver = new TestCircuitProver(new NoopTelemetryClient(), simulationProvider);
    } else {
      const bbConfig: BBProverConfig = {
        acvmBinaryPath: config.expectedAcvmPath,
        acvmWorkingDirectory: config.acvmWorkingDirectory,
        bbBinaryPath: config.expectedBBPath,
        bbWorkingDirectory: config.bbWorkingDirectory,
      };
      localProver = await createProver(bbConfig);
    }

    const queue = new MemoryProvingQueue();
    const orchestrator = new ProvingOrchestrator(actualDb, queue, telemetry);
    const agent = new ProverAgent(localProver, proverCount);

    queue.start();
    agent.start(queue);

    return new this(
      publicExecutor,
      publicContractsDB,
      publicWorldStateDB,
      processor,
      simulationProvider,
      globalVariables,
      actualDb,
      localProver,
      agent,
      orchestrator,
      blockNumber,
      [config?.directoryToCleanup ?? ''],
      logger,
    );
  }

  async cleanup() {
    await this.proverAgent.stop();
    for (const dir of this.directoriesToCleanup.filter(x => x !== '')) {
      await fs.rm(dir, { recursive: true, force: true });
    }
  }

  public async processPublicFunctions(
    txs: Tx[],
    maxTransactions: number,
    blockProver?: BlockProver,
    txValidator?: TxValidator<ProcessedTx>,
  ) {
    const defaultExecutorImplementation = (
      execution: PublicExecutionRequest,
      _globalVariables: GlobalVariables,
      availableGas: Gas,
      _txContext: TxContext,
      _pendingNullifiers: Nullifier[],
      transactionFee?: Fr,
      _sideEffectCounter?: number,
    ) => {
      for (const tx of txs) {
        const allCalls = tx.publicTeardownFunctionCall.isEmpty()
          ? tx.enqueuedPublicFunctionCalls
          : [...tx.enqueuedPublicFunctionCalls, tx.publicTeardownFunctionCall];
        for (const request of allCalls) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            const result = PublicExecutionResultBuilder.fromPublicCallRequest({ request }).build({
              startGasLeft: availableGas,
              endGasLeft: availableGas,
              transactionFee,
            });
            return Promise.resolve(result);
          }
        }
      }
      throw new Error(`Unexpected execution request: ${execution}`);
    };
    return await this.processPublicFunctionsWithMockExecutorImplementation(
      txs,
      maxTransactions,
      blockProver,
      txValidator,
      defaultExecutorImplementation,
    );
  }

  public async processPublicFunctionsWithMockExecutorImplementation(
    txs: Tx[],
    maxTransactions: number,
    blockProver?: BlockProver,
    txValidator?: TxValidator<ProcessedTx>,
    executorMock?: (
      execution: PublicExecutionRequest,
      globalVariables: GlobalVariables,
      availableGas: Gas,
      txContext: TxContext,
      pendingNullifiers: Nullifier[],
      transactionFee?: Fr,
      sideEffectCounter?: number,
    ) => Promise<PublicExecutionResult>,
  ) {
    if (executorMock) {
      this.publicExecutor.simulate.mockImplementation(executorMock);
    }
    return await this.publicProcessor.process(txs, maxTransactions, blockProver, txValidator);
  }
}
