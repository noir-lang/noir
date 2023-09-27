import { GlobalVariables, HistoricBlockData } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

import { Oracle, acvm, extractCallStack, extractPublicCircuitPublicInputs } from '../acvm/index.js';
import { ExecutionError, createSimulationError } from '../common/errors.js';
import { SideEffectCounter } from '../common/index.js';
import { PackedArgsCache } from '../common/packed_args_cache.js';
import { AcirSimulator } from '../index.js';
import { CommitmentsDB, PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution, PublicExecutionResult } from './execution.js';
import { PublicExecutionContext } from './public_execution_context.js';

/**
 * Execute a public function and return the execution result.
 */
export async function executePublicFunction(
  context: PublicExecutionContext,
  acir: Buffer,
  log = createDebugLogger('aztec:simulator:public_execution'),
): Promise<PublicExecutionResult> {
  const execution = context.execution;
  const { contractAddress, functionData } = execution;
  const selector = functionData.selector;
  log(`Executing public external function ${contractAddress.toString()}:${selector}`);

  const initialWitness = context.getInitialWitness();
  const acvmCallback = new Oracle(context);
  const { partialWitness } = await acvm(await AcirSimulator.getSolver(), acir, initialWitness, acvmCallback).catch(
    (err: Error) => {
      throw new ExecutionError(
        err.message,
        {
          contractAddress,
          functionSelector: selector,
        },
        extractCallStack(err),
        { cause: err },
      );
    },
  );

  const {
    returnValues,
    newL2ToL1Msgs,
    newCommitments: newCommitmentsPadded,
    newNullifiers: newNullifiersPadded,
  } = extractPublicCircuitPublicInputs(partialWitness, acir);

  const newL2ToL1Messages = newL2ToL1Msgs.filter(v => !v.isZero());
  const newCommitments = newCommitmentsPadded.filter(v => !v.isZero());
  const newNullifiers = newNullifiersPadded.filter(v => !v.isZero());

  const { contractStorageReads, contractStorageUpdateRequests } = context.getStorageActionData();
  log(
    `Contract storage reads: ${contractStorageReads
      .map(r => r.toFriendlyJSON() + ` - sec: ${r.sideEffectCounter}`)
      .join(', ')}`,
  );

  const nestedExecutions = context.getNestedExecutions();
  const unencryptedLogs = context.getUnencryptedLogs();

  return {
    execution,
    newCommitments,
    newL2ToL1Messages,
    newNullifiers,
    contractStorageReads,
    contractStorageUpdateRequests,
    returnValues,
    nestedExecutions,
    unencryptedLogs,
  };
}

/**
 * Handles execution of public functions.
 */
export class PublicExecutor {
  constructor(
    private readonly stateDb: PublicStateDB,
    private readonly contractsDb: PublicContractsDB,
    private readonly commitmentsDb: CommitmentsDB,
    private readonly blockData: HistoricBlockData,
  ) {}

  /**
   * Executes a public execution request.
   * @param execution - The execution to run.
   * @param globalVariables - The global variables to use.
   * @returns The result of the run plus all nested runs.
   */
  public async simulate(execution: PublicExecution, globalVariables: GlobalVariables): Promise<PublicExecutionResult> {
    const selector = execution.functionData.selector;
    const acir = await this.contractsDb.getBytecode(execution.contractAddress, selector);
    if (!acir) throw new Error(`Bytecode not found for ${execution.contractAddress}:${selector}`);

    // Functions can request to pack arguments before calling other functions.
    // We use this cache to hold the packed arguments.
    const packedArgs = await PackedArgsCache.create([]);

    const sideEffectCounter = new SideEffectCounter();

    const context = new PublicExecutionContext(
      execution,
      this.blockData,
      globalVariables,
      packedArgs,
      sideEffectCounter,
      this.stateDb,
      this.contractsDb,
      this.commitmentsDb,
    );

    try {
      return await executePublicFunction(context, acir);
    } catch (err) {
      throw createSimulationError(err instanceof Error ? err : new Error('Unknown error during public execution'));
    }
  }
}
