import { EthAddress } from '@aztec/foundation';
import { LogResponse, TransactionReceipt, TxHash } from '../eth_rpc/index.js';
import { DecodedError } from './decode_error.js';

export interface EventLog<Args, Name = string> {
  id: string | null;
  removed?: boolean;
  event: Name;
  address: EthAddress;
  args: Args;
  logIndex: number | null;
  transactionIndex: number | null;
  transactionHash: TxHash | null;
  blockHash: string | null;
  blockNumber: number | null;
  raw: { data: string; topics: string[] };
  signature: string | null;
}

export interface ContractTxReceipt<Events = void> extends TransactionReceipt {
  anonymousLogs: LogResponse[];
  events: Events extends void ? { [eventName: string]: EventLog<any>[] } : Events;
  error?: { message: string; decodedError?: DecodedError };
}
