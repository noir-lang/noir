import {
  AztecAddress,
  CallContext,
  EthAddress,
  Fr,
  FunctionData,
  FunctionSelector,
  GlobalVariables,
  HistoricBlockData,
  RETURN_VALUES_LENGTH,
} from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { FunctionL2Logs } from '@aztec/types';

import {
  ZERO_ACVM_FIELD,
  acvm,
  convertACVMFieldToBuffer,
  extractCallStack,
  extractPublicCircuitPublicInputs,
  frToAztecAddress,
  fromACVMField,
  toACVMField,
  toACVMWitness,
  toAcvmCommitmentLoadOracleInputs,
  toAcvmL1ToL2MessageLoadOracleInputs,
} from '../acvm/index.js';
import { oracleDebugCallToFormattedStr } from '../client/debug.js';
import { ExecutionError, createSimulationError } from '../common/errors.js';
import { PackedArgsCache } from '../common/packed_args_cache.js';
import { AcirSimulator } from '../index.js';
import { CommitmentsDB, PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution, PublicExecutionResult } from './execution.js';
import { ContractStorageActionsCollector } from './state_actions.js';

/**
 * Handles execution of public functions.
 */
export class PublicExecutor {
  constructor(
    private readonly stateDb: PublicStateDB,
    private readonly contractsDb: PublicContractsDB,
    private readonly commitmentsDb: CommitmentsDB,
    private readonly blockData: HistoricBlockData,
    private sideEffectCounter: number = 0,
    private log = createDebugLogger('aztec:simulator:public-executor'),
  ) {}

  /**
   * Executes a public execution request.
   * @param execution - The execution to run.
   * @param globalVariables - The global variables to use.
   * @returns The result of the run plus all nested runs.
   */
  public async simulate(execution: PublicExecution, globalVariables: GlobalVariables): Promise<PublicExecutionResult> {
    try {
      return await this.execute(execution, globalVariables);
    } catch (err) {
      throw createSimulationError(err instanceof Error ? err : new Error('Unknown error during public execution'));
    }
  }

