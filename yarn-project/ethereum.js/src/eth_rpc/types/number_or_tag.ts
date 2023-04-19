import { numberToHex } from '../../hex_string/index.js';

/**
 * Type representing Ethereum block numbers or tags, used to specify a particular block when querying blockchain data.
 */
export type NumberOrTag = number | 'latest' | 'earliest' | 'pending';

export const numberOrTagToHex = (numberOrTag: NumberOrTag) =>
  typeof numberOrTag === 'number' ? numberToHex(numberOrTag) : numberOrTag;
