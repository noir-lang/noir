import { EthAddress } from '@aztec/foundation';
import { bufferToHex } from '../../hex_string/index.js';
import { NumberOrTag, numberOrTagToHex } from './number_or_tag.js';

/**
 * Represents a log request configuration for Ethereum events.
 * Contains optional filter criteria, block range, contract addresses, and topics to refine the log search.
 */
export interface LogRequest<Event = { [k: string]: any }> {
  /**
   * Filter object for specifying event fields to be matched in logs.
   */
  filter?: Partial<Event>;
  /**
   * The block number or block tag to end the log search.
   */
  toBlock?: NumberOrTag;
  /**
   * The starting block number or identifier to fetch logs from.
   */
  fromBlock?: NumberOrTag;
  /**
   * Ethereum address or array of addresses to filter logs by.
   */
  address?: EthAddress | EthAddress[];
  /**
   * An array of topic filters for log events, allowing the selection of specific events or a combination thereof.
   */
  topics?: (Buffer | Buffer[] | null)[];
}

/**
 * Represents a raw log request object for querying logs from Ethereum nodes.
 * Contains optional parameters to filter and format the logs based on block range, address, and topics.
 */
export interface RawLogRequest {
  /**
   * The block number or tag until which logs should be fetched, inclusive.
   */
  toBlock?: string;
  /**
   * The starting block for the log search, inclusive.
   */
  fromBlock?: string;
  /**
   * Ethereum address or an array of addresses, used as a filter for the logs.
   */
  address?: string | string[];
  /**
   * An array of topics used for filtering specific event logs.
   */
  topics?: ((string | null) | (string | null)[])[];
}

/**
 * Converts a LogRequest object into a RawLogRequest object with hex string values.
 * This function is useful for preparing log requests to be sent to Ethereum nodes.
 *
 * @param logRequest - A LogRequest object containing the filter, block range, address, and topics for events.
 * @returns A RawLogRequest object with converted hex string values for block numbers and topics.
 */
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
