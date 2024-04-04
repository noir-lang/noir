import { UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import { Fr, type GlobalVariables, type Header, PublicCircuitPublicInputs } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

import { spawn } from 'child_process';
import fs from 'fs/promises';
import path from 'path';

import { Oracle, acvm, extractCallStack, extractReturnWitness } from '../acvm/index.js';
import { AvmContext } from '../avm/avm_context.js';
import { AvmMachineState } from '../avm/avm_machine_state.js';
import { AvmSimulator } from '../avm/avm_simulator.js';
import { HostStorage } from '../avm/journal/host_storage.js';
import { AvmPersistableStateManager } from '../avm/journal/index.js';
import {
  isAvmBytecode,
  temporaryConvertAvmResults,
  temporaryCreateAvmExecutionEnvironment,
} from '../avm/temporary_executor_migration.js';
import { AcirSimulator } from '../client/simulator.js';
import { ExecutionError, createSimulationError } from '../common/errors.js';
import { SideEffectCounter } from '../common/index.js';
import { PackedArgsCache } from '../common/packed_args_cache.js';
import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from './db.js';
import { type PublicExecution, type PublicExecutionResult, checkValidStaticCall } from './execution.js';
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
      // TODO (side effects) get these values in the revert case from the vm
      startSideEffectCounter: Fr.ZERO,
      endSideEffectCounter: Fr.ZERO,
      newNullifiers: [],
      nullifierReadRequests: [],
      nullifierNonExistentReadRequests: [],
      contractStorageReads: [],
      contractStorageUpdateRequests: [],
      nestedExecutions: [],
      unencryptedLogs: UnencryptedFunctionL2Logs.empty(),
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
    nullifierNonExistentReadRequests: nullifierNonExistentReadRequestsPadded,
    newL2ToL1Msgs,
    newNoteHashes: newNoteHashesPadded,
    newNullifiers: newNullifiersPadded,
    startSideEffectCounter,
    endSideEffectCounter,
  } = PublicCircuitPublicInputs.fromFields(returnWitness);

  const nullifierReadRequests = nullifierReadRequestsPadded.filter(v => !v.isEmpty());
  const nullifierNonExistentReadRequests = nullifierNonExistentReadRequestsPadded.filter(v => !v.isEmpty());
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
    startSideEffectCounter,
    endSideEffectCounter,
    nullifierReadRequests,
    nullifierNonExistentReadRequests,
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

  private readonly log = createDebugLogger('aztec:simulator:public_executor');
  /**
   * Executes a public execution request.
   * @param execution - The execution to run.
   * @param globalVariables - The global variables to use.
   * @returns The result of the run plus all nested runs.
   */
  public async simulate(
    execution: PublicExecution,
    globalVariables: GlobalVariables,
    sideEffectCounter: number = 0,
  ): Promise<PublicExecutionResult> {
    const selector = execution.functionData.selector;
    const bytecode = await this.contractsDb.getBytecode(execution.contractAddress, selector);
    if (!bytecode) {
      throw new Error(`Bytecode not found for ${execution.contractAddress}:${selector}`);
    }

    if (isAvmBytecode(bytecode)) {
      return await this.simulateAvm(execution, globalVariables, sideEffectCounter);
    } else {
      return await this.simulateAcvm(execution, globalVariables, sideEffectCounter);
    }
  }

  /**
   * Executes a public execution request with the ACVM.
   * @param execution - The execution to run.
   * @param globalVariables - The global variables to use.
   * @returns The result of the run plus all nested runs.
   */
  private async simulateAcvm(
    execution: PublicExecution,
    globalVariables: GlobalVariables,
    sideEffectCounter: number = 0,
  ): Promise<PublicExecutionResult> {
    const selector = execution.functionData.selector;
    const acir = await this.contractsDb.getBytecode(execution.contractAddress, selector);
    if (!acir) {
      throw new Error(`Bytecode not found for ${execution.contractAddress}:${selector}`);
    }

    // Functions can request to pack arguments before calling other functions.
    // We use this cache to hold the packed arguments.
    const packedArgs = PackedArgsCache.create([]);

    const context = new PublicExecutionContext(
      execution,
      this.header,
      globalVariables,
      packedArgs,
      new SideEffectCounter(sideEffectCounter),
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
   * Executes a public execution request in the AVM.
   * @param execution - The execution to run.
   * @param globalVariables - The global variables to use.
   * @returns The result of the run plus all nested runs.
   */
  private async simulateAvm(
    execution: PublicExecution,
    globalVariables: GlobalVariables,
    _sideEffectCounter = 0,
  ): Promise<PublicExecutionResult> {
    // Temporary code to construct the AVM context
    // These data structures will permeate across the simulator when the public executor is phased out
    const hostStorage = new HostStorage(this.stateDb, this.contractsDb, this.commitmentsDb);
    const worldStateJournal = new AvmPersistableStateManager(hostStorage);
    const executionEnv = temporaryCreateAvmExecutionEnvironment(execution, globalVariables);
    // TODO(@spalladino) Load initial gas from the public execution request
    const machineState = new AvmMachineState(1e10, 1e10, 1e10);

    const context = new AvmContext(worldStateJournal, executionEnv, machineState);
    const simulator = new AvmSimulator(context);

    const result = await simulator.execute();
    const newWorldState = context.persistableState.flush();

    // TODO(@spalladino) Read gas left from machineState and return it
    return temporaryConvertAvmResults(execution, newWorldState, result);
  }

  /**
   * These functions are currently housed in the temporary executor as it relies on access to
   * oracles like the contractsDB and this is the least intrusive way to achieve this.
   * When we remove this executor(tracking issue #4792) and have an interface that is compatible with the kernel circuits,
   * this will be moved to sequencer-client/prover.
   */

  /**
   * Generates a proof for an associated avm execution. This is currently only used for testing purposes,
   * as proof generation is not fully complete in the AVM yet.
   * @param execution - The execution to run.
   * @returns An AVM proof and the verification key.
   */
  public async getAvmProof(avmExecution: PublicExecution): Promise<Buffer[]> {
    // The paths for the barretenberg binary and the write path are hardcoded for now.
    const bbPath = path.resolve('../../barretenberg/cpp');
    const artifactsPath = path.resolve('target');

    // Create the directory if it does not exist
    await fs.rm(artifactsPath, { recursive: true, force: true });
    await fs.mkdir(artifactsPath, { recursive: true });

    const calldataPath = path.join(artifactsPath, 'calldata.bin');
    const bytecodePath = path.join(artifactsPath, 'avm_bytecode.bin');
    const proofPath = path.join(artifactsPath, 'proof');

    const { args, functionData, contractAddress } = avmExecution;
    const bytecode = await this.contractsDb.getBytecode(contractAddress, functionData.selector);
    // Write call data and bytecode to files.
    await fs.writeFile(
      calldataPath,
      args.map(c => c.toBuffer()),
    );
    await fs.writeFile(bytecodePath, bytecode!);

    const bbExec = path.join(bbPath, 'build', 'bin', 'bb');
    const bbArgs = ['avm_prove', '-b', bytecodePath, '-d', calldataPath, '-o', proofPath];
    this.log(`calling '${bbExec} ${bbArgs.join(' ')}'`);
    const bbBinary = spawn(bbExec, bbArgs);

    // The binary writes the proof and the verification key to the write path.
    return new Promise((resolve, reject) => {
      let stdout: string = '';
      let stderr: string = '';

      bbBinary.on('close', () => {
        this.log(`Proof generation complete. Reading proof and vk from ${proofPath}.`);
        return resolve(Promise.all([fs.readFile(proofPath), fs.readFile(path.join(artifactsPath, 'vk'))]));
      });

      // Catch stdout.
      bbBinary.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });
      bbBinary.stdout.on('end', () => {
        if (stdout.length > 0) {
          this.log(`stdout: ${stdout}`);
        }
      });

      // Catch stderr.
      bbBinary.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });
      bbBinary.stderr.on('end', () => {
        if (stderr.length > 0) {
          this.log(`stderr: ${stderr}`);
        }
      });

      // Catch and propagate errors from spawning
      bbBinary.on('error', err => {
        reject(err);
      });
    });
  }

  /**
   * Verifies an AVM proof. This function is currently only used for testing purposes, as verification
   * is not fully complete in the AVM yet.
   * @param vk - The verification key to use.
   * @param proof - The proof to verify.
   * @returns True if the proof is valid, false otherwise.
   */
  async verifyAvmProof(vk: Buffer, proof: Buffer): Promise<boolean> {
    // The relative paths for the barretenberg binary and the write path are hardcoded for now.
    const bbPath = path.resolve('../../barretenberg/cpp');
    const artifactsPath = path.resolve('./target');

    const vkPath = path.join(artifactsPath, 'vk');
    const proofPath = path.join(artifactsPath, 'proof');

    // Write the verification key and the proof to files.
    await fs.writeFile(vkPath, vk);
    await fs.writeFile(proofPath, proof);

    const bbExec = path.join(bbPath, 'build', 'bin', 'bb');
    const bbArgs = ['avm_verify', '-p', proofPath];
    this.log(`calling '${bbPath} ${bbArgs.join(' ')}'`);
    const bbBinary = spawn(bbExec, bbArgs);

    // The binary prints to stdout 1 if the proof is valid and 0 if it is not.
    return new Promise((resolve, reject) => {
      let result = Buffer.alloc(0);
      bbBinary.stdout.on('data', data => {
        result += data;
      });
      bbBinary.on('close', () => {
        resolve(result.toString() === '1');
      });
      // Catch and propagate errors from spawning
      bbBinary.on('error', err => {
        reject(err);
      });
    });
  }
}
