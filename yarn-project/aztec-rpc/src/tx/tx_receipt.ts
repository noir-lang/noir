import { TxHash } from '@aztec/types';
import { AztecAddress } from '@aztec/foundation';

export enum TxStatus {
  DROPPED = 'dropped',
  MINED = 'mined',
  PENDING = 'pending',
}

export interface TxReceipt {
  txHash: TxHash;
  blockHash?: Buffer;
  blockNumber?: number;
  from?: AztecAddress;
  to?: AztecAddress;
  contractAddress?: AztecAddress;
  status: TxStatus;
  error: string;
}
