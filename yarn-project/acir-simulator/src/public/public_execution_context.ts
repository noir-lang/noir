import { CallContext, FunctionData, FunctionSelector, GlobalVariables, HistoricBlockData } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { FunctionL2Logs, UnencryptedL2Log } from '@aztec/types';

import {
  TypedOracle,
  toACVMCallContext,
  toACVMGlobalVariables,
  toACVMHistoricBlockData,
  toACVMWitness,
} from '../acvm/index.js';
import { PackedArgsCache, SideEffectCounter } from '../common/index.js';
import { CommitmentsDB, PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution, PublicExecutionResult } from './execution.js';
import { executePublicFunction } from './executor.js';
import { ContractStorageActionsCollector } from './state_actions.js';

/**
 * The execution context for a public tx simulation.
 */
export class PublicExecutionContext extends TypedOracle {
  private storageActions: ContractStorageActionsCollector;
  private nestedExecutions: PublicExecutionResult[] = [];
  private unencryptedLogs: UnencryptedL2Log[] = [];

  constructor(
    /**
     * Data for this execution.
     */
    public readonly execution: PublicExecution,
    private readonly historicBlockData: HistoricBlockData,
    private readonly globalVariables: GlobalVariables,
    private readonly packedArgsCache: PackedArgsCache,
    private readonly sideEffectCounter: SideEffectCounter,
    private readonly stateDb: PublicStateDB,
    private readonly contractsDb: PublicContractsDB,
    private readonly commitmentsDb: CommitmentsDB,
    private log = createDebugLogger('aztec:simulator:public_execution_context'),
  ) {
    super();
    this.storageActions = new ContractStorageActionsCollector(stateDb, execution.contractAddress);
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
  public getInitialWitness(witnessStartIndex = 1) {
    const { callContext, args } = this.execution;
    const fields = [
      ...toACVMCallContext(callContext),
      ...toACVMHistoricBlockData(this.historicBlockData),
      ...toACVMGlobalVariables(this.globalVariables),

      ...args,
    ];

    return toACVMWitness(witnessStartIndex, fields);
  }

  /**
   * Return the nested execution results during this execution.
   */
  public getNestedExecutions() {
    return this.nestedExecutions;
  }

  /**
   * Return the encrypted logs emitted during this execution.
   */
  public getUnencryptedLogs() {
    return new FunctionL2Logs(this.unencryptedLogs.map(log => log.toBuffer()));
  }

  /**
   * Return the data read and updated during this execution.
   */
  public getStorageActionData() {
    const [contractStorageReads, contractStorageUpdateRequests] = this.storageActions.collect();
    return { contractStorageReads, contractStorageUpdateRequests };
  }

  /**
   * Pack the given arguments.
   * @param args - Arguments to pack
   */
  public packArguments(args: Fr[]): Promise<Fr> {
    return Promise.resolve(this.packedArgsCache.pack(args));
  }

  /**
   * Fetches the a message from the db, given its key.
   * @param msgKey - A buffer representing the message key.
   * @returns The l1 to l2 message data
   */
  public async getL1ToL2Message(msgKey: Fr) {
    // l1 to l2 messages in public contexts TODO: https://github.com/AztecProtocol/aztec-packages/issues/616
    const message = await this.commitmentsDb.getL1ToL2Message(msgKey);
    return { ...message, root: this.historicBlockData.l1ToL2MessagesTreeRoot };
  }

  /**
   * Emit an unencrypted log.
   * @param log - The unencrypted log to be emitted.
   */
  public emitUnencryptedLog(log: UnencryptedL2Log) {
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/885)
    this.unencryptedLogs.push(log);
    this.log(`Emitted unencrypted log: "${log.toHumanReadable()}"`);
  }

