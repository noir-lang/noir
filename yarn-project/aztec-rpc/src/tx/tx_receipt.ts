import { TxHash } from '@aztec/tx';
import { AztecAddress } from '../circuits.js';

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
