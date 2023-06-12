import { AztecAddress, Fr, TxContext } from '@aztec/circuits.js';
import { KeyStore, PublicKey, getAddressFromPublicKey } from '@aztec/key-store';
import { ExecutionRequest, SignedTxExecutionRequest, TxExecutionRequest } from '@aztec/types';
import { AccountImplementation } from './index.js';

/** Account implementation backed by an EOA */
export class EcdsaExternallyOwnedAccount implements AccountImplementation {
  constructor(private address: AztecAddress, private pubKey: PublicKey, private keyStore: KeyStore) {
    if (!address.equals(getAddressFromPublicKey(pubKey))) {
      throw new Error(`Address and public key don't match for EOA`);
    }
  }

  async createAuthenticatedTxRequest(
    executions: ExecutionRequest[],
    txContext: TxContext,
  ): Promise<SignedTxExecutionRequest> {
    if (executions.length !== 1) throw new Error(`EOAs can only submit a single execution at a time`);
    const [execution] = executions;

    if (!execution.from.equals(this.address)) throw new Error(`Sender does not match account address`);

    const txExecRequest = new TxExecutionRequest(
      this.address,
      execution.to,
      execution.functionData,
      execution.args,
      Fr.random(),
      txContext,
      Fr.ZERO,
    );
    const txRequest = await txExecRequest.toTxRequest();
    const toSign = txRequest.toBuffer();
    const signature = await this.keyStore.ecdsaSign(toSign, this.pubKey);
    return new SignedTxExecutionRequest(txExecRequest, signature);
  }
}
