import { AztecAddress } from '../circuits.js';
import { TxHash } from './tx_hash.js';

export interface TxReceipt {
  txHash: TxHash;
  // txIndex: number;
  blockHash: Buffer;
  blockNumber: number;
  from: AztecAddress;
  to?: AztecAddress;
  contractAddress?: AztecAddress;
  status: boolean;
  error: string;
}
