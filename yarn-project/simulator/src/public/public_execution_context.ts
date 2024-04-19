import { UnencryptedFunctionL2Logs, type UnencryptedL2Log } from '@aztec/circuit-types';
import {
  CallContext,
  FunctionData,
  type FunctionSelector,
  type GlobalVariables,
  type Header,
} from '@aztec/circuits.js';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { type ContractInstance } from '@aztec/types/contracts';

import { TypedOracle, toACVMWitness } from '../acvm/index.js';
import { type PackedValuesCache, type SideEffectCounter } from '../common/index.js';
import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from './db.js';
import { type PublicExecution, type PublicExecutionResult, checkValidStaticCall } from './execution.js';
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
    public readonly header: Header,
    public readonly globalVariables: GlobalVariables,
    private readonly packedValuesCache: PackedValuesCache,
    private readonly sideEffectCounter: SideEffectCounter,
    public readonly stateDb: PublicStateDB,
    public readonly contractsDb: PublicContractsDB,
    public readonly commitmentsDb: CommitmentsDB,
    private log = createDebugLogger('aztec:simulator:public_execution_context'),
  ) {
    super();
    this.storageActions = new ContractStorageActionsCollector(stateDb, execution.callContext.storageContractAddress);
  }

  /**
   * Generates the initial witness for a public function.
   * @param args - The arguments to the function.
   * @param callContext - The call context of the function.
   * @param header - Contains data required to reconstruct a block hash (historical roots etc.).
   * @param globalVariables - The global variables.
   * @param witnessStartIndex - The index where to start inserting the parameters.
   * @returns The initial witness.
   */
  public getInitialWitness(witnessStartIndex = 0) {
    const { callContext, args } = this.execution;
    const fields = [
      ...callContext.toFields(),
      ...this.header.toFields(),
      ...this.globalVariables.toFields(),
      new Fr(this.sideEffectCounter.current()),
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
    return new UnencryptedFunctionL2Logs(this.unencryptedLogs);
  }

  /**
   * Return the data read and updated during this execution.
   */
  public getStorageActionData() {
    const [contractStorageReads, contractStorageUpdateRequests] = this.storageActions.collect();
    return { contractStorageReads, contractStorageUpdateRequests };
  }

  /**
   * Pack the given array of arguments.
   * @param args - Arguments to pack
   */
  public override packArgumentsArray(args: Fr[]): Promise<Fr> {
    return Promise.resolve(this.packedValuesCache.pack(args));
  }

  /**
   * Pack the given returns.
   * @param returns - Returns to pack
   */
  public override packReturns(returns: Fr[]): Promise<Fr> {
    return Promise.resolve(this.packedValuesCache.pack(returns));
  }

  /**
   * Unpack the given returns.
   * @param returnsHash - Returns hash to unpack
   */
  public override unpackReturns(returnsHash: Fr): Promise<Fr[]> {
    return Promise.resolve(this.packedValuesCache.unpack(returnsHash));
  }

  /**
   * Fetches a message from the db, given its key.
   * @param contractAddress - Address of a contract by which the message was emitted.
   * @param messageHash - Hash of the message.
   * @param secret - Secret used to compute a nullifier.
   * @dev Contract address and secret are only used to compute the nullifier to get non-nullified messages
   * @returns The l1 to l2 membership witness (index of message in the tree and sibling path).
   */
  public override async getL1ToL2MembershipWitness(contractAddress: AztecAddress, messageHash: Fr, secret: Fr) {
    return await this.commitmentsDb.getL1ToL2MembershipWitness(contractAddress, messageHash, secret);
  }

  /**
   * Emit an unencrypted log.
   * @param log - The unencrypted log to be emitted.
   */
  public override emitUnencryptedLog(log: UnencryptedL2Log) {
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/885)
    this.unencryptedLogs.push(log);
    this.log.verbose(`Emitted unencrypted log: "${log.toHumanReadable()}"`);
    return Fr.fromBuffer(log.hash());
  }

  /**
   * Read the public storage data.
   * @param startStorageSlot - The starting storage slot.
   * @param numberOfElements - Number of elements to read from the starting storage slot.
   */
  public override async storageRead(startStorageSlot: Fr, numberOfElements: number) {
    const values = [];
    for (let i = 0; i < Number(numberOfElements); i++) {
      const storageSlot = new Fr(startStorageSlot.value + BigInt(i));
      const sideEffectCounter = this.sideEffectCounter.count();
      const value = await this.storageActions.read(storageSlot, sideEffectCounter);
      this.log.debug(`Oracle storage read: slot=${storageSlot.toString()} value=${value.toString()}`);
      values.push(value);
    }
    return values;
  }

  /**
   * Write some values to the public storage.
   * @param startStorageSlot - The starting storage slot.
   * @param values - The values to be written.
   */
  public override async storageWrite(startStorageSlot: Fr, values: Fr[]) {
    const newValues = [];
    for (let i = 0; i < values.length; i++) {
      const storageSlot = new Fr(startStorageSlot.toBigInt() + BigInt(i));
      const newValue = values[i];
      const sideEffectCounter = this.sideEffectCounter.count();
      this.storageActions.write(storageSlot, newValue, sideEffectCounter);
      await this.stateDb.storageWrite(this.execution.callContext.storageContractAddress, storageSlot, newValue);
      this.log.debug(`Oracle storage write: slot=${storageSlot.toString()} value=${newValue.toString()}`);
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
  public override async callPublicFunction(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    isStaticCall = isStaticCall || this.execution.callContext.isStaticCall;

    const args = this.packedValuesCache.unpack(argsHash);
    this.log.verbose(
      `Public function call: addr=${targetContractAddress} selector=${functionSelector} args=${args.join(',')}`,
    );

    const portalAddress = (await this.contractsDb.getPortalContractAddress(targetContractAddress)) ?? EthAddress.ZERO;
    const functionData = new FunctionData(functionSelector, /*isPrivate=*/ false);
    const { transactionFee, gasSettings, gasLeft } = this.execution.callContext;
    const callContext = CallContext.from({
      msgSender: isDelegateCall ? this.execution.callContext.msgSender : this.execution.contractAddress,
      storageContractAddress: isDelegateCall ? this.execution.contractAddress : targetContractAddress,
      portalContractAddress: portalAddress,
      functionSelector,
      gasLeft, // Propagate the same gas left as when we started since ACVM public functions don't have any metering
      isDelegateCall,
      isStaticCall,
      sideEffectCounter,
      gasSettings,
      transactionFee,
    });

    const nestedExecution: PublicExecution = {
      args,
      contractAddress: targetContractAddress,
      functionData,
      callContext,
    };

    const context = new PublicExecutionContext(
      nestedExecution,
      this.header,
      this.globalVariables,
      this.packedValuesCache,
      this.sideEffectCounter,
      this.stateDb,
      this.contractsDb,
      this.commitmentsDb,
      this.log,
    );

    const childExecutionResult = await executePublicFunction(context, /*nested=*/ true);

    if (isStaticCall) {
      checkValidStaticCall(
        childExecutionResult.newNoteHashes,
        childExecutionResult.newNullifiers,
        childExecutionResult.contractStorageUpdateRequests,
        childExecutionResult.newL2ToL1Messages,
        childExecutionResult.unencryptedLogs,
      );
    }

    this.nestedExecutions.push(childExecutionResult);
    this.log.debug(`Returning from nested call: ret=${childExecutionResult.returnValues.join(', ')}`);

    return childExecutionResult.returnValues;
  }

  public override async checkNullifierExists(nullifier: Fr): Promise<boolean> {
    const witness = await this.commitmentsDb.getNullifierMembershipWitnessAtLatestBlock(nullifier);
    return !!witness;
  }

  public override async getContractInstance(address: AztecAddress): Promise<ContractInstance> {
    // Note to AVM implementor: The wrapper of the oracle call get_contract_instance in aztec-nr
    // automatically checks that the returned instance is correct, by hashing it together back
    // into the address. However, in the AVM, we also need to prove the negative, otherwise a malicious
    // sequencer could just lie about not having the instance available in its local db. We can do this
    // by using the prove_contract_non_deployment_at method if the contract is not found in the db.
    const instance = await this.contractsDb.getContractInstance(address);
    if (!instance) {
      throw new Error(`Contract instance at ${address} not found`);
    }
    return instance;
  }
}
