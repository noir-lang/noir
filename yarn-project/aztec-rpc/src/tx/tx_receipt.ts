import { TxHash } from '@aztec/tx';
import { AztecAddress } from '@aztec/circuits.js';

export interface TxReceipt {
  txHash: TxHash;
  // txIndex: number;
  blockHash: Buffer | undefined;
  blockNumber: number | undefined;
  from: AztecAddress;
  to?: AztecAddress;
  contractAddress?: AztecAddress;
  status: boolean;
  error: string;
}
