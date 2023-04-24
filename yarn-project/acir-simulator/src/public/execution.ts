import { CallContext, FunctionData, StateRead, StateTransition, TxRequest } from '@aztec/circuits.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { createDebugLogger } from '@aztec/foundation/log';
import { select_return_flattened as selectPublicWitnessFlattened } from '@noir-lang/noir_util_wasm';
import { acvm, fromACVMField, toACVMField, toACVMWitness } from '../acvm/index.js';
import { PublicDB } from './db.js';
import { StateActionsCollector } from './state_actions.js';

export interface PublicFunctionBytecode {
  bytecode: Buffer;
}

export interface PublicExecutionResult {
  returnValues: Fr[];
  stateReads: StateRead[];
  stateTransitions: StateTransition[];
}

function getInitialWitness(args: Fr[], callContext: CallContext, witnessStartIndex = 1) {
  return toACVMWitness(
    witnessStartIndex,
    callContext.isContractDeployment,
    callContext.isDelegateCall,
    callContext.isStaticCall,
    callContext.msgSender,
    callContext.portalContractAddress,
    callContext.storageContractAddress,
    ...args,
  );
}

export class PublicExecution {
  constructor(
    public readonly db: PublicDB,
    public readonly publicFunction: PublicFunctionBytecode,
    public readonly contractAddress: AztecAddress,
    public readonly functionData: FunctionData,
    public readonly args: Fr[],
    public readonly callContext: CallContext,

    private log = createDebugLogger('aztec:simulator:public-execution'),
  ) {}

  static fromTransactionRequest(
    db: PublicDB,
    request: TxRequest,
    bytecode: PublicFunctionBytecode,
    portalContractAddress: EthAddress,
  ) {
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

  public async run(): Promise<PublicExecutionResult> {
    const selectorHex = this.functionData.functionSelector.toString('hex');
    this.log(`Executing public external function ${this.contractAddress.toShortString()}:${selectorHex}`);

    const acir = this.publicFunction.bytecode;
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
