import { CallContext, FunctionData } from '@aztec/circuits.js';
import { FunctionAbi } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { select_return_flattened as selectReturnFlattened } from '@noir-lang/noir_util_wasm';
import { decodeReturnValues } from '../abi_coder/decoder.js';
import { frToNumber } from '../acvm/deserialize.js';
import { ACVMField, ZERO_ACVM_FIELD, acvm, fromACVMField, toACVMField, toACVMWitness } from '../acvm/index.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { fieldsToFormattedStr } from './debug.js';

const notAvailable = () => {
  return Promise.reject(new Error(`Not available for unconstrained function execution`));
};

/**
 * The unconstrained function execution class.
 */
export class UnconstrainedFunctionExecution {
  constructor(
    private context: ClientTxExecutionContext,
    private abi: FunctionAbi,
    private contractAddress: AztecAddress,
    private functionData: FunctionData,
    private args: Fr[],
    _: CallContext, // not used ATM

    private log = createDebugLogger('aztec:simulator:unconstrained_execution'),
  ) {}

  /**
   * Executes the unconstrained function.
   * @returns The return values of the executed function.
   */
  public async run(): Promise<any[]> {
    this.log(
      `Executing unconstrained function ${this.contractAddress.toShortString()}:${this.functionData.functionSelectorBuffer.toString(
        'hex',
      )}`,
    );

    const acir = Buffer.from(this.abi.bytecode, 'hex');
    const initialWitness = toACVMWitness(1, this.args);

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: async ([ownerX, ownerY]: ACVMField[]) => [
        toACVMField(
          await this.context.db.getSecretKey(
            this.contractAddress,
            Point.fromCoordinates(fromACVMField(ownerX), fromACVMField(ownerY)),
          ),
        ),
      ],
      getNotes2: async ([storageSlot]: ACVMField[]) => {
        const { preimages } = await this.context.getNotes(this.contractAddress, storageSlot, 2);
        return preimages;
      },
      getRandomField: () => Promise.resolve([toACVMField(Fr.random())]),
      viewNotesPage: ([acvmSlot, acvmLimit, acvmOffset]) =>
        this.context.viewNotes(
          this.contractAddress,
          acvmSlot,
          frToNumber(fromACVMField(acvmLimit)),
          frToNumber(fromACVMField(acvmOffset)),
        ),
      debugLog: (fields: ACVMField[]) => {
        this.log(fieldsToFormattedStr(fields));
        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
      getL1ToL2Message: ([msgKey]: ACVMField[]) => this.context.getL1ToL2Message(fromACVMField(msgKey)),
      getCommitment: ([commitment]: ACVMField[]) =>
        this.context
          .getCommitment(this.contractAddress, fromACVMField(commitment))
          .then(commitmentData => commitmentData.acvmData),
      enqueuePublicFunctionCall: notAvailable,
      notifyCreatedNote: notAvailable,
      notifyNullifiedNote: notAvailable,
      callPrivateFunction: notAvailable,
      callPublicFunction: notAvailable,
      storageRead: notAvailable,
      storageWrite: notAvailable,
      createCommitment: notAvailable,
      createL2ToL1Message: notAvailable,
      emitEncryptedLog: notAvailable,
      emitUnencryptedLog: notAvailable,
    });

    const returnValues: ACVMField[] = selectReturnFlattened(acir, partialWitness);

    return decodeReturnValues(this.abi, returnValues.map(fromACVMField));
  }
}
