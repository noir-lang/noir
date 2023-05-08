import { AztecAddress, CallContext, EthAddress, Fr, FunctionData, TxRequest } from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { select_return_flattened as selectPublicWitnessFlattened } from '@noir-lang/noir_util_wasm';
import { acvm, frToAztecAddress, frToSelector, fromACVMField, toACVMField, toACVMWitness } from '../acvm/index.js';
import { PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution, PublicExecutionResult } from './execution.js';
import { StateActionsCollector } from './state_actions.js';

// Copied from crate::abi at noir-contracts/src/contracts/noir-aztec3/src/abi.nr
const NOIR_MAX_RETURN_VALUES = 4;

/**
 * Handles execution of public functions.
 */
export class PublicExecutor {
  constructor(
    private readonly stateDb: PublicStateDB,
    private readonly contractsDb: PublicContractsDB,

    private log = createDebugLogger('aztec:simulator:public-executor'),
  ) {}

  /**
   * Executes a public execution request.
   * @param execution - The execution to run.
   * @returns The result of the run plus all nested runs.
   */
  public async execute(execution: PublicExecution): Promise<PublicExecutionResult> {
    const selectorHex = execution.functionData.functionSelector.toString('hex');
    this.log(`Executing public external function ${execution.contractAddress.toShortString()}:${selectorHex}`);

    const selector = execution.functionData.functionSelector;
    const acir = await this.contractsDb.getBytecode(execution.contractAddress, selector);
    if (!acir) throw new Error(`Bytecode not found for ${execution.contractAddress.toShortString()}:${selectorHex}`);

    const initialWitness = getInitialWitness(execution.args, execution.callContext);
    const stateActions = new StateActionsCollector(this.stateDb, execution.contractAddress);
    const nestedExecutions: PublicExecutionResult[] = [];

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
      callPublicFunction: async ([address, functionSelector, ...args]) => {
        this.log(`Public function call: addr=${address} selector=${functionSelector} args=${args.join(',')}`);
        const childExecutionResult = await this.callPublicFunction(
          frToAztecAddress(fromACVMField(address)),
          frToSelector(fromACVMField(functionSelector)),
          args.map(f => fromACVMField(f)),
          execution.callContext,
        );

        nestedExecutions.push(childExecutionResult);
        this.log(`Returning from nested call: ret=${childExecutionResult.returnValues.join(', ')}`);
        return padArrayEnd(childExecutionResult.returnValues, Fr.ZERO, NOIR_MAX_RETURN_VALUES).map(fr => fr.toString());
      },
    });

    const returnValues = selectPublicWitnessFlattened(acir, partialWitness).map(fromACVMField);
    const [stateReads, stateTransitions] = stateActions.collect();

    return {
      stateReads,
      stateTransitions,
      returnValues,
      nestedExecutions,
    };
  }

  /**
   * Creates a PublicExecution out of a TxRequest to a public function.
   * @param input - The TxRequest calling a public function.
   * @returns A PublicExecution object that can be run via execute.
   */
  public async getPublicExecution(input: TxRequest): Promise<PublicExecution> {
    const contractAddress = input.to;
    const portalContractAddress = (await this.contractsDb.getPortalContractAddress(contractAddress)) ?? EthAddress.ZERO;
    const callContext: CallContext = new CallContext(input.from, input.to, portalContractAddress, false, false, false);

    return { callContext, contractAddress, functionData: input.functionData, args: input.args };
  }

  private async callPublicFunction(
    targetContractAddress: AztecAddress,
    targetFunctionSelector: Buffer,
    targetArgs: Fr[],
    callerContext: CallContext,
  ) {
    const portalAddress = (await this.contractsDb.getPortalContractAddress(targetContractAddress)) ?? EthAddress.ZERO;
    const functionData = new FunctionData(targetFunctionSelector, false, false);

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

    return this.execute(nestedExecution);
  }
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
