import { CallContext, FunctionData, StateRead, StateTransition, TxRequest } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { select_return_flattened as selectPublicWitnessFlattened } from '@noir-lang/noir_util_wasm';
import { acvm, fromACVMField, toACVMField, toACVMWitness } from '../acvm/index.js';
import { PublicDB } from './db.js';
import { StateActionsCollector } from './state_actions.js';
import { Fr } from '@aztec/foundation/fields';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

/**
 * The public function execution result.
 */
export interface PublicExecutionResult {
  /** The return values of the function. */
  returnValues: Fr[];
  /** The state reads performed by the function. */
  stateReads: StateRead[];
  /** The state transitions performed by the function. */
  stateTransitions: StateTransition[];
}

/**
 * Generates the initial witness for a public function.
 * @param args - The arguments to the function.
 * @param callContext - The call context of the function.
 * @param witnessStartIndex - The index where to start inserting the parameters.
 * @returns The initial witness.
 */
function getInitialWitness(args: Fr[], callContext: CallContext, witnessStartIndex = 1) {
  return toACVMWitness(witnessStartIndex, [
    callContext.isContractDeployment,
    callContext.isDelegateCall,
    callContext.isStaticCall,
    callContext.msgSender,
    callContext.portalContractAddress,
    callContext.storageContractAddress,
    ...args,
  ]);
}

/**
 * The public function execution class.
 */
export class PublicExecution {
  constructor(
    /** The public database. */
    public readonly db: PublicDB,
    /** The ACIR bytecode of the public function. */
    public readonly publicFunctionBytecode: Buffer,
    /** The address of the contract to execute. */
    public readonly contractAddress: AztecAddress,
    /** The function data of the function to execute. */
    public readonly functionData: FunctionData,
    /** The arguments of the function to execute. */
    public readonly args: Fr[],
    /** The call context of the execution. */
    public readonly callContext: CallContext,

    private log = createDebugLogger('aztec:simulator:public-execution'),
  ) {}

  /**
   * Creates a public function execution from a transaction request.
   * @param db - The public database.
   * @param request - The transaction request.
   * @param bytecode - The bytecode of the public function.
   * @param portalContractAddress - The address of the portal contract.
   * @returns The public function execution.
   */
  static fromTransactionRequest(db: PublicDB, request: TxRequest, bytecode: Buffer, portalContractAddress: EthAddress) {
    const contractAddress = request.to;
    const callContext: CallContext = new CallContext(
      request.from,
      request.to,
      portalContractAddress,
      false,
      false,
      false,
    );
    return new this(db, bytecode, contractAddress, request.functionData, request.args, callContext);
  }

  /**
   * Executes the public function.
   * @returns The execution result.
   */
  public async run(): Promise<PublicExecutionResult> {
    const selectorHex = this.functionData.functionSelector.toString('hex');
    this.log(`Executing public external function ${this.contractAddress.toShortString()}:${selectorHex}`);

    const acir = this.publicFunctionBytecode;
    const initialWitness = getInitialWitness(this.args, this.callContext);
    const stateActions = new StateActionsCollector(this.db, this.contractAddress);

    const notAvailable = () => Promise.reject(`Built-in not available for public execution simulation`);

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: notAvailable,
      getNotes2: notAvailable,
      getRandomField: notAvailable,
      notifyCreatedNote: notAvailable,
      notifyNullifiedNote: notAvailable,
      callPrivateFunction: notAvailable,
      viewNotesPage: notAvailable,
      storageRead: async ([slot]) => {
        const storageSlot = fromACVMField(slot);
        const value = await stateActions.read(storageSlot);
        this.log(`Oracle storage read: slot=${storageSlot.toShortString()} value=${value.toString()}`);
        return [toACVMField(value)];
      },
      storageWrite: async ([slot, value]) => {
        const storageSlot = fromACVMField(slot);
        const newValue = fromACVMField(value);
        await stateActions.write(storageSlot, newValue);
        this.log(`Oracle storage write: slot=${storageSlot.toShortString()} value=${value.toString()}`);
        return [toACVMField(newValue)];
      },
    });

    const returnValues = selectPublicWitnessFlattened(acir, partialWitness).map(fromACVMField);
    const [stateReads, stateTransitions] = stateActions.collect();

    return {
      stateReads,
      stateTransitions,
      returnValues,
    };
  }
}
