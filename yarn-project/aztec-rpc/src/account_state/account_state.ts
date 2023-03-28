import { AztecAddress } from '@aztec/circuits.js';
import { TxHash } from '@aztec/tx';
import { Database } from '../database/index.js';

export class AccountState {
  constructor(public readonly publicKey: AztecAddress, private db: Database) {}

  getTx(txHash: TxHash) {
    return this.db.getTx(txHash);
  }
}
