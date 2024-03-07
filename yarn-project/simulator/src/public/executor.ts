import { FunctionL2Logs } from '@aztec/circuit-types';
import { GlobalVariables, Header, PublicCircuitPublicInputs } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

import { Oracle, acvm, extractCallStack, extractReturnWitness } from '../acvm/index.js';
import { AvmContext } from '../avm/avm_context.js';
import { AvmMachineState } from '../avm/avm_machine_state.js';
import { AvmSimulator } from '../avm/avm_simulator.js';
import { HostStorage } from '../avm/journal/host_storage.js';
import { AvmPersistableStateManager } from '../avm/journal/index.js';
import {
  temporaryConvertAvmResults,
  temporaryCreateAvmExecutionEnvironment,
} from '../avm/temporary_executor_migration.js';
import { AcirSimulator } from '../client/simulator.js';
import { ExecutionError, createSimulationError } from '../common/errors.js';
import { SideEffectCounter } from '../common/index.js';
import { PackedArgsCache } from '../common/packed_args_cache.js';
import { CommitmentsDB, PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution, PublicExecutionResult, checkValidStaticCall } from './execution.js';
import { PublicExecutionContext } from './public_execution_context.js';

/**
 * Execute a public function and return the execution result.
 */
export async function executePublicFunction(
  context: PublicExecutionContext,
  acir: Buffer,
  nested: boolean,
  log = createDebugLogger('aztec:simulator:public_execution'),
): Promise<PublicExecutionResult> {
  const execution = context.execution;
  const { contractAddress, functionData } = execution;
  const selector = functionData.selector;
  log(`Executing public external function ${contractAddress.toString()}:${selector}`);

  const initialWitness = context.getInitialWitness();
  const acvmCallback = new Oracle(context);
  const { partialWitness, reverted, revertReason } = await acvm(
    await AcirSimulator.getSolver(),
    acir,
    initialWitness,
    acvmCallback,
  )
    .then(result => ({
      partialWitness: result.partialWitness,
      reverted: false,
      revertReason: undefined,
    }))
    .catch((err: Error) => {
      const ee = new ExecutionError(
        err.message,
        {
          contractAddress,
          functionSelector: selector,
        },
        extractCallStack(err),
        { cause: err },
      );

      if (nested) {
        // If we're nested, throw the error so the parent can handle it
        throw ee;
      } else {
        return {
          partialWitness: undefined,
          reverted: true,
          revertReason: createSimulationError(ee),
        };
      }
    });
  if (reverted) {
    if (!revertReason) {
      throw new Error('Reverted but no revert reason');
    }

    return {
      execution,
      returnValues: [],
      newNoteHashes: [],
      newL2ToL1Messages: [],
      newNullifiers: [],
      nullifierReadRequests: [],
      contractStorageReads: [],
      contractStorageUpdateRequests: [],
      nestedExecutions: [],
      unencryptedLogs: FunctionL2Logs.empty(),
      reverted,
      revertReason,
    };
  }

  if (!partialWitness) {
    throw new Error('No partial witness returned from ACVM');
  }

  const returnWitness = extractReturnWitness(acir, partialWitness);
  const {
    returnValues,
    nullifierReadRequests: nullifierReadRequestsPadded,
    newL2ToL1Msgs,
    newNoteHashes: newNoteHashesPadded,
    newNullifiers: newNullifiersPadded,
  } = PublicCircuitPublicInputs.fromFields(returnWitness);

  const nullifierReadRequests = nullifierReadRequestsPadded.filter(v => !v.isEmpty());
  const newL2ToL1Messages = newL2ToL1Msgs.filter(v => !v.isEmpty());
  const newNoteHashes = newNoteHashesPadded.filter(v => !v.isEmpty());
  const newNullifiers = newNullifiersPadded.filter(v => !v.isEmpty());

  const { contractStorageReads, contractStorageUpdateRequests } = context.getStorageActionData();

  log(
    `Contract storage reads: ${contractStorageReads
      .map(r => r.toFriendlyJSON() + ` - sec: ${r.sideEffectCounter}`)
      .join(', ')}`,
  );
  log(
    `Contract storage update requests: ${contractStorageUpdateRequests
      .map(r => r.toFriendlyJSON() + ` - sec: ${r.sideEffectCounter}`)
      .join(', ')}`,
  );

  const nestedExecutions = context.getNestedExecutions();
  const unencryptedLogs = context.getUnencryptedLogs();

  return {
    execution,
    newNoteHashes,
    newL2ToL1Messages,
    newNullifiers,
    nullifierReadRequests,
    contractStorageReads,
    contractStorageUpdateRequests,
    returnValues,
    nestedExecutions,
    unencryptedLogs,
    reverted: false,
    revertReason: undefined,
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
    private readonly header: Header,
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
    if (!acir) {
      throw new Error(`Bytecode not found for ${execution.contractAddress}:${selector}`);
    }

    // Functions can request to pack arguments before calling other functions.
    // We use this cache to hold the packed arguments.
    const packedArgs = PackedArgsCache.create([]);

    const sideEffectCounter = new SideEffectCounter();

    const context = new PublicExecutionContext(
      execution,
      this.header,
      globalVariables,
      packedArgs,
      sideEffectCounter,
      this.stateDb,
      this.contractsDb,
      this.commitmentsDb,
    );

    const executionResult = await executePublicFunction(context, acir, false /** nested */);

    if (executionResult.execution.callContext.isStaticCall) {
      checkValidStaticCall(
        executionResult.newNoteHashes,
        executionResult.newNullifiers,
        executionResult.contractStorageUpdateRequests,
        executionResult.newL2ToL1Messages,
        executionResult.unencryptedLogs,
      );
    }

    return executionResult;
  }

  /**
   * Executes a public execution request in the avm.
   * @param execution - The execution to run.
   * @param globalVariables - The global variables to use.
   * @returns The result of the run plus all nested runs.
   */
  public async simulateAvm(
    execution: PublicExecution,
    globalVariables: GlobalVariables,
  ): Promise<PublicExecutionResult> {
    // Temporary code to construct the AVM context
    // These data structures will permiate across the simulator when the public executor is phased out
    const hostStorage = new HostStorage(this.stateDb, this.contractsDb, this.commitmentsDb);
    const worldStateJournal = new AvmPersistableStateManager(hostStorage);
    const executionEnv = temporaryCreateAvmExecutionEnvironment(execution, globalVariables);
    const machineState = new AvmMachineState(0, 0, 0);

    const context = new AvmContext(worldStateJournal, executionEnv, machineState);
    const simulator = new AvmSimulator(context);

    const result = await simulator.execute();
    const newWorldState = context.persistableState.flush();
    return temporaryConvertAvmResults(execution, newWorldState, result);
  }
}