  /**
   * Retrieves the portal contract address associated with the given contract address.
   * Returns zero address if the input contract address is not found or invalid.
   * @param contractAddress - The address of the contract whose portal address is to be fetched.
   * @returns The portal contract address.
   */
  public async getPortalContractAddress(contractAddress: AztecAddress) {
    return (await this.contractsDb.getPortalContractAddress(contractAddress)) ?? EthAddress.ZERO;
  }

  /**
   * Read the public storage data.
   * @param startStorageSlot - The starting storage slot.
   * @param numberOfElements - Number of elements to read from the starting storage slot.
   */
  public async storageRead(startStorageSlot: Fr, numberOfElements: number) {
    const values = [];
    for (let i = 0; i < Number(numberOfElements); i++) {
      const storageSlot = new Fr(startStorageSlot.value + BigInt(i));
      const sideEffectCounter = this.sideEffectCounter.count();
      const value = await this.storageActions.read(storageSlot, sideEffectCounter);
      this.log(`Oracle storage read: slot=${storageSlot.toString()} value=${value.toString()}`);
      values.push(value);
    }
    return values;
  }

  /**
   * Write some values to the public storage.
   * @param startStorageSlot - The starting storage slot.
   * @param values - The values to be written.
   */
  public async storageWrite(startStorageSlot: Fr, values: Fr[]) {
    const newValues = [];
    for (let i = 0; i < values.length; i++) {
      const storageSlot = new Fr(startStorageSlot.value + BigInt(i));
      const newValue = values[i];
      const sideEffectCounter = this.sideEffectCounter.count();
      await this.storageActions.write(storageSlot, newValue, sideEffectCounter);
      await this.stateDb.storageWrite(this.execution.contractAddress, storageSlot, newValue);
      this.log(`Oracle storage write: slot=${storageSlot.toString()} value=${newValue.toString()}`);
      newValues.push(newValue);
    }
    return newValues;
  }

  /**
   * Calls a public function as a nested execution.
   * @param targetContractAddress - The address of the contract to call.
   * @param functionSelector - The function selector of the function to call.
   * @param argsHash - The packed arguments to pass to the function.
   * @returns The return values of the public function.
   */
  public async callPublicFunction(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
  ) {
    const args = this.packedArgsCache.unpack(argsHash);
    this.log(`Public function call: addr=${targetContractAddress} selector=${functionSelector} args=${args.join(',')}`);

    const portalAddress = (await this.contractsDb.getPortalContractAddress(targetContractAddress)) ?? EthAddress.ZERO;
    const isInternal = await this.contractsDb.getIsInternal(targetContractAddress, functionSelector);
    if (isInternal === undefined) {
      throw new Error(`ERR: Method not found - ${targetContractAddress.toString()}:${functionSelector.toString()}`);
    }

    const acir = await this.contractsDb.getBytecode(targetContractAddress, functionSelector);
    if (!acir) {
      throw new Error(`Bytecode not found for ${targetContractAddress}:${functionSelector}`);
    }

    const functionData = new FunctionData(functionSelector, isInternal, false, false);

    const callContext = CallContext.from({
      msgSender: this.execution.contractAddress,
      portalContractAddress: portalAddress,
      storageContractAddress: targetContractAddress,
      functionSelector,
      isContractDeployment: false,
      isDelegateCall: false,
      isStaticCall: false,
    });

    const nestedExecution: PublicExecution = {
      args,
      contractAddress: targetContractAddress,
      functionData,
      callContext,
    };

    const context = new PublicExecutionContext(
      nestedExecution,
      this.historicBlockData,
      this.globalVariables,
      this.packedArgsCache,
      this.sideEffectCounter,
      this.stateDb,
      this.contractsDb,
      this.commitmentsDb,
      this.log,
    );

    const childExecutionResult = await executePublicFunction(context, acir);

    this.nestedExecutions.push(childExecutionResult);
    this.log(`Returning from nested call: ret=${childExecutionResult.returnValues.join(', ')}`);

    return childExecutionResult.returnValues;
  }
}
