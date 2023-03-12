import { numberToHex } from '../../hex_string/index.js';

export type NumberOrTag = number | 'latest' | 'earliest' | 'pending';

export const numberOrTagToHex = (numberOrTag: NumberOrTag) =>
  typeof numberOrTag === 'number' ? numberToHex(numberOrTag) : numberOrTag;
