import { default as hash } from 'hash.js';

import { toBigIntBE, toBufferBE } from '../../bigint-buffer/index.js';
import { Fr } from '../../fields/fields.js';

export const sha256 = (data: Buffer) => Buffer.from(hash.sha256().update(data).digest());

/**
 * Squashes the output of sha256 into a field element.
 * WARNING: if you have not thought about why you are using this, or talked to somebody who has do not use it.
 * @param buf - Input buffer
 * @returns Returns a sha256 output squashed into a field element.
 */
export const sha256ToField = (buf: Buffer): Fr => {
  return Fr.fromBuffer(toBufferBE(toBigIntBE(sha256(buf)) % Fr.MODULUS, 32));
};
