import { type BlockProver, type ProcessedTx, type Tx, type TxValidator } from '@aztec/circuit-types';
import { GlobalVariables, Header } from '@aztec/circuits.js';
import { type DebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import {
  type ContractsDataSourcePublicDB,
  type PublicExecution,
  type PublicExecutionResult,
  PublicExecutionResultBuilder,
  type PublicExecutor,
  PublicProcessor,
  RealPublicKernelCircuitSimulator,
  type SimulationProvider,
  WASMSimulator,
  type WorldStatePublicDB,
} from '@aztec/simulator';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import * as fs from 'fs/promises';
import { type MockProxy, mock } from 'jest-mock-extended';

import { ProvingOrchestrator } from '../orchestrator/orchestrator.js';
import { type BBProverConfig } from '../prover/bb_prover.js';
import { type CircuitProver } from '../prover/interface.js';
import { TestCircuitProver } from '../prover/test_circuit_prover.js';
import { getEnvironmentConfig, getSimulationProvider, makeGlobals } from './fixtures.js';

export class TestContext {
  constructor(
    public publicExecutor: MockProxy<PublicExecutor>,
    public publicContractsDB: MockProxy<ContractsDataSourcePublicDB>,
    public publicWorldStateDB: MockProxy<WorldStatePublicDB>,
    public publicProcessor: PublicProcessor,
    public simulationProvider: SimulationProvider,
    public globalVariables: GlobalVariables,
    public actualDb: MerkleTreeOperations,
    public prover: CircuitProver,
    public orchestrator: ProvingOrchestrator,
    public blockNumber: number,
    public directoriesToCleanup: string[],
    public logger: DebugLogger,
  ) {}

  static async new(
    logger: DebugLogger,
    createProver: (bbConfig: BBProverConfig) => Promise<CircuitProver> = _ =>
      Promise.resolve(new TestCircuitProver(new WASMSimulator())),
    blockNumber = 3,
  ) {
    const globalVariables = makeGlobals(blockNumber);

    const publicExecutor = mock<PublicExecutor>();
    const publicContractsDB = mock<ContractsDataSourcePublicDB>();
    const publicWorldStateDB = mock<WorldStatePublicDB>();
    const publicKernel = new RealPublicKernelCircuitSimulator(new WASMSimulator());
    const actualDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    const processor = new PublicProcessor(
      actualDb,
      publicExecutor,
      publicKernel,
      GlobalVariables.empty(),
      Header.empty(),
      publicContractsDB,
      publicWorldStateDB,
    );

    let localProver: CircuitProver;
    const config = await getEnvironmentConfig(logger);
    const simulationProvider = await getSimulationProvider({
      acvmWorkingDirectory: config?.acvmWorkingDirectory,
      acvmBinaryPath: config?.expectedAcvmPath,
    });
    if (!config) {
      localProver = new TestCircuitProver(simulationProvider);
    } else {
      const bbConfig: BBProverConfig = {
        acvmBinaryPath: config.expectedAcvmPath,
        acvmWorkingDirectory: config.acvmWorkingDirectory,
        bbBinaryPath: config.expectedBBPath,
        bbWorkingDirectory: config.bbWorkingDirectory,
      };
      localProver = await createProver(bbConfig);
    }

    const orchestrator = await ProvingOrchestrator.new(actualDb, localProver);

    return new this(
      publicExecutor,
      publicContractsDB,
      publicWorldStateDB,
      processor,
      simulationProvider,
      globalVariables,
      actualDb,
      localProver,
      orchestrator,
      blockNumber,
      [config?.directoryToCleanup ?? ''],
      logger,
    );
  }

  async cleanup() {
    await this.orchestrator.stop();
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
    const defaultExecutorImplementation = (execution: PublicExecution, _1: GlobalVariables, _2?: number) => {
      for (const tx of txs) {
        for (const request of tx.enqueuedPublicFunctionCalls) {
          if (execution.contractAddress.equals(request.contractAddress)) {
            const result = PublicExecutionResultBuilder.fromPublicCallRequest({ request }).build();
            // result.unencryptedLogs = tx.unencryptedLogs.functionLogs[0];
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
      execution: PublicExecution,
      globalVariables: GlobalVariables,
      sideEffectCounter?: number,
    ) => Promise<PublicExecutionResult>,
  ) {
    if (executorMock) {
      this.publicExecutor.simulate.mockImplementation(executorMock);
    }
    return await this.publicProcessor.process(txs, maxTransactions, blockProver, txValidator);
  }
}
