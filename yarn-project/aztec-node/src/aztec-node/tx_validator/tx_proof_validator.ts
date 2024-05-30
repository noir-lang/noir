import { type ClientProtocolCircuitVerifier, Tx, type TxValidator } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';

export class TxProofValidator implements TxValidator<Tx> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:private_proof');

  constructor(private verifier: ClientProtocolCircuitVerifier) {}

  async validateTxs(txs: Tx[]): Promise<[validTxs: Tx[], invalidTxs: Tx[]]> {
    const validTxs: Tx[] = [];
    const invalidTxs: Tx[] = [];

    for (const tx of txs) {
      if (await this.verifier.verifyProof(tx)) {
        validTxs.push(tx);
      } else {
        this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for invalid proof`);
        invalidTxs.push(tx);
      }
    }

    return [validTxs, invalidTxs];
  }

  validateTx(tx: Tx): Promise<boolean> {
    return this.verifier.verifyProof(tx);
  }
}