  private async execute(execution: PublicExecution, globalVariables: GlobalVariables): Promise<PublicExecutionResult> {
    const selector = execution.functionData.selector;
    this.log(`Executing public external function ${execution.contractAddress.toString()}:${selector}`);

    const acir = await this.contractsDb.getBytecode(execution.contractAddress, selector);
    if (!acir) throw new Error(`Bytecode not found for ${execution.contractAddress.toString()}:${selector}`);

    const initialWitness = getInitialWitness(execution.args, execution.callContext, this.blockData, globalVariables);
    const storageActions = new ContractStorageActionsCollector(this.stateDb, execution.contractAddress);
    const nestedExecutions: PublicExecutionResult[] = [];
    const unencryptedLogs = new FunctionL2Logs([]);
    // Functions can request to pack arguments before calling other functions.
    // We use this cache to hold the packed arguments.
    const packedArgs = await PackedArgsCache.create([]);
    const { partialWitness } = await acvm(await AcirSimulator.getSolver(), acir, initialWitness, {
      computeSelector: (...args) => {
        const signature = oracleDebugCallToFormattedStr(args);
        return Promise.resolve(toACVMField(FunctionSelector.fromSignature(signature).toField()));
      },
      packArguments: async args => {
        return toACVMField(await packedArgs.pack(args.map(fromACVMField)));
      },
      debugLog: (...args) => {
        this.log(oracleDebugCallToFormattedStr(args));
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      getL1ToL2Message: async ([msgKey]) => {
        const messageInputs = await this.commitmentsDb.getL1ToL2Message(fromACVMField(msgKey));
        return toAcvmL1ToL2MessageLoadOracleInputs(messageInputs, this.blockData.l1ToL2MessagesTreeRoot);
      }, // l1 to l2 messages in public contexts TODO: https://github.com/AztecProtocol/aztec-packages/issues/616
      getCommitment: async ([commitment]) => {
        const commitmentInputs = await this.commitmentsDb.getCommitmentOracle(
          execution.contractAddress,
          fromACVMField(commitment),
        );
        return toAcvmCommitmentLoadOracleInputs(commitmentInputs, this.blockData.privateDataTreeRoot);
      },
      storageRead: async ([slot], [numberOfElements]) => {
        const startStorageSlot = fromACVMField(slot);
        const values = [];
        for (let i = 0; i < Number(numberOfElements); i++) {
          const storageSlot = new Fr(startStorageSlot.value + BigInt(i));
          const value = await storageActions.read(storageSlot, this.sideEffectCounter++); // update the sideEffectCounter after assigning its current value to storage action
          this.log(`Oracle storage read: slot=${storageSlot.toString()} value=${value.toString()}`);
          values.push(value);
        }
        return values.map(v => toACVMField(v));
      },
      storageWrite: async ([slot], values) => {
        const startStorageSlot = fromACVMField(slot);
        const newValues = [];
        for (let i = 0; i < values.length; i++) {
          const storageSlot = new Fr(startStorageSlot.value + BigInt(i));
          const newValue = fromACVMField(values[i]);
          await storageActions.write(storageSlot, newValue, this.sideEffectCounter++); // update the sideEffectCounter after assigning its current value to storage action
          await this.stateDb.storageWrite(execution.contractAddress, storageSlot, newValue);
          this.log(`Oracle storage write: slot=${storageSlot.toString()} value=${newValue.toString()}`);
          newValues.push(newValue);
        }
        return newValues.map(v => toACVMField(v));
      },
      callPublicFunction: async ([address], [functionSelector], [argsHash]) => {
        const args = packedArgs.unpack(fromACVMField(argsHash));
        this.log(`Public function call: addr=${address} selector=${functionSelector} args=${args.join(',')}`);
        const childExecutionResult = await this.callPublicFunction(
          frToAztecAddress(fromACVMField(address)),
          FunctionSelector.fromField(fromACVMField(functionSelector)),
          args,
          execution.callContext,
          globalVariables,
        );

        nestedExecutions.push(childExecutionResult);
        this.log(`Returning from nested call: ret=${childExecutionResult.returnValues.join(', ')}`);
        return padArrayEnd(childExecutionResult.returnValues, Fr.ZERO, RETURN_VALUES_LENGTH).map(toACVMField);
      },
      emitUnencryptedLog: args => {
        // https://github.com/AztecProtocol/aztec-packages/issues/885
        const log = Buffer.concat(args.map((charBuffer: any) => convertACVMFieldToBuffer(charBuffer).subarray(-1)));
        unencryptedLogs.logs.push(log);
        this.log(`Emitted unencrypted log: "${log.toString('ascii')}"`);
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      getPortalContractAddress: async ([aztecAddress]) => {
        const contractAddress = AztecAddress.fromString(aztecAddress);
        const portalContactAddress =
          (await this.contractsDb.getPortalContractAddress(contractAddress)) ?? EthAddress.ZERO;
        return Promise.resolve(toACVMField(portalContactAddress));
      },
    }).catch((err: Error) => {
      throw new ExecutionError(
        err.message,
        {
          contractAddress: execution.contractAddress,
          functionSelector: selector,
        },
        extractCallStack(err),
        { cause: err },
      );
    });

    const {
      returnValues,
      newL2ToL1Msgs,
      newCommitments: newCommitmentsPadded,
      newNullifiers: newNullifiersPadded,
    } = extractPublicCircuitPublicInputs(partialWitness, acir);

    const newL2ToL1Messages = newL2ToL1Msgs.filter(v => !v.isZero());
    const newCommitments = newCommitmentsPadded.filter(v => !v.isZero());
    const newNullifiers = newNullifiersPadded.filter(v => !v.isZero());

    const [contractStorageReads, contractStorageUpdateRequests] = storageActions.collect();
    this.log(
      `Contract storage reads: ${contractStorageReads
        .map(r => r.toFriendlyJSON() + ` - sec: ${r.sideEffectCounter}`)
        .join(', ')}`,
    );

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

  private async callPublicFunction(
    targetContractAddress: AztecAddress,
    targetFunctionSelector: FunctionSelector,
    targetArgs: Fr[],
    callerContext: CallContext,
    globalVariables: GlobalVariables,
  ) {
    const portalAddress = (await this.contractsDb.getPortalContractAddress(targetContractAddress)) ?? EthAddress.ZERO;
    const isInternal = await this.contractsDb.getIsInternal(targetContractAddress, targetFunctionSelector);
    if (isInternal === undefined) {
      throw new Error(
        `ERR: ContractsDb don't contain isInternal for ${targetContractAddress.toString()}:${targetFunctionSelector.toString()}. Defaulting to false.`,
      );
    }

    const functionData = new FunctionData(targetFunctionSelector, isInternal, false, false);

    const callContext = CallContext.from({
      msgSender: callerContext.storageContractAddress,
      portalContractAddress: portalAddress,
      storageContractAddress: targetContractAddress,
      isContractDeployment: false,
      isDelegateCall: false,
      isStaticCall: false,
    });

    const nestedExecution: PublicExecution = {
      args: targetArgs,
      contractAddress: targetContractAddress,
      functionData,
      callContext,
    };

    return this.execute(nestedExecution, globalVariables);
  }
}

/**
 * Generates the initial witness for a public function.
 * @param args - The arguments to the function.
 * @param callContext - The call context of the function.
 * @param historicBlockData - Historic Trees roots and data required to reconstruct block hash.
 * @param globalVariables - The global variables.
 * @param witnessStartIndex - The index where to start inserting the parameters.
 * @returns The initial witness.
 */
function getInitialWitness(
  args: Fr[],
  callContext: CallContext,
  historicBlockData: HistoricBlockData,
  globalVariables: GlobalVariables,
  witnessStartIndex = 1,
) {
  return toACVMWitness(witnessStartIndex, [
    callContext.msgSender,
    callContext.storageContractAddress,
    callContext.portalContractAddress,
    callContext.isDelegateCall,
    callContext.isStaticCall,
    callContext.isContractDeployment,

    ...historicBlockData.toArray(),

    globalVariables.chainId,
    globalVariables.version,
    globalVariables.blockNumber,
    globalVariables.timestamp,

    ...args,
  ]);
}
