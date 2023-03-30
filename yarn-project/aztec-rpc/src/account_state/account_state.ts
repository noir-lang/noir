import { AztecAddress } from '@aztec/circuits.js';
import { L2Block } from '@aztec/l2-block';
import { TxHash } from '@aztec/tx';
import { Database } from '../database/index.js';

export class AccountState {
  public syncedTo = 0;
  constructor(public readonly publicKey: AztecAddress, private db: Database) {}

  getTx(txHash: TxHash) {
    return this.db.getTx(txHash);
  }

  syncToBlock(block: L2Block) {
    this.syncedTo = block.number;
  }
}
