import { AztecAddress } from '../circuits.js';
import { Database } from '../database/index.js';
import { TxHash } from '../tx/index.js';

export class AccountState {
  constructor(public readonly publicKey: AztecAddress, private db: Database) {}

  getTx(txHash: TxHash) {
    return this.db.getTx(txHash);
  }
}
