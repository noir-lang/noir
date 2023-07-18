import { CallContext, FunctionData } from '@aztec/circuits.js';
import { FunctionAbi, decodeReturnValues } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import { extractReturnWitness, frToAztecAddress } from '../acvm/deserialize.js';
import { ACVMField, ZERO_ACVM_FIELD, acvm, fromACVMField, toACVMField, toACVMWitness } from '../acvm/index.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { oracleDebugCallToFormattedStr } from './debug.js';

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

    const acir = Buffer.from(this.abi.bytecode, 'base64');
    const initialWitness = toACVMWitness(1, this.args);

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: ([ownerX], [ownerY]) => this.context.getSecretKey(this.contractAddress, ownerX, ownerY),
      getPublicKey: async ([acvmAddress]) => {
        const address = frToAztecAddress(fromACVMField(acvmAddress));
        const [pubKey, partialContractAddress] = await this.context.db.getPublicKey(address);
        return [pubKey.x, pubKey.y, partialContractAddress].map(toACVMField);
      },
      getNotes: ([slot], sortBy, sortOrder, [limit], [offset], [returnSize]) =>
        this.context.getNotes(this.contractAddress, slot, sortBy, sortOrder, limit, offset, returnSize),
      getRandomField: () => Promise.resolve(toACVMField(Fr.random())),
      debugLog: (...params) => {
        this.log(oracleDebugCallToFormattedStr(params));
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      getL1ToL2Message: ([msgKey]) => this.context.getL1ToL2Message(fromACVMField(msgKey)),
      getCommitment: ([commitment]) => this.context.getCommitment(this.contractAddress, commitment),
    });

    const returnValues: ACVMField[] = extractReturnWitness(acir, partialWitness);

    return decodeReturnValues(this.abi, returnValues.map(fromACVMField));
  }
}
