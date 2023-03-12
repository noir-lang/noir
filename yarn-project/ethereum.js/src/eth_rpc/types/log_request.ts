import { EthAddress } from '../../eth_address/index.js';
import { bufferToHex } from '../../hex_string/index.js';
import { NumberOrTag, numberOrTagToHex } from './number_or_tag.js';

export interface LogRequest<Event = { [k: string]: any }> {
  filter?: Partial<Event>;
  toBlock?: NumberOrTag;
  fromBlock?: NumberOrTag;
  address?: EthAddress | EthAddress[];
  topics?: (Buffer | Buffer[] | null)[];
}

export interface RawLogRequest {
  toBlock?: string;
  fromBlock?: string;
  address?: string | string[];
  topics?: ((string | null) | (string | null)[])[];
}

export function toRawLogRequest(logRequest: LogRequest = {}): RawLogRequest {
  const rawLogRequest: RawLogRequest = {};

  if (logRequest.fromBlock !== undefined) {
    rawLogRequest.fromBlock = numberOrTagToHex(logRequest.fromBlock);
  }

  if (logRequest.toBlock !== undefined) {
    rawLogRequest.toBlock = numberOrTagToHex(logRequest.toBlock);
  }

  // Convert topics to hex.
  rawLogRequest.topics = (logRequest.topics || []).map(topic => {
    const toTopic = (value: Buffer | null) => {
      if (!value) {
        return null;
      }
      return bufferToHex(value);
    };
    return Array.isArray(topic) ? topic.map(toTopic) : toTopic(topic);
  });

  if (logRequest.address) {
    rawLogRequest.address = Array.isArray(logRequest.address)
      ? logRequest.address.map(a => a.toString().toLowerCase())
      : logRequest.address.toString().toLowerCase();
  }

  return rawLogRequest;
}

// export function fromRawLogRequest(rawLogRequest: RawLogRequest): LogRequest {
//   const { toBlock, fromBlock, address, topics } = rawLogRequest;
//   return {
//     toBlock: toBlock ? hexToNumber(toBlock) : undefined,
//     fromBlock: fromBlock ? hexToNumber(fromBlock) : undefined,
//     address: address
//       ? Array.isArray(address)
//         ? address.map(EthAddress.fromString)
//         : EthAddress.fromString(address)
//       : undefined,
//     topics: topics
//       ? topics.map(topicOrArr => (Array.isArray(topicOrArr) ? topicOrArr.map(t => hexToBuffer(t) : null) : hexToBuffer(topicOrArr!)))
//       : undefined,
//   };
// }
