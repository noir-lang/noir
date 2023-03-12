import { keccak256String } from '../../crypto/index.js';
import { EthAddress } from '../../eth_address/index.js';
import { numberToHex } from '../../hex_string/index.js';
import { TxHash } from '../tx_hash.js';

export interface RawLogResponse {
  id?: string;
  removed?: boolean;
  logIndex: string | null;
  blockNumber: string | null;
  blockHash: string | null;
  transactionHash: string | null;
  transactionIndex: string | null;
  address: string;
  data: string;
  topics: string[];
}

export interface LogResponse {
  id: string | null;
  removed?: boolean;
  logIndex: number | null;
  blockNumber: number | null;
  blockHash: string | null;
  transactionHash: TxHash | null;
  transactionIndex: number | null;
  address: EthAddress;
  data: string;
  topics: string[];
}

export function fromRawLogResponse(log: RawLogResponse): LogResponse {
  let id: string | null = log.id || null;

  // Generate a custom log id.
  if (
    typeof log.blockHash === 'string' &&
    typeof log.transactionHash === 'string' &&
    typeof log.logIndex === 'string'
  ) {
    const shaId = keccak256String(
      log.blockHash.replace('0x', '') + log.transactionHash.replace('0x', '') + log.logIndex.replace('0x', ''),
    );
    id = 'log_' + shaId.replace('0x', '').substring(0, 8);
  }

  const blockNumber = log.blockNumber !== null ? Number(log.blockNumber) : null;
  const transactionIndex = log.transactionIndex !== null ? Number(log.transactionIndex) : null;
  const logIndex = log.logIndex !== null ? Number(log.logIndex) : null;
  const address = EthAddress.fromString(log.address);
  const transactionHash = log.transactionHash !== null ? TxHash.fromString(log.transactionHash) : null;

  return { ...log, id, blockNumber, transactionIndex, transactionHash, logIndex, address };
}

export function toRawLogResponse(log: LogResponse): RawLogResponse {
  const { id, blockNumber, transactionIndex, logIndex, address, transactionHash } = log;
  return {
    ...log,
    id: id ? id : undefined,
    blockNumber: blockNumber !== null ? numberToHex(blockNumber) : null,
    transactionIndex: transactionIndex !== null ? numberToHex(transactionIndex) : null,
    logIndex: logIndex !== null ? numberToHex(logIndex) : null,
    address: address.toString().toLowerCase(),
    transactionHash: transactionHash !== null ? transactionHash.toString() : null,
  };
}
